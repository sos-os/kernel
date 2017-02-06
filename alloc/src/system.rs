use spin::{Mutex, MutexGuard};
use super::{Address, Allocator, AllocErr, Layout};

extern crate params;

use self::params::InitParams;

use core::ops::{Deref, DerefMut};
use core::ptr::Unique;
#[cfg(feature = "bump_ptr")]
use bump_ptr::BumpPtr;

#[cfg(feature = "buddy")]
use buddy::Heap as BuddyHeap;
#[cfg(all(feature = "bump_ptr", feature="buddy"))]
pub enum Tier<'a> {
    Uninitialized
    , Bump(BumpPtr)
    , Buddy(BuddyHeap<'a>)
}
#[cfg(all(feature = "bump_ptr", feature="buddy"))]
impl Deref for Tier<'static> {
    type Target = Allocator + 'static ;
    fn deref(&self) -> &Self::Target{
        match self {
            &Tier::Bump(ref alloc) => alloc
          , &Tier::Buddy(ref alloc) => alloc
          , _ => panic!("no allocator!")
        }
    }
}

#[cfg(all(feature = "bump_ptr", feature="buddy"))]
unsafe impl<'a> Allocator for Tier<'a> {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<Address, AllocErr> {
        match *self {
            Tier::Bump(ref mut alloc) => alloc.alloc(layout)
          , Tier::Buddy(ref mut alloc) => alloc.alloc(layout)
          , _ => Err(AllocErr::Unsupported {
                    details: "System allocator uninitialized!"
                })
        }
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, ptr: Address, layout: Layout) {
        match *self {
            Tier::Bump(ref mut alloc) => alloc.dealloc(ptr, layout)
          , Tier::Buddy(ref mut alloc) => alloc.dealloc(ptr, layout)
          , _ =>  {
              // just leak it? not sure if we should panic here...
          }
        }
    }

}

pub type SystemAllocator = Mutex<Tier<'static>>;

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
// TODO: can this allocate pointers to _objects_ rather than `*mut u8`s?
//       - eliza, 1/23/2017
pub struct BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    ptr: Unique<u8>
  , layout: Layout
  , allocator: &'a Mutex<A>
}

impl<'a, A> Deref for BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    type Target = *mut u8;
    fn deref(&self) ->  &Self::Target { &(*self.ptr) }
}
//
// impl<'a, A> ops::DerefMut for BorrowedPtr<'a, A>
// where A: Allocator
//     , A: 'a {
//     fn deref_mut(&mut self) ->  &mut Self::Target { &mut self.frame }
// }

impl<'a, A> Drop for BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock().dealloc(*self.ptr, self.layout.clone())
        }
    }
}

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
pub struct Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    value: Unique<T>
  , allocator: &'a Mutex<A>
}

impl<'a, A, T> Deref for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    type Target = T;
    fn deref(&self) ->  &Self::Target { unsafe { self.value.get() } }
}

impl<'a, A, T> DerefMut for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    fn deref_mut(&mut self) ->  &mut Self::Target {
        unsafe { self.value.get_mut() }
    }
}

impl<'a, A, T> Drop for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock()
                .dealloc( *self.value as *mut u8
                        , Layout::for_value(self.value.get()))
        }
    }
}
