//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
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
#![no_std]

#[macro_use] extern crate bitflags;

extern crate memory;

use core::{ intrinsics, ops, mem, slice };

pub mod section;
pub mod file;
pub mod program;

/// An ELF section header.
pub type Section<'a> = section::Header<'a>;
/// An ELF header file.
pub type FileHeader<W> = file::Header<W>;

pub type ElfResult<T> = Result<T, &'static str>;

pub trait ElfWord: Sized + Copy + Clone
                         + ops::Add<Self> + ops::Sub<Self>
                         + ops::Mul<Self> + ops::Div<Self>
                         + ops::Shl<Self> + ops::Shr<Self> { }
impl ElfWord for u64 { }
impl ElfWord for u32 { }

/// A handle on a parsed ELF binary
///  TODO: do we want this to own a HashMap of section names to section headers,
///        to speed up section lookup?
//          - eliza, 03/08/2017
#[derive(Debug)]
pub struct Image<'a, Word>
where Word: ElfWord + 'a {
    /// the binary's [file header](file/struct.Header.html)
    pub header: &'a file::Header<Word>
  , /// references to each [section header](section/enum.Header.html)
    pub sections: &'a [section::Header<'a>]
  , /// the raw binary contents of the ELF binary.
    /// note that this includes the _entire_ binary contents of the file,
    /// so the file header and each section header is included in this slice.
    binary: &'a [u8]
}

impl<'a, Word: ElfWord + 'a> Image<'a, Word> {
    /// Returns the section header [string table].
    ///
    /// [string table]: section/struct.StrTable.html
    pub fn sh_str_table(&'a self) -> section::StrTable<'a> {
        // TODO: do we want to validate that the string table index is
        //       reasonable (e.g. it's not longer than the binary)?
        //          - eliza, 03/08/2017
        let offset = self.header.sh_str_idx as usize;
        // TODO: do we want to cache a ref to the string table?
        //          - eliza, 03/08/2017
        section::StrTable::from(&self.binary[offset..])
    }

}

/// if `n` == 0, this will give you an `&[]`. just a warning.
//  thanks to Max for making  me figure this out.
unsafe fn extract_from_slice<T: Sized>( data: &[u8]
                                      , offset: usize
                                      , n: usize)
                                      -> &[T] {
    assert!( data.len() - offset >= mem::size_of::<T>() * n
           , "Slice too short to contain {} objects of type {}"
           , n
           , intrinsics::type_name::<T>()
           );
    slice::from_raw_parts(data[offset..].as_ptr() as *const T, n)
}
