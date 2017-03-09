//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use super::{ElfResult, ElfWord, Section, section};

use core::{fmt, mem};

/// An ELF file header
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Header<W: ElfWord> {
    pub ident: Ident
  , elftype: TypeRepr
  , pub machine: Machine
  , /// Program entry point
    pub entry_point: W
  , /// Offset for start of program headers
    pub ph_offset: W
  , /// Offset for start of section headers
    pub sh_offset: W
  , pub flags: u32
  , pub header_size: u16
  , pub ph_entry_size: u16
  , pub ph_count: u16
  , pub sh_entry_size: u16
  , pub sh_count: u16
  , /// Index of the section header string table
    pub sh_str_idx: u16
}

impl<W> Header<W> where W: ElfWord {

    /// Attempt to extract an ELF file header from a slice of bytes.
    pub fn from_slice<'a>(input: &'a [u8]) -> ElfResult<&'a Header<W>> {
        if input.len() < mem::size_of::<Header<W>>() {
            Err("Input too short to extract ELF header")
        } else {
            unsafe { Ok(&super::extract_from_slice::<Header<W>>(input, 0, 1)[0]) }
        }
    }

    #[inline]
    pub fn get_type(&self) -> Type { self.elftype.as_type() }


}

impl Header<u64> {

    /// Attempt to extract a section header from a slice of bytes.
    pub fn parse_section<'a>(&'a self, input: &'a [u8], idx: u16)
                            -> ElfResult<&'a Section>
    {
        if idx < section::SHN_LORESERVE {
            Err("Cannot parse reserved section.")
        } else {
            let start: u64// start offset for section
                = self.sh_offset + idx as u64 * self.sh_entry_size as u64;
            let end: u64 // end offset for section
                = start + self.sh_entry_size as u64;
            let raw
                = &input[start as usize .. end as usize];

            match self.ident.class {
                Class::None => Err("Invalid ELF class (ELFCLASSNONE).")
              , Class::Elf32 => Err("Cannot parse 32-bit section from 64-bit \
                                     ELF file.")
              , Class::Elf64 => unsafe {
                    Ok(&*(raw as *const [u8] as *const u8 as *const Section))
                }
            }
        }
    }

}

impl Header<u32> {

    /// Attempt to extract a section header from a slice of bytes.
    pub fn parse_section<'a>(&'a self, input: &'a [u8], idx: u16)
                            -> ElfResult<&'a Section>
    {
        if idx < section::SHN_LORESERVE {
            Err("Cannot parse reserved section.")
        } else {
            let start: u32// start offset for section
                = self.sh_offset + idx as u32 * self.sh_entry_size as u32;
            let end: u32 // end offset for section
                = start + self.sh_entry_size as u32;
            let raw
                = &input[start as usize .. end as usize];

            match self.ident.class {
                Class::None => Err("Invalid ELF class (ELFCLASSNONE).")
              , Class::Elf32 => unsafe {
                    Ok(&*(raw as *const [u8] as *const u8 as *const Section))
                }
              , Class::Elf64 => Err("Cannot parse 64-bit section from 32-bit \
                                     ELF file.")
            }
        }
    }

}

/// ELF header magic
pub const MAGIC: Magic = [0x7f, b'E', b'L', b'F'];

/// Type of header magic
pub type Magic = [u8; 4];


/// ELF identifier (`e_ident` in the ELF standard)
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Ident {
    /// ELF magic numbers. Must be [0x7, E, L, F]
    pub magic: Magic
  , /// ELF file class (32- or 64-bit)
    pub class: Class
  , /// ELF data encoding (big- or little-endian)
    pub data: DataEncoding
  , /// ELF file version
    pub version: Version
  , pub abi: OsAbi
  , /// ABI version (often this is just padding)
    pub abi_version: u8
  , _padding: [u8; 7]
}

impl Ident {
    #[inline] pub fn check_magic(&self) -> bool { self.magic == MAGIC }
}

/// Identifies the class of the ELF file
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Class {
    /// Invalid ELF class file (`ELFCLASSNONE` in the standard)
    None  = 0
  , /// 32-bit ELF file (`ELFCLASS32` in the standard)
    Elf32 = 1
  , /// 64-bit ELF file (`ELFCLASS64` in the standard)
    Elf64 = 2
}

/// Identifies the data encoding of the ELF file
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum DataEncoding {
    /// Invalid data encoding (`ELFDATANONE` in the standard)
    None  = 0
  , /// Twos-complement little-endian data encoding
    /// (`ELFDATA2LSB` in the standard)
    LittleEndian = 1
  , /// Twos-complement big-endian data encoding
    /// (`ELFDATA2MSB` in the standard)
    BigEndian = 2
}

/// Operating system ABI
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OsAbi { /// Ox00 also represents "none"
                 SystemV = 0x00
               , HpUx    = 0x01
               , NetBsd  = 0x02
               , Linux   = 0x03
               , Solaris = 0x06
               , Aix     = 0x07
               , Irix    = 0x08
               , FreeBsd = 0x09
               , OpenBsd = 0x0C
               , OpenVms = 0x0D
               }

/// Identifies the version of the ELF file
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Version { None = 0, Current = 1 }

#[derive(Clone, Copy, PartialEq)]
struct TypeRepr(u16);

impl TypeRepr {
    pub fn as_type(&self) -> Type {
        match self.0 {
            0 => Type::None
          , 1 => Type::Relocatable
          , 2 => Type::Executable
          , 3 => Type::SharedObject
          , 4 => Type::Core
          , anything => Type::Other(anything)
        }
    }
}

impl fmt::Debug for TypeRepr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_type().fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Type { None
              , Relocatable
              , Executable
              , SharedObject
              , Core
              , Other(u16)
              }

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u16)]
pub enum Machine { None    = 0
                 , Sparc   = 0x02
                 , X86     = 0x03
                 , Mips    = 0x08
                 , PowerPc = 0x14
                 , Arm     = 0x28
                 , SuperH  = 0x2A
                 , Ia64    = 0x32
                 , X86_64  = 0x3E
                 , AArch64 = 0xB7
                 }
