//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//  Allocator API copyright 2015 The Rust Project Developers.
//  See the COPYRIGHT file at http://rust-lang.org/COPYRIGHT.
//
//  Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
//  http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
//  <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
//  option. This file may not be copied, modified, or distributed
//  except according to those terms.
//! SOS memory allocation library
//!
//! This is in its own crate so it can be used by kernel-space and user-space
//! OS components.
//!
//! We reproduce the Rust [`Allocator`] trait described in Rust [RFC 1398], so
//! that our allocators can implement it. Once RFC 1398 is implemented, we will
//! be able to remove the [`Allocator`] and [`Layout`] types from this crate.
//!
//! For more information on the Allocator API, refer to:
//! + the text of [RFC 1398]
//! + rust-lang/rust [tracking issue]
//!
//! [`Allocator`]: trait.Allocator.html
//! [`Layout`]: struct.Layout.html
//! [RFC 1398]: https://github.com/rust-lang/rfcs/blob/master/text/1398-kinds-of-allocators.md
//! [tracking issue]: https://github.com/rust-lang/rust/issues/32838
#![crate_name = "sos_alloc"]
#![crate_type = "lib"]

// The compiler needs to be instructed that this crate is an allocator in order
// to realize that when this is linked in another allocator like jemalloc
// should not be linked in
#![cfg_attr( feature = "system", feature(allocator) )]
#![cfg_attr( feature = "system", allocator )]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![cfg_attr( feature = "placement_in"
           , feature(placement_new_protocol, placement_in_syntax))]

// Allocators are not allowed to depend on the standard library which in turn
// requires an allocator in order to avoid circular dependencies. This crate,
// however, can use all of libcore.
#![no_std]

#![feature(const_fn)]
#![feature(unique)]
#![feature(core_intrinsics)]
#![feature(step_trait)]

#![cfg_attr(all(test, feature = "bench"), feature(test))]
#![cfg_attr(test, feature(collections))]
#[cfg(all(test, feature = "bench"))] extern crate test;
#[cfg(test)] extern crate collections;

extern crate memory;

#[cfg(feature = "first_fit")]
extern crate arrayvec;

#[cfg(feature = "buddy")]
extern crate sos_intrusive as intrusive;

extern crate spin;

#[cfg(feature = "buddy_as_system")]
#[macro_use] extern crate once;

#[macro_use] extern crate log;

extern crate params;

use core::{cmp, ops, ptr, mem};
use ptr::Unique;

pub type AllocResult<T> = Result<T, AllocErr>;

pub type Size = usize;
pub type Capacity = usize;
pub type Alignment = usize;

pub type Address = *mut u8;

pub mod frame;
pub use frame::{Allocator as FrameAllocator, Lender as FrameLender};

/// Represents the combination of a starting address and
/// a total capacity of the returned block.
pub struct Excess(Address, Capacity);

fn size_align<T>() -> (usize, usize) {
    (mem::size_of::<T>(), mem::align_of::<T>())
}

/// Category for a memory record.
///
/// An instance of `Layout` describes a particular layout of memory.
/// You build a `Layout` up as an input to give to an allocator.
///
/// All layouts have an associated non-negative size and positive alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    // size of the requested block of memory, measured in bytes.
    size: Size,
    // alignment of the requested block of memory, measured in bytes.
    // we ensure that this is always a power-of-two, because API's
    // like `posix_memalign` require it and it is a reasonable
    // constraint to impose on Layout constructors.
    //
    // (However, we do not analogously require `align >= sizeof(void*)`,
    //  even though that is *also* a requirement of `posix_memalign`.)
    align: Alignment,
}


// FIXME: audit default implementations for overflow errors,
// (potentially switching to overflowing_add and
//  overflowing_mul as necessary).

impl Layout {
    // (private constructor)
    pub fn from_size_align(size: usize, align: usize) -> Layout {
        assert!(align.is_power_of_two());
        assert!(align > 0);
        Layout { size: size, align: align }
    }

    /// The minimum size in bytes for a memory block of this layout.
    pub fn size(&self) -> usize { self.size }

    /// The minimum byte alignment for a memory block of this layout.
    pub fn align(&self) -> usize { self.align }

    /// Constructs a `Layout` suitable for holding a value of type `T`.
    pub fn new<T>() -> Self {
        let (size, align) = size_align::<T>();
        Layout::from_size_align(size, align)
    }

    /// Produces layout describing a record that could be used to
    /// allocate backing structure for `T` (which could be a trait
    /// or other unsized type like a slice).
    pub fn for_value<T: ?Sized>(t: &T) -> Self {
        let (size, align) = (mem::size_of_val(t), mem::align_of_val(t));
        Layout::from_size_align(size, align)
    }

    /// Creates a layout describing the record that can hold a value
    /// of the same layout as `self`, but that also is aligned to
    /// alignment `align` (measured in bytes).
    ///
    /// If `self` already meets the prescribed alignment, then returns
    /// `self`.
    ///
    /// Note that this method does not add any padding to the overall
    /// size, regardless of whether the returned layout has a different
    /// alignment. In other words, if `K` has size 16, `K.align_to(32)`
    /// will *still* have size 16.
    pub fn align_to(&self, align: Alignment) -> Self {
        if align > self.align {
            let pow2_align = align.checked_next_power_of_two().unwrap();
            debug_assert!(pow2_align > 0); // (this follows from self.align > 0...)
            Layout { align: pow2_align,
                     ..*self }
        } else {
            self.clone()
        }
    }

    /// Returns the amount of padding we must insert after `self`
    /// to ensure that the following address will satisfy `align`
    /// (measured in bytes).
    ///
    /// Behavior undefined if `align` is not a power-of-two.
    ///
    /// Note that in practice, this is only useable if `align <=
    /// self.align` otherwise, the amount of inserted padding would
    /// need to depend on the particular starting address for the
    /// whole record, because `self.align` would not provide
    /// sufficient constraint.
    pub fn padding_needed_for(&self, align: Alignment) -> usize {
        debug_assert!(align <= self.align());
        let len = self.size();
        let len_rounded_up = (len + align - 1) & !(align - 1);
        return len_rounded_up - len;
    }

    /// Creates a layout describing the record for `n` instances of
    /// `self`, with a suitable amount of padding between each to
    /// ensure that each instance is given its requested size and
    /// alignment. On success, returns `(k, offs)` where `k` is the
    /// layout of the array and `offs` is the distance between the start
    /// of each element in the array.
    ///
    /// On arithmetic overflow, returns `None`.
    pub fn repeat(&self, n: usize) -> Option<(Self, usize)> {
        let padded_size = match self.size.checked_add(self.padding_needed_for(self.align)) {
            None => return None,
            Some(padded_size) => padded_size,
        };
        let alloc_size = match padded_size.checked_mul(n) {
            None => return None,
            Some(alloc_size) => alloc_size,
        };
        Some((Layout::from_size_align(alloc_size, self.align), padded_size))
    }

    /// Creates a layout describing the record for `self` followed by
    /// `next`, including any necessary padding to ensure that `next`
    /// will be properly aligned. Note that the result layout will
    /// satisfy the alignment properties of both `self` and `next`.
    ///
    /// Returns `Some((k, offset))`, where `k` is layout of the concatenated
    /// record and `offset` is the relative location, in bytes, of the
    /// start of the `next` embedded witnin the concatenated record
    /// (assuming that the record itself starts at offset 0).
    ///
    /// On arithmetic overflow, returns `None`.
    pub fn extend(&self, next: Self) -> Option<(Self, usize)> {
        let new_align = cmp::max(self.align, next.align);
        let realigned = Layout { align: new_align, ..*self };
        let pad = realigned.padding_needed_for(new_align);
        let offset = self.size() + pad;
        let new_size = offset + next.size();
        Some((Layout::from_size_align(new_size, new_align), offset))
    }

    /// Creates a layout describing the record for `n` instances of
    /// `self`, with no padding between each instance.
    ///
    /// On arithmetic overflow, returns `None`.
    pub fn repeat_packed(&self, n: usize) -> Option<Self> {
        let scaled = match self.size().checked_mul(n) {
            None => return None,
            Some(scaled) => scaled,
        };
        let size = { assert!(scaled > 0); scaled };
        Some(Layout { size: size, align: self.align })
    }

    /// Creates a layout describing the record for `self` followed by
    /// `next` with no additional padding between the two. Since no
    /// padding is inserted, the alignment of `next` is irrelevant,
    /// and is not incoporated *at all* into the resulting layout.
    ///
    /// Returns `(k, offset)`, where `k` is layout of the concatenated
    /// record and `offset` is the relative location, in bytes, of the
    /// start of the `next` embedded witnin the concatenated record
    /// (assuming that the record itself starts at offset 0).
    ///
    /// (The `offset` is always the same as `self.size()`; we use this
    ///  signature out of convenience in matching the signature of
    ///  `fn extend`.)
    ///
    /// On arithmetic overflow, returns `None`.
    pub fn extend_packed(&self, next: Self) -> Option<(Self, usize)> {
        let new_size = match self.size().checked_add(next.size()) {
            None => return None,
            Some(new_size) => new_size,
        };
        Some((Layout { size: new_size, ..*self }, self.size()))
    }

    // Below family of methods *assume* inputs are pre- or
    // post-validated in some manner. (The implementations here
    ///do indirectly validate, but that is not part of their
    /// specification.)
    //
    // Since invalid inputs could yield ill-formed layouts, these
    // methods are `unsafe`.

    /// Creates layout describing the record for a single instance of `T`.
    pub unsafe fn new_unchecked<T>() -> Self {
        let (size, align) = size_align::<T>();
        Layout::from_size_align(size, align)
    }


    /// Creates a layout describing the record for `self` followed by
    /// `next`, including any necessary padding to ensure that `next`
    /// will be properly aligned. Note that the result layout will
    /// satisfy the alignment properties of both `self` and `next`.
    ///
    /// Returns `(k, offset)`, where `k` is layout of the concatenated
    /// record and `offset` is the relative location, in bytes, of the
    /// start of the `next` embedded witnin the concatenated record
    /// (assuming that the record itself starts at offset 0).
    ///
    /// Requires no arithmetic overflow from inputs.
    pub unsafe fn extend_unchecked(&self, next: Self) -> (Self, usize) {
        self.extend(next).unwrap()
    }

    /// Creates a layout describing the record for `n` instances of
    /// `self`, with a suitable amount of padding between each.
    ///
    /// Requires non-zero `n` and no arithmetic overflow from inputs.
    /// (See also the `fn array` checked variant.)
    pub unsafe fn repeat_unchecked(&self, n: usize) -> (Self, usize) {
        self.repeat(n).unwrap()
    }

    /// Creates a layout describing the record for `n` instances of
    /// `self`, with no padding between each instance.
    ///
    /// Requires non-zero `n` and no arithmetic overflow from inputs.
    /// (See also the `fn array_packed` checked variant.)
    pub unsafe fn repeat_packed_unchecked(&self, n: usize) -> Self {
        self.repeat_packed(n).unwrap()
    }

    /// Creates a layout describing the record for `self` followed by
    /// `next` with no additional padding between the two. Since no
    /// padding is inserted, the alignment of `next` is irrelevant,
    /// and is not incoporated *at all* into the resulting layout.
    ///
    /// Returns `(k, offset)`, where `k` is layout of the concatenated
    /// record and `offset` is the relative location, in bytes, of the
    /// start of the `next` embedded witnin the concatenated record
    /// (assuming that the record itself starts at offset 0).
    ///
    /// (The `offset` is always the same as `self.size()`; we use this
    ///  signature out of convenience in matching the signature of
    ///  `fn extend`.)
    ///
    /// Requires no arithmetic overflow from inputs.
    /// (See also the `fn extend_packed` checked variant.)
    pub unsafe fn extend_packed_unchecked(&self, next: Self) -> (Self, usize) {
        self.extend_packed(next).unwrap()
    }

    /// Creates a layout describing the record for a `[T; n]`.
    ///
    /// On zero `n`, zero-sized `T`, or arithmetic overflow, returns `None`.
    pub fn array<T>(n: usize) -> Option<Self> {
        Layout::new::<T>()
            .repeat(n)
            .map(|(k, offs)| {
                debug_assert!(offs == mem::size_of::<T>());
                k
            })
    }

    /// Creates a layout describing the record for a `[T; n]`.
    ///
    /// Requires nonzero `n`, nonzero-sized `T`, and no arithmetic
    /// overflow; otherwise behavior undefined.
    pub fn array_unchecked<T>(n: usize) -> Self {
        Layout::array::<T>(n).unwrap()
    }

}

/// The `AllocErr` error specifies whether an allocation failure is
/// specifically due to resource exhaustion or if it is due to
/// something wrong when combining the given input arguments with this
/// allocator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AllocErr {
    /// Error due to hitting some resource limit or otherwise running
    /// out of memory. This condition strongly implies that *some*
    /// series of deallocations would allow a subsequent reissuing of
    /// the original allocation request to succeed.
    Exhausted { request: Layout },

    /// Error due to allocator being fundamentally incapable of
    /// satisfying the original request. This condition implies that
    /// such an allocation request will never succeed on the given
    /// allocator, regardless of environment, memory pressure, or
    /// other contextual condtions.
    ///
    /// For example, an allocator that does not support zero-sized
    /// blocks can return this error variant.
    Unsupported { details: &'static str },
}

impl AllocErr {
    pub fn invalid_input(details: &'static str) -> Self {
        AllocErr::Unsupported { details: details }
    }
    pub fn is_memory_exhausted(&self) -> bool {
        if let AllocErr::Exhausted { .. } = *self { true } else { false }
    }
    pub fn is_request_unsupported(&self) -> bool {
        if let AllocErr::Unsupported { .. } = *self { true } else { false }
    }
}

/// The `CannotReallocInPlace` error is used when `fn realloc_in_place`
/// was unable to reuse the given memory block for a requested layout.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CannotReallocInPlace;

/// An implementation of `Allocator` can allocate, reallocate, and
/// deallocate arbitrary blocks of data described via `Layout`.
///
/// Some of the methods require that a layout *fit* a memory block.
/// What it means for a layout to "fit" a memory block means is that
/// the following two conditions must hold:
///
/// 1. The block's starting address must be aligned to `layout.align()`.
///
/// 2. The block's size must fall in the range `[use_min, use_max]`, where:
///
///    * `use_min` is `self.usable_size(layout).0`, and
///
///    * `use_max` is the capacity that was (or would have been)
///      returned when (if) the block was allocated via a call to
///      `alloc_excess` or `realloc_excess`.
///
/// Note that:
///
///  * the size of the layout most recently used to allocate the block
///    is guaranteed to be in the range `[use_min, use_max]`, and
///
///  * a lower-bound on `use_max` can be safely approximated by a call to
///    `usable_size`.
///
pub unsafe trait Allocator {
    /// Returns a pointer suitable for holding data described by
    /// `layout`, meeting its size and alignment guarantees.
    ///
    /// The returned block of storage may or may not have its contents
    /// initialized. (Extension subtraits might restrict this
    /// behavior, e.g. to ensure initialization.)
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout`
    /// does not meet allocator's size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory
    /// exhaustion rather than panicking or aborting, but this is
    /// not a strict requirement. (Specifically: it is *legal* to use
    /// this trait to wrap an underlying native allocation library
    /// that aborts on memory exhaustion.)
    unsafe fn alloc(&mut self, layout: Layout) -> Result<Address, AllocErr>;

    /// Deallocate the memory referenced by `ptr`.
    ///
    /// `ptr` must have previously been provided via this allocator,
    /// and `layout` must *fit* the provided block (see above);
    /// otherwise yields undefined behavior.
    unsafe fn dealloc(&mut self, ptr: Address, layout: Layout);

    /// Allocator-specific method for signalling an out-of-memory
    /// condition.
    ///
    /// Implementations of the `oom` method are discouraged from
    /// infinitely regressing in nested calls to `oom`. In
    /// practice this means implementors should eschew allocating,
    /// especially from `self` (directly or indirectly).
    ///
    /// Implementions of this trait's allocation methods are discouraged
    /// from panicking (or aborting) in the event of memory exhaustion;
    /// instead they should return an appropriate error from the
    /// invoked method, and let the client decide whether to invoke
    /// this `oom` method.
    fn oom(&mut self, _: AllocErr) -> ! {
        unsafe { ::core::intrinsics::abort() }
    }

    // == ALLOCATOR-SPECIFIC QUANTITIES AND LIMITS ==
    // usable_size

    /// Returns bounds on the guaranteed usable size of a successful
    /// allocation created with the specified `layout`.
    ///
    /// In particular, for a given layout `k`, if `usable_size(k)` returns
    /// `(l, m)`, then one can use a block of layout `k` as if it has any
    /// size in the range `[l, m]` (inclusive).
    ///
    /// (All implementors of `fn usable_size` must ensure that
    /// `l <= k.size() <= m`)
    ///
    /// Both the lower- and upper-bounds (`l` and `m` respectively) are
    /// provided: An allocator based on size classes could misbehave
    /// if one attempts to deallocate a block without providing a
    /// correct value for its size (i.e., one within the range `[l, m]`).
    ///
    /// Clients who wish to make use of excess capacity are encouraged
    /// to use the `alloc_excess` and `realloc_excess` instead, as
    /// this method is constrained to conservatively report a value
    /// less than or equal to the minimum capacity for *all possible*
    /// calls to those methods.
    ///
    /// However, for clients that do not wish to track the capacity
    /// returned by `alloc_excess` locally, this method is likely to
    /// produce useful results.
    unsafe fn usable_size(&self, layout: &Layout) -> (Capacity, Capacity) {
        (layout.size(), layout.size())
    }

    // == METHODS FOR MEMORY REUSE ==
   // realloc. alloc_excess, realloc_excess

   /// Returns a pointer suitable for holding data described by
   /// `new_layout`, meeting its size and alignment guarantees. To
   /// accomplish this, this may extend or shrink the allocation
   /// referenced by `ptr` to fit `new_layout`.
   ///
   /// * `ptr` must have previously been provided via this allocator.
   ///
   /// * `layout` must *fit* the `ptr` (see above). (The `new_layout`
   ///   argument need not fit it.)
   ///
   /// Behavior undefined if either of latter two constraints are unmet.
   ///
   /// In addition, `new_layout` should not impose a different alignment
   /// constraint than `layout`. (In other words, `new_layout.align()`
   /// should equal `layout.align()`.)
   /// However, behavior is well-defined (though underspecified) when
   /// this constraint is violated; further discussion below.
   ///
   /// If this returns `Ok`, then ownership of the memory block
   /// referenced by `ptr` has been transferred to this
   /// allocator. The memory may or may not have been freed, and
   /// should be considered unusable (unless of course it was
   /// transferred back to the caller again via the return value of
   /// this method).
   ///
   /// Returns `Err` only if `new_layout` does not meet the allocator's
   /// size and alignment constraints of the allocator or the
   /// alignment of `layout`, or if reallocation otherwise fails. (Note
   /// that did not say "if and only if" -- in particular, an
   /// implementation of this method *can* return `Ok` if
   /// `new_layout.align() != old_layout.align()`; or it can return `Err`
   /// in that scenario, depending on whether this allocator
   /// can dynamically adjust the alignment constraint for the block.)
   ///
   /// If this method returns `Err`, then ownership of the memory
   /// block has not been transferred to this allocator, and the
   /// contents of the memory block are unaltered.
   unsafe fn realloc(&mut self,
                     ptr: Address,
                     layout: Layout,
                     new_layout: Layout) -> Result<Address, AllocErr> {
       let (min, max) = self.usable_size(&layout);
       let s = new_layout.size();
       // All Layout alignments are powers of two, so a comparison
       // suffices here (rather than resorting to a `%` operation).
       if min <= s && s <= max && new_layout.align() <= layout.align() {
           return Ok(ptr);
       } else {
           let new_size = new_layout.size();
           let old_size = layout.size();
           let result = self.alloc(new_layout);
           if let Ok(new_ptr) = result {
               ptr::copy(ptr as *const u8, new_ptr, cmp::min(old_size, new_size));
               self.dealloc(ptr, layout);
           }
           result
       }
   }

   /// Behaves like `fn alloc`, but also returns the whole size of
   /// the returned block. For some `layout` inputs, like arrays, this
   /// may include extra storage usable for additional data.
   unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
       let usable_size = self.usable_size(&layout);
       self.alloc(layout).map(|p| Excess(p, usable_size.1))
   }

   /// Behaves like `fn realloc`, but also returns the whole size of
   /// the returned block. For some `layout` inputs, like arrays, this
   /// may include extra storage usable for additional data.
   unsafe fn realloc_excess(&mut self,
                            ptr: Address,
                            layout: Layout,
                            new_layout: Layout) -> Result<Excess, AllocErr> {
       let usable_size = self.usable_size(&new_layout);
       self.realloc(ptr, layout, new_layout)
           .map(|p| Excess(p, usable_size.1))
   }

   /// Attempts to extend the allocation referenced by `ptr` to fit `new_layout`.
   ///
   /// * `ptr` must have previously been provided via this allocator.
   ///
   /// * `layout` must *fit* the `ptr` (see above). (The `new_layout`
   ///   argument need not fit it.)
   ///
   /// Behavior undefined if either of latter two constraints are unmet.
   ///
   /// If this returns `Ok`, then the allocator has asserted that the
   /// memory block referenced by `ptr` now fits `new_layout`, and thus can
   /// be used to carry data of that layout. (The allocator is allowed to
   /// expend effort to accomplish this, such as extending the memory block to
   /// include successor blocks, or virtual memory tricks.)
   ///
   /// If this returns `Err`, then the allocator has made no assertion
   /// about whether the memory block referenced by `ptr` can or cannot
   /// fit `new_layout`.
   ///
   /// In either case, ownership of the memory block referenced by `ptr`
   /// has not been transferred, and the contents of the memory block
   /// are unaltered.
   unsafe fn realloc_in_place(&mut self,
                              ptr: Address,
                              layout: Layout,
                              new_layout: Layout) -> Result<(), CannotReallocInPlace> {
       let (_, _, _) = (ptr, layout, new_layout);
       Err(CannotReallocInPlace)
   }

    // == COMMON USAGE PATTERNS ==
    // alloc_one, dealloc_one, alloc_array, realloc_array. dealloc_array

    /// Allocates a block suitable for holding an instance of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// The returned block is suitable for passing to the
    /// `alloc`/`realloc` methods of this allocator.
    ///
    /// May return `Err` for zero-sized `T`.
    unsafe fn alloc_one<T>(&mut self) -> Result<Unique<T>, AllocErr>
        where Self: Sized {
        let k = Layout::new::<T>();
        if k.size() > 0 {
            self.alloc(k).map(|p|Unique::new(*p as *mut T))
        } else {
            Err(AllocErr::invalid_input("zero-sized type invalid for alloc_one"))
        }
    }

    /// Deallocates a block suitable for holding an instance of `T`.
    ///
    /// The given block must have been produced by this allocator,
    /// and must be suitable for storing a `T` (in terms of alignment
    /// as well as minimum and maximum size); otherwise yields
    /// undefined behavior.
    ///
    /// Captures a common usage pattern for allocators.
    unsafe fn dealloc_one<T>(&mut self, mut ptr: Unique<T>)
        where Self: Sized {
        let raw_ptr = ptr.as_mut() as *mut T as *mut u8;
        self.dealloc(raw_ptr, Layout::new::<T>());
    }

    /// Allocates a block suitable for holding `n` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// The returned block is suitable for passing to the
    /// `alloc`/`realloc` methods of this allocator.
    ///
    /// May return `Err` for zero-sized `T` or `n == 0`.
    ///
    /// Always returns `Err` on arithmetic overflow.
    unsafe fn alloc_array<T>(&mut self, n: usize) -> Result<Unique<T>, AllocErr>
        where Self: Sized {
        match Layout::array::<T>(n) {
            Some(ref layout) if layout.size() > 0 => {
                self.alloc(layout.clone())
                    .map(|p| {
                        // println!("alloc_array layout: {:?} yielded p: {:?}", layout, p);
                        Unique::new(p as *mut T)
                    })
            }
            _ => Err(AllocErr::invalid_input("invalid layout for alloc_array")),
        }
    }

    /// Reallocates a block previously suitable for holding `n_old`
    /// instances of `T`, returning a block suitable for holding
    /// `n_new` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// The returned block is suitable for passing to the
    /// `alloc`/`realloc` methods of this allocator.
    ///
    /// May return `Err` for zero-sized `T` or `n == 0`.
    ///
    /// Always returns `Err` on arithmetic overflow.
    unsafe fn realloc_array<T>(&mut self,
                               ptr: Unique<T>,
                               n_old: usize,
                               n_new: usize) -> Result<Unique<T>, AllocErr>
        where Self: Sized {
        match ( Layout::array::<T>(n_old)
              , Layout::array::<T>(n_new)
              , ptr.as_ptr()) {
            (Some(ref k_old), Some(ref k_new), ptr) if k_old.size() > 0 && k_new.size() > 0 => {
                self.realloc(ptr as *mut u8, k_old.clone(), k_new.clone())
                    .map(|p|Unique::new(p as *mut T))
            }
            _ => {
                Err(AllocErr::invalid_input("invalid layout for realloc_array"))
            }
        }
    }

    /// Deallocates a block suitable for holding `n` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    unsafe fn dealloc_array<T>(&mut self, ptr: Unique<T>, n: usize) -> Result<(), AllocErr>
        where Self: Sized {
        let raw_ptr = ptr.as_ptr() as *mut u8;
        match Layout::array::<T>(n) {
            Some(ref k) if k.size() > 0 => {
                Ok(self.dealloc(raw_ptr, k.clone()))
            }
            _ => {
                Err(AllocErr::invalid_input("invalid layout for dealloc_array"))
            }
        }
    }

    // UNCHECKED METHOD VARIANTS

    /// Returns a pointer suitable for holding data described by
    /// `layout`, meeting its size and alignment guarantees.
    ///
    /// The returned block of storage may or may not have its contents
    /// initialized. (Extension subtraits might restrict this
    /// behavior, e.g. to ensure initialization.)
    ///
    /// Returns `None` if request unsatisfied.
    ///
    /// Behavior undefined if input does not meet size or alignment
    /// constraints of this allocator.
    unsafe fn alloc_unchecked(&mut self, layout: Layout) -> Option<Address> {
        // (default implementation carries checks, but impl's are free to omit them.)
        self.alloc(layout).ok()
    }

    /// Returns a pointer suitable for holding data described by
    /// `new_layout`, meeting its size and alignment guarantees. To
    /// accomplish this, may extend or shrink the allocation
    /// referenced by `ptr` to fit `new_layout`.
    ////
    /// (In other words, ownership of the memory block associated with
    /// `ptr` is first transferred back to this allocator, but the
    /// same block may or may not be transferred back as the result of
    /// this call.)
    ///
    /// * `ptr` must have previously been provided via this allocator.
    ///
    /// * `layout` must *fit* the `ptr` (see above). (The `new_layout`
    ///   argument need not fit it.)
    ///
    /// * `new_layout` must meet the allocator's size and alignment
    ///    constraints. In addition, `new_layout.align()` must equal
    ///    `layout.align()`. (Note that this is a stronger constraint
    ///    that that imposed by `fn realloc`.)
    ///
    /// Behavior undefined if any of latter three constraints are unmet.
    ///
    /// If this returns `Some`, then the memory block referenced by
    /// `ptr` may have been freed and should be considered unusable.
    ///
    /// Returns `None` if reallocation fails; in this scenario, the
    /// original memory block referenced by `ptr` is unaltered.
    unsafe fn realloc_unchecked(&mut self,
                                ptr: Address,
                                layout: Layout,
                                new_layout: Layout) -> Option<Address> {
        // (default implementation carries checks, but impl's are free to omit them.)
        self.realloc(ptr, layout, new_layout).ok()
    }

    /// Behaves like `fn alloc_unchecked`, but also returns the whole
    /// size of the returned block.
    unsafe fn alloc_excess_unchecked(&mut self, layout: Layout) -> Option<Excess> {
        self.alloc_excess(layout).ok()
    }

    /// Behaves like `fn realloc_unchecked`, but also returns the
    /// whole size of the returned block.
    unsafe fn realloc_excess_unchecked(&mut self,
                                       ptr: Address,
                                       layout: Layout,
                                       new_layout: Layout) -> Option<Excess> {
        self.realloc_excess(ptr, layout, new_layout).ok()
    }


    /// Allocates a block suitable for holding `n` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// Requires inputs are non-zero and do not cause arithmetic
    /// overflow, and `T` is not zero sized; otherwise yields
    /// undefined behavior.
    unsafe fn alloc_array_unchecked<T>(&mut self, n: usize) -> Option<Unique<T>>
        where Self: Sized {
        let layout = Layout::array_unchecked::<T>(n);
        self.alloc_unchecked(layout).map(|p|Unique::new(*p as *mut T))
    }

    /// Reallocates a block suitable for holding `n_old` instances of `T`,
    /// returning a block suitable for holding `n_new` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// Requires inputs are non-zero and do not cause arithmetic
    /// overflow, and `T` is not zero sized; otherwise yields
    /// undefined behavior.
    unsafe fn realloc_array_unchecked<T>(&mut self,
                                         ptr: Unique<T>,
                                         n_old: usize,
                                         n_new: usize) -> Option<Unique<T>>
        where Self: Sized {
        let (k_old, k_new, ptr) = ( Layout::array_unchecked::<T>(n_old)
                                  , Layout::array_unchecked::<T>(n_new)
                                  , ptr.as_ptr());
        self.realloc_unchecked(ptr as *mut u8, k_old, k_new)
            .map(|p|Unique::new(*p as *mut T))
    }

    /// Deallocates a block suitable for holding `n` instances of `T`.
    ///
    /// Captures a common usage pattern for allocators.
    ///
    /// Requires inputs are non-zero and do not cause arithmetic
    /// overflow, and `T` is not zero sized; otherwise yields
    /// undefined behavior.
    unsafe fn dealloc_array_unchecked<T>(&mut self, ptr: Unique<T>, n: usize)
        where Self: Sized {
        let layout = Layout::array_unchecked::<T>(n);
        self.dealloc(ptr.as_ptr() as *mut u8, layout);
    }
}


#[cfg(feature = "borrow")] pub mod borrow;

#[cfg(feature = "buddy")]
pub mod buddy;
#[cfg(feature = "first_fit")]
pub mod first_fit;
#[cfg(feature = "bump_ptr")]
pub mod bump_ptr;

#[cfg(feature = "system")] pub mod system;
#[cfg(feature = "system")] pub use system::*;

#[cfg(feature = "placement_in")] pub mod place;
#[cfg(feature = "placement_in")] pub use place::*;
