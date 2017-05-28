//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
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

use core::{ ops, mem, slice, convert };
use core::convert::TryFrom;

use memory::{ FrameRange, PhysicalPage };


macro_rules! impl_getters {
    ($(#[$attr:meta])* pub fn $name:ident(&self) -> PAddr; $($rest:tt)*) => {
        $(#[$attr])* #[inline] pub fn $name(&self) -> ::memory::PAddr {
            use ::memory::{PAddr, Addr};
            PAddr::from(self.$name as <PAddr as Addr>::Repr)
        }
        impl_getters!{ $( $rest )* }
    };
    ($(#[$attr:meta])* fn $name:ident(&self) -> PAddr; $($rest:tt)*) => {
        $(#[$attr])* #[inline] fn $name(&self) -> ::memory::PAddr {
            use ::memory::{PAddr, Addr};
            PAddr::from(self.$name as <PAddr as Addr>::Repr)
        }
        impl_getters!{ $( $rest )* }
    };
    ($(#[$attr:meta])* pub fn $name:ident(&self) -> PAddr;) => {
        $(#[$attr])* #[inline] pub fn $name(&self) -> ::memory::PAddr {
            use ::memory::{PAddr, Addr};
            PAddr::from(self.$name as <PAddr as Addr>::Repr)
        }
        impl_getters!{ $( $rest )* }
    };
    ($(#[$attr:meta])* fn $name:ident(&self) -> PAddr;) => {
        $(#[$attr])* #[inline] fn $name(&self) -> ::memory::PAddr {
            use ::memory::{PAddr, Addr};
            PAddr::from(self.$name as <PAddr as Addr>::Repr)
        }
        impl_getters!{ $( $rest )* }
    };
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
pub type Section<W> = section::Header<Word = W>;
pub type ProgramHeader<W> = program::Header<Word = W>;
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

#[cfg(target_pointer_width = "32")]
type DefaultWord = u32;
#[cfg(target_pointer_width = "64")]
type DefaultWord = u64;

/// Hack to make the type-system let me do what I want
trait ValidatesWord<Word: ElfWord> {
    fn check(&self) -> ElfResult<()>;
}

/// A handle on a parsed ELF binary
///  TODO: do we want this to own a HashMap of section names to section headers,
///        to speed up section lookup?
//          - eliza, 03/08/2017
#[derive(Debug)]
pub struct Image< 'bytes             // lifetime of the byte slice
                , Word = DefaultWord // default to machine's pointer size
                , ProgHeader = ProgramHeader<Word> // same word type
                , SectHeader = Section<Word>
                , Header = FileHeader<Word> // must have same word type
                > // jesus christ
where Word: ElfWord + 'bytes
    , ProgHeader: program::Header<Word = Word> + Sized + 'bytes
    , SectHeader: section::Header<Word = Word> + Sized + 'bytes
    , Header: file::Header<Word = Word> + 'bytes
    {
    /// the binary's [file header](file/trait.Header.html)
    pub header: &'bytes Header
  , /// references to each [section header](section/struct.Header.html)
    pub sections: &'bytes [SectHeader]
  , /// references to each [program header](program/trait.Header.html)
    pub program_headers: &'bytes [ProgHeader]
  , /// the raw binary contents of the ELF binary.
    /// note that this includes the _entire_ binary contents of the file,
    /// so the file header and each section header is included in this slice.
    binary: &'bytes [u8]
}

impl <'a, Word, ProgHeader, SectHeader, Header>
Image<'a, Word, ProgHeader, SectHeader, Header>
where Word: ElfWord + 'a
    , ProgHeader: program::Header<Word = Word> + Sized + 'a
    , SectHeader: section::Header<Word = Word> + Sized + 'a
    , Header: file::Header<Word = Word> + 'a
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

impl<'a, Word, PH, SH, FH> TryFrom<&'a [u8]> for Image<'a, Word, PH, SH, FH>
where Word: ElfWord + 'a
    , PH: program::Header<Word = Word> + 'a
    , SH: section::Header<Word = Word> + 'a
    , FH: file::Header<Word = Word> + 'a
    , &'a FH: convert::TryFrom<&'a [u8], Error = &'static str>
    {

    type Error = &'static str;

    fn try_from(bytes: &'a [u8]) -> ElfResult<Self> {
        let header: &'a FH = <&'a FH>::try_from(bytes)?;

        let sections = unsafe { extract_from_slice::<SH>(
            &bytes[header.sh_range()]
          , 0
          , header.sh_count()
        )? };
        let prog_headers = unsafe { extract_from_slice::<PH>(
            &bytes[header.ph_range()]
          , 0
          , header.ph_count()
        )? };
        Ok(Image { header: header
              , sections: sections
              , program_headers: prog_headers
              , binary: bytes
        })
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
/// + If the index `offset` is longer than `T`
///
/// TODO: rewrite this as a `TryFrom` implementation (see issue #85)
//          - eliza, 03/09/2017
///       wait, possibly we should NOT do that. actually we should
///       almost certainly not do that. since this function is unsafe,
///       but `TryFrom` is not, and because this would be WAY generic.
//          - eliza, 03/09/2017
/// TODO: is this general enough to move into util?
//          - eliza, 03/09/2017
/// TODO: refactor this to take a `RangeArgument`?
//          - eliza, 03/13/2017
///       or, we could just remove the offset and expect the caller to
///       offset the slice?
//          - eliza, 03/14/2017
///
/// [`slice::from_raw_parts`]: https://doc.rust-lang.org/stable/std/slice/fn.from_raw_parts.html
unsafe fn extract_from_slice<'slice, T: Sized>( data: &'slice [u8]
                                              , offset: usize
                                              , n: usize)
                                              -> ElfResult<&'slice [T]> {
    if offset % mem::align_of::<T>() != 0 {
        // TODO: these error messages don't contain as much information as they
        //       used to, since the return type is `&'static str` that can't be
        //       dynamically formatted as the panic was. refactor this?
        //       (e.g. should ELF get its own error type?)
        //        - eliza, 03/15/2017
        Err("extract_from_slice: Offset not aligned on type T sized boundary!")
        // assert!(
        //        , "Offset {} not aligned on a {}-sized boundary (must be \
        //           divisible by {})."
        //        , offset, type_name::<T>(), mem::align_of::<T>()
        //        );
    } else if data.len() - offset < mem::size_of::<T>() * n {
        Err("extract_from_slice: Slice too short to contain n instances of T!")
        // assert!(
        //        , "Slice too short to contain {} objects of type {}"
        //        , n, type_name::<T>()
        //        );
    } else {
        Ok(slice::from_raw_parts(data[offset..].as_ptr() as *const T, n))
    }
}

impl<'a, W: ElfWord> convert::Into<FrameRange> for &'a Section<W> {
    #[inline]
    fn into(self) -> FrameRange {
        use memory::Page;
        let start = PhysicalPage::containing(self.address());
        let end = PhysicalPage::containing(self.end_address());
        start .. end
    }
}
