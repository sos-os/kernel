//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use super::{ElfResult, ElfWord};

use core::{convert, fmt, ops};
use core::iter::IntoIterator;

use memory::{Addr, PAddr};

// Distinguished section indices.
pub const SHN_UNDEF: u16        = 0;
pub const SHN_LORESERVE: u16    = 0xff00;
pub const SHN_LOPROC: u16       = 0xff00;
pub const SHN_HIPROC: u16       = 0xff1f;
pub const SHN_LOOS: u16         = 0xff20;
pub const SHN_HIOS: u16         = 0xff3f;
pub const SHN_ABS: u16          = 0xfff1;
pub const SHN_COMMON: u16       = 0xfff2;
pub const SHN_XINDEX: u16       = 0xffff;
pub const SHN_HIRESERVE: u16    = 0xffff;

pub const SHT_LOOS: u32   = 0x60000000;
pub const SHT_HIOS: u32   = 0x6fffffff;
pub const SHT_LOPROC: u32 = 0x70000000;
pub const SHT_HIPROC: u32 = 0x7fffffff;
pub const SHT_LOUSER: u32 = 0x80000000;
pub const SHT_HIUSER: u32 = 0xffffffff;

/// Represents an ELF section header
///
/// Refer to [Figure 4-8], "Section Header", from Chapter 4 of the ELF standard
/// for more information.
///
/// [Figure 4-8]: (http://www.sco.com/developers/gabi/latest/ch4.sheader.html#section_header)
// #[derive(Clone, Copy, Debug)]
// pub enum Header<'a> {
//     ThirtyTwo(&'a HeaderRepr<u32>)
//   , SixtyFour(&'a HeaderRepr<u64>)
// }
pub trait Header: fmt::Debug {
    type Word: ElfWord;

    // /// Returns the start address of this section
    // fn addr(&self) -> PAddr;

    /// Returns the end address of this section
    /// TODO: refactor this to return a Range instead?
    //          - eliza, 03/14/2017
    #[inline] fn end_address(&self) -> PAddr {
        self.address() + self.length() as <PAddr as Addr>::Repr
    }

    /// Returns true if this section is writable.
    #[inline] fn is_writable(&self) -> bool {
        self.flags().contains(SHF_WRITE)
    }

    /// Returns true if this section occupies memory during program execution.
    #[inline] fn is_allocated(&self) -> bool {
        self.flags().contains(SHF_ALLOC)
    }

    /// Returns true if this section contains executable instructions.
    #[inline] fn is_executable(&self) -> bool {
        self.flags().contains(SHF_EXECINSTR)
    }

    /// Returns true if this section can be merged.
    #[inline] fn is_mergeable(&self) -> bool {
        self.flags().contains(SHF_MERGE)
    }

    /// Returns true if this section contains data that is of a uniform size.
    #[inline] fn is_uniform(&self) -> bool {
        let flags = self.flags();
        flags.contains(SHF_MERGE) &&
        !flags.contains(SHF_STRINGS)
    }

    /// Look up the name of this section in the passed string table.
    #[inline] fn get_name<'a>(&self, strtab: StrTable<'a>) -> &'a str {
        unimplemented!()
    }

    // Field accessors -------------------------------------------------
    fn name_offset(&self) -> u32;
    /// This member categorizes the section's contents and semantics.
    fn get_type(&self) -> ElfResult<Type>;
    fn flags(&self) -> Flags;
    fn address(&self) -> PAddr;
    fn offset(&self) -> usize;
    /// TODO: should offset + length make a Range?
    //          - eliza, 03/14/2017
    fn length(&self) -> usize;
    fn link(&self) -> u32;
    fn info(&self) -> u32;
    fn address_align(&self) -> Self::Word;
    fn entry_length(&self) -> usize;
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

            /// Returns the type of this section
            #[inline] fn get_type(&self) -> ElfResult<Type> {
                 self.ty.as_type()
             }

            impl_getters! {
                fn name_offset(&self) -> u32;
                fn flags(&self) -> Flags;
                /// TODO: shold this return a PAddr?
                //          - eliza, 03/14/2017
                fn address(&self) -> PAddr;
                fn offset(&self) -> usize;
                /// TODO: should offset + length make a Range?
                //          - eliza, 03/14/2017
                fn length(&self) -> usize;
                fn link(&self) -> u32;
                fn info(&self) -> u32;
                fn address_align(&self) -> Self::Word;
                fn entry_length(&self) -> usize;
            }
        })+
    };
}

impl<Word> fmt::Display for Header<Word = Word>
where Word: ElfWord
    , Header<Word = Word>: fmt::Debug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: do we want to actually get the section's name from the string
        //       table and display it here?
        //          - eliza, 03/05/2017
        // TODO: do we want to print the header's flags, or would that make the //       format too long?
        //          - eliza, 03/05/2017
        if let Ok(ty) = self.get_type() {
            // the ELF section was valid
            write!(f, "{:?} section at {:?} ... {:?}"
                  , ty, self.address(), self.end_address())
        } else {
            // we couldn't successfully extract a type from the ELF section!
            write!(f, "Bad ELF section {:?}", self)
        }
    }
}

macro_attr! {
    /// Raw representation of an ELF section header in an ELF binary.
    ///
    /// Refer to [Figure 4-8], "Section Header", from Chapter 4 of the ELF
    /// standard for more information.
    ///
    /// [Figure 4-8]: (http://www.sco.com/developers/gabi/latest/ch4.sheader.html#section_header)
    //  TODO: add docs for all fields!
    //          - eliza, 03/05/2017
    #[derive(Debug, Header!(u32, u64))]
    #[repr(C)]
    pub struct HeaderRepr<Word> {
        /// This member specifies the name of the section.
        ///
        /// Its value is an index into the section header string table section,
        /// giving the location of a null-terminated string.
        name_offset: u32
      , /// This member categorizes the section's contents and semantics.
        ty: TypeRepr
      , flags: Flags
      , address: Word
      , offset: Word
      , length: Word
      , link: u32
      , info: u32
      , address_align: Word
      , entry_length: Word
    }
}


bitflags! {
    // TODO: add documentation to the flags
    //          - eliza, 03/05/2017
    pub flags Flags: usize {
        // Flags (SectionHeader::flags)
        const SHF_WRITE            =        0x1
      , const SHF_ALLOC            =        0x2
      , const SHF_EXECINSTR        =        0x4
      , const SHF_MERGE            =       0x10
      , const SHF_STRINGS          =       0x20
      , const SHF_INFO_LINK        =       0x40
      , const SHF_LINK_ORDER       =       0x80
      , const SHF_OS_NONCONFORMING =      0x100
      , const SHF_GROUP            =      0x200
      , const SHF_TLS              =      0x400
      , const SHF_COMPRESSED       =      0x800
      , const SHF_MASKOS           = 0x0ff00000
      , const SHF_MASKPROC         = 0xf0000000
    }
}

impl fmt::LowerHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.bits.fmt(f)
    }
}

bitflags! {
    pub flags GroupFlags: u32 {
        const GRP_COMDAT	=        0x1
      , const GRP_MASKOS	= 0x0ff00000
      , const GRP_MASKPROC	= 0xf0000000
    }
}

pub enum Contents<'a> {
    Empty
  , Undefined(&'a [u8])
  , Group { flags: &'a u32
          , indicies: &'a[u32] }
}

/// Underlying representation of a section header type field
///
/// Unfortunately, we cannot have enums with open ranges yet, so we have
/// to convert between the ELF file underlying representation and our
/// type-safe representation.
///
/// Refer to [Figure 4-9]: "Section Types, `sh_type`" in Section 4 of the
/// ELF standard for more information.
///
/// [Figure 4-9]:  http://www.sco.com/developers/gabi/latest/ch4.sheader.html#sh_type
#[derive(Debug, Copy, Clone)]
struct TypeRepr(u32);

impl TypeRepr {
    /// TODO: rewrite this as a `TryFrom` implementation (see issue #85)
    //          - eliza, 03/09/2017
    #[inline] fn as_type(&self) -> ElfResult<Type> {
        match self.0 {
            0 => Ok(Type::Null)
          , 1 => Ok(Type::ProgramBits)
          , 2 => Ok(Type::SymbolTable)
          , 3 => Ok(Type::StringTable)
          , 4 => Ok(Type::Rela)
          , 5 => Ok(Type::HashTable)
          , 6 => Ok(Type::Dynamic)
          , 7 => Ok(Type::Notes)
          , 8 => Ok(Type::NoBits)
          , 9 => Ok(Type::Rel)
          , 10 => Ok(Type::Shlib)
          , 11 => Ok(Type::DynSymTable)
          , 14 => Ok(Type::InitArray)
          , 15 => Ok(Type::FiniArray)
          , 16 => Ok(Type::PreInitArray)
          , x @ SHT_LOOS ... SHT_HIOS => Ok(Type::OsSpecific(x))
          , x @ SHT_LOPROC ... SHT_HIPROC => Ok(Type::ProcessorSpecific(x))
          , x @ SHT_LOUSER ... SHT_HIUSER => Ok(Type::User(x))
          , _ => Err("Invalid section type!")
        }
    }
}

/// Enum representing an ELF file section type.
///
/// Refer to [Figure 4-9]: "Section Types, `sh_type`" in Section 4 of the
/// ELF standard for more information.
///
/// [Figure 4-9]:  http://www.sco.com/developers/gabi/latest/ch4.sheader.html#sh_type
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    /// Section type 0: `SHT_NULL`
    ///
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have
    /// undefined values.
    Null
  , /// Section type 1: `SHT_PROGBITS`
    ///
    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramBits
  , /// Section type 2: `SHT_SYMTAB`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    SymbolTable
  , /// Section type 3: `SHT_STRTAB`
    ///
    /// The section holds a string table. An object file may have multiple
    /// string table sections.
    StringTable
  , /// Section type 4: `SHT_RELA`
    ///
    /// The section holds relocation entries with explicit addends, such as
    /// type `Elf32_Rela` for the 32-bit class of object files. An object file
    /// may have multiple relocation sections.
    Rela
  , /// Section type 5: `SHT_HASH`
    ///
    /// The section holds a symbol hash table. All objects participating in
    /// dynamic linking must contain a symbol hash table. Currently, an object
    /// file may have only one hash table, but this restriction may be relaxed
    /// in the future.
    HashTable
  , /// Section type 6: `SHT_DYNAMIC`
    ///
    /// The section holds information for dynamic linking. Currently, an object
    /// file may have only one dynamic section, but this restriction may be
    ///  relaxed in the future.
    Dynamic
  , /// Section type 7: `SHT_NOTE`
    ///
    /// The section holds information that marks the file in some way.
    Notes
  , /// Section type 8: `SHT_NOBITS`
    ///
    /// A section of this type occupies no space in the file but otherwise
    /// resembles `SHT_PROGBITS`. Although this section contains no bytes, the
    /// `sh_offset` member contains the conceptual file offset.
    NoBits
  , /// Section type 9: `SHT_REL`
    ///
    /// The section holds relocation entries without explicit addends, such as
    /// type `Elf32_Rel` for the 32-bit class of object files. An object file
    /// may have multiple reloca- tion sections.
    Rel
  , /// Section type 10: `SHT_SHLIB`
    ///
    /// This section type is reserved but has unspecified semantics. Programs
    /// that contain a section of this type do not conform to the ABI.
    Shlib
  , /// Section type 11: `SHT_DYNSYM`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    DynSymTable
  , InitArray
  , FiniArray
  , PreInitArray
  , Group
  , SymbolTableShIndex
  , OsSpecific(u32)
  , ProcessorSpecific(u32)
  , User(u32)
}

/// Iterator over ELF64 sections
#[derive(Clone,Debug)]
pub struct Sections<'a, W: 'a>
where W: ElfWord { curr: &'a HeaderRepr<W>
                 , remaining: u32
                 , size: u32
                 }

impl<'a, W: 'a> Sections<'a, W>
where W: ElfWord {

    pub fn new(curr: &'a HeaderRepr<W>, remaining: u32, size: u32)
               -> Sections<'a, W>
    {
        Sections { curr: curr, remaining: remaining, size: size }

    }

}


impl<'a, W> Iterator for Sections<'a, W>
where W: ElfWord
    , HeaderRepr<W>: Header<Word = W> {
    type Item = &'a Header<Word = W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            let current = self.curr;
            self.curr =  unsafe {
                (self.curr as *const HeaderRepr<W>).offset(1)
                    .as_ref()
                    .expect("Expected an ELF section header, but got a null \
                             pointer!\nThis shouldn't happen!")
            };
            self.remaining -= 1;
            if current.get_type().unwrap() == Type::Null {
                self.next()
            } else {
                Some(current)
            }
        }
    }
}

/// Characters in the ELF string table are 8-bit ASCII characters.
type ElfChar = u8;

/// An ELF string table.
///
/// Refer to the String Table [entry] in Section 4 of the ELF standard
/// for more information.
///
/// [entry]: http://www.sco.com/developers/gabi/latest/ch4.strtab.html
//  TODO: this should be indexable by string number, possibly?
//          - eliza, 03/07/2017
//  TODO: add a function to get the name of the section (which always lives)
//        at index 0.
//          - eliza, 03/07/2017
#[derive(Clone, Debug)]
pub struct StrTable<'a>(&'a [ElfChar]);

impl<'a> convert::From<&'a [ElfChar]> for StrTable<'a> {
    #[inline(always)]
    fn from(binary: &'a [ElfChar]) -> Self { StrTable(binary) }
}

impl<'a> StrTable<'a> {

    /// Returns the string at a given index in the string table,
    /// if there is one.
    // TODO: these docs are Bad
    //          - eliza, 03/07/2017
    // TODO: this def. shouldn't be u64, but i didn't want to annotate the
    //       string table type with ElfWord...figure this out
    //          - eliza, 03/07/2017
    // TODO: can this be replaced with an ops::Index implementation?
    //       but then we can't implement Deref to a slice any more?
    //          - eliza, 03/07/2017
    pub fn at_index(&'a self, i: usize) -> Option<&'a str> {
        use core::str::from_utf8_unchecked;
        if i <= self.len() {
            read_to_null(&self[i..])
                .map(|bytes| unsafe {
                    // TODO: should this be checked, or do we assume the ELF
                    //       binary has only well-formed strings? this could be
                    //       a Security Thing...
                    //          - eliza, 03/07/2017
                    from_utf8_unchecked(bytes)
                    // TODO: can the conversion to a Rust string be moved to
                    //       `read_to_null()`? we also do this in the iterator
                    //          - eliza, 03/07/2017
                })

        } else {
            None
        }
    }
}

// impl<'a> StrTable<'a> {
//     #[inline] fn len(&self) -> usize { self.0.len}
//
// }
impl<'a> ops::Deref for StrTable<'a> {
    type Target = [ElfChar];

    #[inline] fn deref(&self) -> &Self::Target { self.0 }
}

impl<'a> IntoIterator for StrTable<'a> {
    type IntoIter = Strings<'a>;
    type Item = &'a str;

    //  TODO: this doesn't strictly need to consume the StringTable...
    //          - eliza, 03/07/2017
    #[inline] fn into_iter(self) -> Self::IntoIter { Strings(&self.0) }
}

/// Returns true if `ch` is the null-terminator character
#[inline] fn is_null(ch: &ElfChar) -> bool { *ch == b'\0' }

/// Read a series of bytes from a slice to the first null-terminator
//  TODO: can this be moved to the StrTable type? no big deal but it would be
//        somewhat prettier...
//          - eliza, 03/07/2017
#[inline] fn read_to_null<'a>(bytes: &'a [ElfChar]) -> Option<&'a [ElfChar]> {
    bytes.iter().position(is_null)
         .map(|i| &bytes[..i] )
}

/// Iterator over the strings in an ELF string table
#[derive(Clone, Debug)]
pub struct Strings<'a>(&'a [ElfChar]);

impl<'a> Iterator for Strings<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        use core::str::from_utf8_unchecked;
        if self.0.len() == 0 {
            // if there are no bytes remaining in the iterator, then we've
            // iterated over all the strings in the string table (or it was
            // empty to begin with).
            //
            // N.B. that `read_to_null()` _will_ return None in this case, so
            // this check isn't strictly necessary; it just saves us from
            // having to create an iterator if the slice is empty, so I'm
            // calling it an "optimisation".
            None
        } else {
            // otherwise, try to read the iterator's slice to the first null
            // character...
            read_to_null(self.0).map(|bytes| {
                // ...if we found a null character, remove the string's bytes
                // from the slice in the iterator (since we're returning that
                // string), and return a string slice containing those bytes
                // interpreted as UTF-8 (which should be equivalent to ASCII)
                self.0 = &self.0[bytes.len() + 1..];
                unsafe {
                    // TODO: should this be checked, or do we assume the ELF
                    //       binary has only well-formed strings? this could be
                    //       a Security Thing...
                    //          - eliza, 03/07/2017
                    from_utf8_unchecked(bytes)
                }
            })
        }
    }

    #[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.0.len() / 2))
    }
}
