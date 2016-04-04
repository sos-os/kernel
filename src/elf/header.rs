use ::memory::PAddr;

use super::Section;
use super::section;

use core::fmt;
use core::mem;

/// An ELF file header
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Header {
    pub ident: Ident
  , elftype: TypeRepr
  , pub machine: Machine
  , /// Program entry point
    pub entry_point: PAddr
  , /// Offset for start of program headers
    pub ph_offset: PAddr
  , /// Offset for start of section headers
    pub sh_offset: PAddr
  , pub flags: u32
  , pub header_size: u16
  , pub ph_entry_size: u16
  , pub ph_count: u16
  , pub sh_entry_size: u16
  , pub sh_count: u16
  , pub sh_str_idx: u16
}

impl Header {

    pub fn from_slice<'a>(input: &'a [u8]) -> Result<&'a Header, &str> {
        if input.len() < mem::size_of::<Header>() {
            Err("Input too short to extract ELF header")
        } else {
            unsafe { Ok(&super::extract_from_slice::<Header>(input, 0, 1)[0]) }
        }
    }

    pub fn section<'a>(&'a self, input: &'a [u8], idx: u16)
                      -> Result<&'a Section, &str>
    {
        if idx < section::SHN_LORESERVE {
            Err("Cannot parse reserved section.")
        } else {
            let start // start offset for section
                = self.sh_offset + idx as u64 * self.sh_entry_size as u64;
            let end // end offset for section
                = start + self.sh_entry_size as u64;

            match self.ident.class {
                Class::None => Err("Invalid ELF class (ELFCLASSNONE).")
              , Class::Elf32 => unimplemented!()
              , Class::Elf64 => unimplemented!()
            }
        }
    }

    #[inline]
    pub fn get_type(&self) -> Type { self.elftype.as_type() }


}

/// ELF header magic
pub const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

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
    pub data: Data
  , /// ELF file version
    pub version: Version
  , pub abi: OsAbi
  , /// ABI version (often this is just padding)
    pub abi_version: u8
  , _padding: [u8; 7]
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
pub enum Data {
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
               , NetBSD  = 0x02
               , Linux   = 0x03
               , Solaris = 0x06
               , Aix     = 0x07
               , Irix    = 0x08
               , FreeBSD = 0x09
               , OpenBSD = 0x0C
               , OpenVMS = 0x0D
               }

/// Identifies the version of the ELF file
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Version { None = 0, Current = 1 }

#[derive(Clone, Copy, PartialEq)]
pub struct TypeRepr(u16);

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
                 , PowerPC = 0x14
                 , Arm     = 0x28
                 , SuperH  = 0x2A
                 , Ia64    = 0x32
                 , X86_64  = 0x3E
                 , AArch64 = 0xB7
                 }
