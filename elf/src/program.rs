//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use memory::{PAddr, VAddr};
use super::ElfWord;

/// Trait representing an ELF Program Header.
///
/// This trait allows [HeaderRepr] to provide a consistent API regardless
/// of whether the header section uses 32- or 64-bit [ELF word]s. A number of
/// field values in the header of various sizes are converted to `usize` by
/// this API so that they can be used as indices, etc.
///
/// For more information on ELF Program Headers, refer to:
/// + the ELF [specification]
/// + the [OS Dev Wiki]
///
/// [ELF word]: ../type.ElfWord.html
/// [HeaderRepr]: struct.HeaderRepr.html
/// [specification](http://www.sco.com/developers/gabi/latest/ch5.pheader.html)
/// [OS Dev Wiki](http://wiki.osdev.org/ELF#Header)
pub trait Header: Sized {
    type Word: ElfWord;

    /// Returns the [type](enum.Type.html) of this program header.
    ///
    /// This member tells what kind of segment this header describes or how to
    /// interpret the array element's information.
    fn ty(&self) -> Type;

    /// Returns this segment's start offset from the beginning of the binary.
    fn offset(&self) -> usize;

    /// Returns the virtual address of the first byte in this segment.
    fn vaddr(&self) -> VAddr;

    /// Returns the physical address of the first byte in this segment.
    fn paddr(&self) -> PAddr;

    /// Returns the number of bytes in the file image of the segment.
    ///
    /// This may be zero.
    fn file_size(&self) -> usize;

    /// Returns the number of bytes in the memory image of the segment.
    ///
    /// This may be zero.
    fn mem_size(&self) -> usize;

    /// Returns the [flags] for this segment.
    ///
    /// [flags]: struct.Flags.html
    fn flags(&self) -> Flags;

    fn align(&self) -> usize;

}

/// The type field of an ELF program header
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Type {
    /// `PT_NULL`: Program header table entry unused
    Null = 0
  , /// `PT_LOAD`: Loadable program segment
    Loadable = 1
  , /// `PT_DYNAMIC`: Dynamic linking information
    Dynamic = 2
  , /// `PT_INTERP`: Program interpreter
    Interpreter = 3
  , /// `PT_NOTE`: Auxiliary information
    Note = 4
  , /// `PT_SHLIB`
    ShLib = 5
  , /// `PT_PHDR`: Program Header table
    HeaderTable = 6
  , /// `PT_TLS`: Thread-local storage
    ThreadLocal = 7
  , /// GCC `.eh_frame_hdr` segment
    GnuEhFrame = 0x6474e550
  , /// Indicates stack executability
    GnuStack = 0x6474e551
  , /// Read-only after relocation
    GnuRelRo = 0x6474e552
}

bitflags! {
    pub flags Flags: u32 {
        const NONE = 0
      , const EXECUTABLE = 1 << 0
      , const WRITABLE = 1 << 1
      , const READABLE = 1 << 2
    }
}

/// An ELF Program Header
#[derive(Copy, Clone, Debug)]
pub struct HeaderRepr64 {
    pub ty: Type
  , pub flags: Flags
  ,
}
