//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Parsing and loading Executable and Linkable Format (ELF) 32- and 64-bit
//! binaries.
//!
//! For more information on the ELF format, refer to:
//!
//!  + [Wikipedia](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
//!  + The [OS Dev Wiki](http://wiki.osdev.org/ELF)
//!  + The [ELF Format Specification](elfspec)
//!
//! [elfspec]: http://www.skyfree.org/linux/references/ELF_Format.pdf
#![feature(core_intrinsics)]
#![feature(try_from)]
#![no_std]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate macro_attr;

extern crate memory;

use core::{ ops, mem, slice };

macro_rules! impl_getters {
    ($(#[$attr:meta])* pub fn $name:ident(&self) -> $ty:ty; $($rest:tt)*) => {
        $(#[$attr])* #[inline] pub fn $name(&self) -> $ty { self.$name as $ty }
        impl_getters!{ $( $rest )* }
    };
    ($(#[$attr:meta])* fn $name:ident(&self) -> $ty:ty; $($rest:tt)*) => {
        $(#[$attr])* #[inline] fn $name(&self) -> $ty { self.$name as $ty }
        impl_getters!{ $( $rest )* }
    };
    ( $(#[$attr:meta])* pub fn $name: ident (&self)-> $ty:ty; ) => {
        $(#[$attr])* #[inline] pub fn $name(&self) -> $ty { self.$name as $ty }
    };
    ( $(#[$attr:meta])* fn $name: ident (&self)-> $ty:ty; ) => {
        $(#[$attr])* #[inline] fn $name(&self) -> $ty { self.$name as $ty }
    };
    () => {};
}

pub mod section;
pub mod file;
pub mod program;

/// An ELF section header.
pub type Section<'a> = section::Header<'a>;
/// An ELF header file.
pub type FileHeader<W> = file::HeaderRepr<W>;

/// TODO: should ELF have its own error type?
pub type ElfResult<T> = Result<T, &'static str>;

pub trait ElfWord: Sized + Copy + Clone
                         + ops::Add<Self> + ops::Sub<Self>
                         + ops::Mul<Self> + ops::Div<Self>
                         + ops::Shl<Self> + ops::Shr<Self> { }
impl ElfWord for u64 { }
impl ElfWord for u32 { }

/// Hack to make the type-system let me do what I want
trait ValidatesWord<Word: ElfWord> {
    fn check(&self) -> ElfResult<()>;
}

/// A handle on a parsed ELF binary
///  TODO: do we want this to own a HashMap of section names to section headers,
///        to speed up section lookup?
//          - eliza, 03/08/2017
#[derive(Debug)]
pub struct Image<'a, Word, ProgHeader, Header = file::HeaderRepr<Word>>
where Word: ElfWord + 'a
    , Header: file::Header<Word = Word> + 'a
    , ProgHeader: program::Header<Word = Word> + 'a
    {
    /// the binary's [file header](file/trait.Header.html)
    pub header: &'a Header
  , /// references to each [section header](section/struct.Header.html)
    pub sections: &'a [section::Header<'a>]
  , /// references to each [program header](program/trait.Header.html)
    pub program_headers: &'a [ProgHeader]
  , /// the raw binary contents of the ELF binary.
    /// note that this includes the _entire_ binary contents of the file,
    /// so the file header and each section header is included in this slice.
    binary: &'a [u8]
}

impl<'a, Word, ProgHeader, Header> Image<'a, Word, ProgHeader, Header>
where Word: ElfWord + 'a
    , Header: file::Header<Word = Word> + 'a
    , ProgHeader: program::Header<Word = Word> + 'a
    {
    /// Returns the section header [string table].
    ///
    /// [string table]: section/struct.StrTable.html
    pub fn sh_str_table(&'a self) -> section::StrTable<'a> {
        // TODO: do we want to validate that the string table index is
        //       reasonable (e.g. it's not longer than the binary)?
        //          - eliza, 03/08/2017
        // TODO: do we want to cache a ref to the string table?
        //          - eliza, 03/08/2017
        section::StrTable::from(&self.binary[self.header.sh_str_idx()..])
    }

}

/// Extract `n` instances of type `T` from a byte slice.
///
/// This is essentially just a _slightly_ safer wrapper around
/// [`slice::from_raw_parts`]. Unlike `from_raw_parts`, this function takes
/// a valid byte slice, rather than a pointer. Therefore, some of the safety
/// issues with `from_raw_parts` are avoided:
///
/// + the lifetime (`'slice`) of the returned slice should be the same as the
///   lifetime of the input slice (`data`), rather than inferred arbitrarily.
/// + this function will panic rather than reading past the end of the slice.
///
/// # Arguments
///
/// + `data`: the byte slice to extract a slice of `&[T]`s from
/// + `offset`: a start offset into `data`
/// + `n`: the number of instances of `T` which should be contained
///        in `data[offset..]`
///
/// # Safety
///
/// While this function is safer than [`slice::from_raw_parts`],
/// it is still unsafe for the following reasons:
///
/// + The contents of `data` may not be able to be interpreted as instances of
///   type `T`.
///
/// # Caveats
///
/// + If `n` == 0, this will give you an `&[]`. Just a warning.
//    thanks to Max for making  me figure this out.
/// + `offset` must be aligned on a `T`-sized boundary.
///
/// # Panics
///
/// + If the slice `data` is not long enough to contain `n` instances of `T`.
/// + If the index `offset` is longer than `T`
/// + If `offset` is not aligned on a `T`-sized boundary.
///
/// TODO: rewrite this as a `TryFrom` implementation (see issue #85)
//          - eliza, 03/09/2017
///       wait, possibly we should NOT do that. actually we should
///       almost certainly not do that. since this function is unsafe,
///       but `TryFrom` is not, and because this would be WAY generic.
//          - eliza, 03/09/2017
/// TODO: is this general enough to move into util?
//          - eliza, 03/09/2017
/// TODO: should this be refactored to return a `Result`?
//          - eliza, 03/13/2017
/// TODO: assert that `offset` is aligned on a `T`-sized boundary
//          - eliza, 03/13/2017
/// TODO: refactor this to take a `RangeArgument`?
//          - eliza, 03/13/2017
///
/// [`slice::from_raw_parts`]: https://doc.rust-lang.org/stable/std/slice/fn.from_raw_parts.html
unsafe fn extract_from_slice<'slice, T: Sized>( data: &'slice [u8]
                                              , offset: usize
                                              , n: usize)
                                              -> &'slice [T] {
    use core::intrinsics::type_name;
    assert!( offset % mem::align_of::<T>() == 0
           , "Offset {} not aligned on a {}-sized boundary (must be \
              divisible by {})."
           , offset, type_name::<T>(), mem::align_of::<T>()
           );
    assert!( data.len() - offset >= mem::size_of::<T>() * n
           , "Slice too short to contain {} objects of type {}"
           , n, type_name::<T>()
           );
    slice::from_raw_parts(data[offset..].as_ptr() as *const T, n)
}
