//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! ELF file header
use super::{ElfResult, ElfWord, Section, section};
use super::ValidatesWord;

use section::Header as SectionHeader;

use core::{fmt, mem, convert};
use core::ops::Range;

/// Trait representing an ELF File Header.
///
/// This trait allows [HeaderRepr] to provide a consistent API regardless
/// of whether the header section uses 32- or 64-bit [ELF word]s. A number of
/// field values in the header of various sizes are converted to `usize` by
/// this API so that they can be used as indices, etc.
///
/// For more information on ELF File Headers, refer to:
/// + the ELF [specification]
/// + the [OS Dev Wiki]
///
/// [ELF word]: ../type.ElfWord.html
/// [HeaderRepr]: struct.HeaderRepr.html
/// [specification](http://www.sco.com/developers/gabi/latest/ch4.eheader.html)
/// [OS Dev Wiki](http://wiki.osdev.org/ELF#Header)
pub trait Header: Sized {
    type Word: ElfWord;

    /// Attempt to extract an ELF file header from a slice of bytes.
    fn from_slice<'a>(input: &'a [u8]) -> ElfResult<&'a Self>;

    /// Attempt to extract a section header from a slice of bytes.
    /// TODO: can/should the index be `usize`?
    //          - eliza, 03/08/2017
    fn parse_section<'a>(&'a self, input: &'a [u8], idx: u16)
                            -> ElfResult<&'a Section<Self::Word>>;

    fn sh_range(&self) -> Range<usize> {
        let start = self.sh_offset();
        start .. start + (self.sh_entry_size() * self.sh_count())
    }

    fn ph_range(&self) -> Range<usize> {
        let start = self.ph_offset();
        start .. start + (self.ph_entry_size() * self.ph_count())
    }

    /// Calculate the index for a [section header]
    ///
    /// TODO: should this check the index is reasonable & return a `Result`
    //          - eliza, 03/10/2017
    /// [section header]: ../section/struct.Header.html
    fn section_index(&self, idx: usize) -> Range<usize> {
        let size = self.sh_entry_size();
        let start = self.sh_offset() + (idx * size);
        start .. start + size
    }

    /// Calculate the index for a program header
    ///
    /// TODO: should this check the index is reasonable & return a `Result`
    //          - eliza, 03/10/2017
    fn program_header_index(&self, idx: usize) -> Range<usize> {
        let size = self.ph_entry_size();
        let start = self.ph_offset() + (idx * size);
        start .. start + size
    }

    // Field accessors -------------------------------------------
    fn ident(&self) -> Ident;
    fn get_type(&self) -> Type;
    fn machine(&self) -> Machine;
    /// Offset of the program entry point
    fn entry_point(&self) -> usize;
    /// Offset of the start of program headers
    fn ph_offset(&self) -> usize;
    /// Number of program headers.
    fn ph_count(&self) -> usize;
    /// Size of a program header.
    fn ph_entry_size(&self) -> usize;
    /// Offset of the start of [section header]s.
    ///
    /// [section header]: ../section/struct.Header.html
    fn sh_offset(&self) -> usize;
    /// Number of [section header]s.
    ///
    /// [section header]: ../section/struct.Header.html
    fn sh_count(&self) -> usize;
    /// Size of a [section header].
    ///
    /// [section header]: ../section/struct.Header.html
    fn sh_entry_size(&self) -> usize;
    /// TODO: can this return the flags type?
    //          - eliza, 03/08/2017
    fn flags(&self) -> u32;
    /// Index of the section header [string table].
    ///
    /// [string table]: ../section/struct.StrTable.html"]
    fn sh_str_idx(&self) -> usize;
}

macro_rules! Header {
    (($($size:ty),+) $(pub)* enum $name:ident $($tail:tt)* ) => {
        Header! { @impl $name, $($size)+ }
    };
    (($($size:ty),+) $(pub)* struct $name:ident $($tail:tt)*) => {
        Header! { @impl $name, $($size)+ }
    };
    (@impl $name:ident, $($size:ty)+) => {
        $(impl Header for $name<$size> {
            type Word = $size;
            /// Attempt to extract an ELF file header from a slice of bytes.
            fn from_slice<'a>(input: &'a [u8]) -> ElfResult<&'a Self> {
                if input.len() < mem::size_of::<Self>() {
                    Err("Input too short to extract ELF header")
                } else {
                    unsafe {
                        super::extract_from_slice::<Self>(input, 0, 1)
                            .map(|x| &x[0]) }
                }
            }

            /// Attempt to extract a [section header] from a slice of bytes.
            ///
            /// TODO: should this move to the `File` type since it owns the
            ///       byte slice (which then wouldn't have to be passed as an
            ///       argument)?
            //          - eliza, 03/10/2017
            ///
            /// [section header]: ../section/struct.Header.html
            fn parse_section<'a>(&'a self, input: &'a [u8], idx: u16)
                                -> ElfResult<&'a Section<Self::Word>>
            where section::HeaderRepr<Self::Word>: SectionHeader<Word = Self::Word> {
                if idx < section::SHN_LORESERVE {
                    Err("Cannot parse reserved section.")
                } else {
                    // use ValidatesWord to check if this section's Class field
                    // will let us interpret the section with the requested
                    // word length. this is a bit of a hack around the type
                    // system not letting me do this the way I wanted to to....
                    let validator: &ValidatesWord<Self::Word>
                        = &self.ident.class;
                    validator.check()?;

                    let raw = &input[self.section_index(idx as usize)];

                    unsafe {
                        Ok(&*(raw as *const [u8]
                                  as *const _
                                  as *const section::HeaderRepr<Self::Word>))
                    }
                }
            }

            #[inline] fn get_type(&self) -> Type { self.elftype.as_type() }

            impl_getters! {
                #[doc = "Index for the start of [section header]s. \
                         [section header]: ../section/struct.Header.html"]
                fn sh_offset(&self) -> usize;
                fn sh_entry_size(&self) -> usize;
                fn sh_count(&self) -> usize;

                #[doc = "Index for the start of program headers"]
                fn ph_offset(&self) -> usize;
                fn ph_entry_size(&self) -> usize;
                fn ph_count(&self) -> usize;

                #[doc = "Index for the program entry point"]
                fn entry_point(&self) -> usize;
                #[doc = "Index of the section header [string table] \
                         [string table]: ../section/struct.StrTable.html"]
                fn sh_str_idx(&self) -> usize;
                fn flags(&self) -> u32;
                fn ident(&self) -> Ident;
                fn machine(&self) -> Machine;
            }
        }

        impl<'a> convert::TryFrom<&'a [u8]> for &'a $name<$size> {
            type Error = &'static str;
            #[inline]
            fn try_from(slice: &'a [u8]) -> ElfResult<Self> {
                <$name<$size> as Header>::from_slice(slice)
            }
        }
        )+
    }
}

macro_attr! {
    /// Raw representation of an ELF file header.
    #[derive(Copy, Clone, Debug, Header!(u32, u64))]
    #[repr(C, packed)]
    pub struct HeaderRepr<W: ElfWord> {
        /// the ELF [file identifier](struct.Ident.html)
        pub ident: Ident
      , elftype: TypeRepr
      , pub machine: Machine
      , /// Program entry point
        entry_point: W
      , /// Offset for start of program headers
        ph_offset: W
      , /// Offset for start of [section header]s.
        /// [section header]: ../section/struct.Header.html
        sh_offset: W
      , pub flags: u32
      , pub header_size: u16
      , pub ph_entry_size: u16
      , pub ph_count: u16
      , pub sh_entry_size: u16
      , pub sh_count: u16
      , /// Index of the section header string table
        sh_str_idx: u16
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
    /// ELF magic numbers. Must be equal to the [ELF magic], `[0x7, E, L, F]`.
    ///
    /// [ELF magic]: constant.MAGIC.html
    pub magic: Magic
  , /// ELF [file class](enum.Class.html) (32- or 64-bit)
    pub class: Class
  , /// ELF [data encoding](enum.DataEncoding.html) (big- or little-endian)
    pub encoding: DataEncoding
  , /// ELF file [version](enum.Version.html)
    pub version: Version
  , /// What [operating system ABI] this file was compiled for.
    ///
    /// [operating system ABI]: enum.OsAbi.html
    pub abi: OsAbi
  , /// ABI version (often this is just padding)
    pub abi_version: u8
  , _padding: [u8; 7]
}

impl Ident {
    #[inline] pub fn check_magic(&self) -> bool { self.magic == MAGIC }

    /// Returns true if the identifier section identifies a valid ELF file.
    #[inline] pub fn is_valid(&self) -> bool {
        // the ELF magic number is correct
        self.check_magic() &&
        // the file class is either 32- or 64-bits
        self.class.is_valid() &&
        // the data encoding is either big- or little-endian
        self.encoding.is_valid()
    }
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

impl Class {
    /// Returns true if the class field for this file is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        match *self { Class::None => false
                    , _ => true
                    }
    }

}

impl ValidatesWord<u64> for Class {
    #[inline]
    fn check(&self) -> ElfResult<()> {
        use self::Class::*;
        match *self {
            None => Err("Invalid ELF type ELFCLASSNONE!")
          , Elf32 => Err("Cannot extract 64-bit section from 32-bit ELF")
          , Elf64 => Ok(())
        }
    }
}

impl ValidatesWord<u32> for Class {
    #[inline]
    fn check(&self) -> ElfResult<()> {
        use self::Class::*;
        match *self {
            None => Err("Invalid ELF type ELFCLASSNONE!")
          , Elf64 =>
                // TODO: is this actually true?
                //          - eliza, 03/08/2017
                Err("Cannot extract 32-bit section from 64-bit ELF")
          , Elf32 => Ok(())
        }
    }
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

impl DataEncoding {
    /// Returns true if the data encoding field for this file is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        match *self { DataEncoding::None => false
                    , _ => true
                    }
    }
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
    /// TODO: rewrite this as a `convert::Into` implementation
    ///         - eliza, 03/09/2017
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
