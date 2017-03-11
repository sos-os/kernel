//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
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
