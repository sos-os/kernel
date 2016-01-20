/// Enum representing an ELF file section type.
///
/// Refer to Figure 1-10: "Section Types, sh_type" in Section 1 of the
/// [ELF standard](http://www.sco.com/developers/gabi/latest/ch4.sheader.html)
/// for more information.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum SectionType {
    /// Section type 0: `SHT_NULL`
    ///
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have
    /// undefined values.
    Null         = 0
  , /// Section type 1: `SHT_PROGBITS`
    ///
    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramBits = 1
  , /// Section type 2: `SHT_SYMTAB`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    SymbolTable = 2
  , /// Section type 3: `SHT_STRTAB`
    ///
    /// The section holds a string table. An object file may have multiple
    /// string table sections.
    StringTable  = 3
  , /// Section type 4: `SHT_RELA`
    ///
    /// The section holds relocation entries with explicit addends, such as
    /// type `Elf32_Rela` for the 32-bit class of object files. An object file
    /// may have multiple relocation sections.
    Rela         = 4
  , /// Section type 5: `SHT_HASH`
    ///
    /// The section holds a symbol hash table. All objects participating in
    /// dynamic linking must contain a symbol hash table. Currently, an object
    /// file may have only one hash table, but this restriction may be relaxed
    /// in the future.
    HashTable   = 5
  , /// Section type 6: `SHT_DYNAMIC`
    ///
    /// The section holds information for dynamic linking. Currently, an object
    /// file may have only one dynamic section, but this restriction may be
    ///  relaxed in the future.
    Dynamic     = 6
  , /// Section type 7: `SHT_NOTE`
    ///
    /// The section holds information that marks the file in some way.
    Notes       = 7
  , /// Section type 8: `SHT_NOBITS`
    ///
    /// A section of this type occupies no space in the file but otherwise
    /// resembles `SHT_PROGBITS`. Although this section contains no bytes, the
    /// `sh_offset` member contains the conceptual file offset.
    NoBits      = 8
  , /// Section type 9: `SHT_REL`
    ///
    /// The section holds relocation entries without explicit addends, such as
    /// type `Elf32_Rel` for the 32-bit class of object files. An object file
    /// may have multiple reloca- tion sections.
    Rel         = 9
  , /// Section type 10: `SHT_SHLIB`
    ///
    /// This section type is reserved but has unspecified semantics. Programs
    /// that contain a section of this type do not conform to the ABI.
    Shlib       = 10
  , /// Section type 11: `SHT_DYNSYM`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    DynSymTable = 11
    // The remainder is reserved
}


#[derive(Debug)]
#[repr(u32)]
pub enum SectionFlags { Writable    = 0x1
                      , Allocated   = 0x2
                      , Executable  = 0x4
                      }

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub mod elf64;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub type Section = elf64::Section;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub type SectionsTag = elf64::SectionsTag;
