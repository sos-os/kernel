//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for reading & extracting data from Multiboot 2 boot information.
//!
//! Consult the Multiboot [specification]  for more information.
//!
//! [specification]: http://nongnu.askapache.com/grub/phcoder/multiboot.pdf
use memory::PAddr;
use elf::section::{Header as Section, Sections, HeaderRepr};
use core::iter::IntoIterator;

const END_TAG_LEN: u32 = 8;

const HEADER_LEN: u32 = 24;

pub const MAGIC: u32 = 0xe85250d6;

#[repr(u32)]
pub enum HeaderArch {
      I386 = 0
    , Mips = 4
}

#[repr(C)]
pub struct Header {
      pub magic: u32
    , pub arch: HeaderArch
    , pub header_length: u32
    , pub checksum: u32
    , pub end_tag: Tag
}

#[linkage = "external"]
#[link_section = ".multiboot_header"]
pub static HEADER: Header = Header {
    magic: MAGIC
  , arch: HeaderArch::I386
  , header_length: HEADER_LEN
  , checksum: -((MAGIC + 0 + HEADER_LEN) as i32) as u32
  , end_tag: Tag { ty: TagType::End
                 , length: END_TAG_LEN
                 }
};

#[repr(C)]
pub struct Info { pub length: u32
                , _pad: u32
                , tag_start: Tag
                }

impl Info {

    pub unsafe fn from(addr: PAddr) -> Result<&'static Self, &'static str> {
        let info: &Info = &*(addr.into(): u64 as *const Info);
        // TODO: check if the multiboot tag *exists* at this location as well?
        //       since if we pass in the wrong address, we'll still make the
        //       "no end tag" error.
        //
        //       which, i suppose is *technically* correct, but not very
        //       helpful...
        //          - eliza, 03/04/2017
        if info.has_end() {
            Ok(info)
        } else {
            Err( "Multiboot info structure at {:?} had no end tag!")
        }
    }

    /// Finds the tag with the given tag type.
    ///
    /// This is actually safe since the tag types are constrained by The
    /// `TagType` enum
    ///
    /// # Returns
    ///  - `Some(tag)` if a tag of the given type could be found.
    ///  - `None` if no tag of the given type could be found.
    pub fn get_tag(&'static self, tag_type: TagType) -> Option<&'static Tag> {
        self.tags()
            .find(|t| t.ty == tag_type)
    }

    /// Finds the memory map tag.
    ///
    ///  # Returns
    ///  - `Some(MemMapTag)` if a memory map tag could be found
    ///  - `None` if no tag of the given type could be found.
    #[inline]
    pub fn mem_map(&'static self) -> Option<&'static MemMapTag> {
        self.get_tag(TagType::MemoryMap)
            .map(|tag| unsafe { &*((tag as *const Tag) as *const MemMapTag) })
    }

    /// Finds the ELF sections tag.
    ///
    ///  # Returns
    ///  - `Some(ElfSectionsTag)` if a memory map tag could be found
    ///  - `None` if no tag of the given type could be found.
    #[inline]
    pub fn elf_sections(&'static self) -> Option<&'static ElfSectionsTag> {
        self.get_tag(TagType::ELFSections)
            .map(|tag| unsafe {
                &*((tag as *const Tag) as *const ElfSectionsTag)
            })
    }

    /// Returns an iterator over all Multiboot tags.
    #[inline]
    fn tags(&'static self) -> Tags { Tags(&self.tag_start as *const Tag) }

    /// Returns true if the multiboot structure has a valid end tag.
    fn has_end(&self) -> bool {
        let end_tag_addr
            = (self as *const _) as usize +
              (self.length - END_TAG_LEN) as usize;
        let end_tag = unsafe {&*(end_tag_addr as *const Tag)};
        end_tag.ty == TagType::End && end_tag.length == 8
    }
}

impl IntoIterator for &'static Info {
    type IntoIter = Tags;
    type Item = &'static Tag;
    #[inline]  fn into_iter(self) -> Self::IntoIter { self.tags() }

}


/// A Multiboot tag.
///
/// From the specification:
///
/// Boot information consists of a fixed part and a series of tags.
/// Its start is 8-bytes aligned. Fixed part is as following:
///
///<rawtext>
///
///             +-------------------+
///     u32     | total_size        |
///     u32     | reserved          |
///             +-------------------+
///</rawtext>
///
/// `total_size` contains the total size of boot information including this
/// field and terminating tag in bytes.
/// `reserved` is always set to zero and must be ignored by OS image.
///
///  Every tag begins with following fields:
///<rawtext>
///
///             +-------------------+
///     u32     | type              |
///     u32     | size              |
///             +-------------------+
///</rawtext>
/// `type` contains an identifier of contents of the rest of the tag. `size`
/// contains the size of tag including header fields but not including padding.
/// Tags follow one another padded when necessary in order for each tag to
/// start at 8-bytes aligned address. Tags are terminated by a tag of type `0`
/// and size `8`.
#[repr(C)]
#[derive(Debug)]
pub struct Tag { /// the type of this tag.
                 pub ty: TagType
               , length: u32
               }

/// Types of Multiboot tags
///
/// Refer to Chapter 3 of the Multiboot 2 spec
#[repr(u32)]
#[derive(Debug, Eq, PartialEq)]
pub enum TagType { /// Tag that indicates the end of multiboot tags
                   End              = 0
                 , /// Command line passed to the bootloader
                   CommandLine      = 1
                 , BootloaderName   = 2
                 , Modules          = 3
                 , BasicMemInfo     = 4
                 , BIOSBootDev      = 5
                 , MemoryMap        = 6
                 , VBEInfo          = 7
                 , FramebufferInfo  = 8
                 , ELFSections      = 9
                 , APMTable         = 10
                 }

/// An iterator over Multiboot 2 tags.
pub struct Tags(*const Tag);

impl Tags {
    #[inline] fn advance(&mut self, size: u32) {
        let next_addr = self.0 as usize + size as usize;
        self.0 = (((next_addr-1) & !0x7) + 0x8) as *const _;
    }
}

impl Iterator for Tags {
    type Item = &'static Tag;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &*self.0 } {
            &Tag { ty: TagType::End, length: END_TAG_LEN } => None
          , tag => {
              self.advance(tag.length);
              Some(tag)
            }
        }
    }
}

/// A Memory Map tag
#[repr(C)]
pub struct MemMapTag { tag: Tag
                     , pub entry_size: u32
                     , pub entry_version: u32
                     , first_entry: MemArea
                     }

impl MemMapTag {

    /// Returns an iterator over all the memory areas in this tag.
    #[inline] pub fn areas(&'static self) -> MemAreas {
        MemAreas { curr: (&self.first_entry) as *const MemArea
                 , last: ((self as *const MemMapTag as u32) +
                         self.tag.length - self.entry_size)
                         as *const MemArea
                 , size: self.entry_size
                 }
    }
}

impl IntoIterator for &'static MemMapTag {
    type Item = &'static MemArea;
    type IntoIter = MemAreas;

    #[inline] fn into_iter(self) -> Self::IntoIter { self.areas() }

}


/// A tag that stores the boot command line.
#[repr(C)]
pub struct CommandLineTag { tag: Tag
                          , /// The boot command line.
                            ///
                            /// The command line is a normal C-style zero-
                            /// terminated UTF-8 string.
                            pub command_line: [u8]
                          }


#[repr(C)]
pub struct ModulesTag { tag: Tag
                      , /// The address at which the module begins.
                        pub mod_begin: PAddr
                      , /// The address at which the module ends.
                        pub mod_end: PAddr
                      , /// A string (typically a command line)
                        pub string: [u8]
                      }

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemAreaType { Available = 1
                     , Acpi      = 3
                     , Preserve  = 4
                     }

/// A multiboot 2 memory area
#[repr(C)]
pub struct MemArea { /// the starting address of the memory area
                     pub base: PAddr
                   , /// the length of the memory area
                     pub length: PAddr
                   , /// the type of the memory area
                     pub ty: MemAreaType
                   , _pad: u32
                   }

impl MemArea {
    #[inline] pub fn address(&self) -> PAddr {
        self.base + self.length - 1
    }
}

/// An iterator over memory areas
#[derive(Clone)]
pub struct MemAreas { curr: *const MemArea
                    , last: *const MemArea
                    , size: u32
                    }

impl Iterator for MemAreas {
    type Item = &'static MemArea;

    fn next(&mut self) -> Option<&'static MemArea> {
        if self.curr > self.last {
            None
        } else {
            let current = unsafe { &*self.curr };
            self.curr = (self.curr as u32 + self.size) as *const MemArea;
            if current.ty == MemAreaType::Available {
                Some(current)
            } else {
                self.next()
            }
        }
    }
}

#[cfg(target_pointer_width = "32")]
pub type Word = u32;
#[cfg(target_pointer_width = "64")]
pub type Word = u64;

/// A Multiboot 2 ELF sections tag
#[derive(Debug)]
#[repr(packed)]
pub struct ElfSectionsTag { tag: Tag
                          , /// the number of sections pointed to by this tag
                            pub n_sections: u32
                          , /// the size of each ELF section
                            pub section_size: u32
                          , stringtable_idx: u32
                          , first_section: HeaderRepr<Word>
                          }

impl ElfSectionsTag {
    /// Returns an iterator over the ELF sections pointed to by this tag.
    //  TODO: can the &'static bound be reduced to &'a? is there any reason to?
    //          - eliza, 03/04/2017
    #[inline] pub fn sections(&'static self) -> Sections<'static, Word> {
        Sections::new( &self.first_section
                     , self.n_sections - 1
                     , self.section_size
                     )
    }
}

impl IntoIterator for &'static ElfSectionsTag {
    //  TODO: can the &'static bound be reduced to &'a? is there any reason to?
    //          - eliza, 03/04/2017
    type Item = Section<'static>;
    type IntoIter = Sections<'static, Word>;

    #[inline] fn into_iter(self) -> Self::IntoIter { self.sections() }

}
