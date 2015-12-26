//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
#![crate_name = "sos_multiboot2"]

#![feature( no_std
          , lang_items )]
#![feature( const_fn
          , slice_patterns )]
#![no_std]

const END_TAG_LEN: u32 = 8;
pub mod elf;
pub mod elf64;

#[repr(C)]
pub struct Info { pub length: u32
                , _pad: u32
                , tag_start: Tag
                }

impl Info {

    pub unsafe fn from(addr: usize) -> &'static Self {
        let info = &*(addr as *const Info);
        assert!(info.has_end());
        info
    }

    /// Finds the tag with the given tag type.
    ///
    /// This is actually safe since the tag types are constrained by The
    /// `TagType` enum
    pub fn get_tag(&self, tag_type: TagType) -> Option<&'static Tag> {
        self.tags()
            .find(|t| t.ty == tag_type)
    }

    #[inline]
    pub fn mem_map(&self) -> Option<&'static MemMapTag> {
        self.get_tag(TagType::MemoryMap)
            .map(|tag| unsafe { &*((tag as *const Tag) as *const MemMapTag) })
    }

    #[inline]
    pub fn elf64_sections(&self) -> Option<&'static elf64::SectionsTag> {
        self.get_tag(TagType::ELFSections)
            .map(|tag| unsafe {
                &*((tag as *const Tag) as *const elf64::SectionsTag)
            })
    }

    #[inline]
    fn tags(&self) -> Tags { Tags(&self.tag_start as *const Tag) }

    fn has_end(&self) -> bool {
        let end_tag_addr
            = (self as *const _) as usize +
              (self.length - END_TAG_LEN) as usize;
        let end_tag = unsafe {&*(end_tag_addr as *const Tag)};
        end_tag.ty == TagType::End && end_tag.length == 8
    }
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
pub struct Tag { ty: TagType
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

struct Tags(*const Tag);

impl Tags {
    #[inline] fn advance(&mut self, size: u32) {
        let next_addr = self.0 as usize + size as usize;
        self.0 = (((next_addr-1) & !0x7) + 0x8) as *const _;
    }
}

impl Iterator for Tags {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &*self.0 } {
            &Tag{ ty: TagType::End, length: END_TAG_LEN } => None
          , tag => { self.advance(tag.length); Some(tag) }
        }
    }
}

#[repr(C)]
pub struct MemMapTag { tag: Tag
                     , entry_size: u32
                     , entry_version: u32
                     , first_entry: MemArea
                     }

impl MemMapTag {

    pub fn areas(&self) -> MemAreas {
        MemAreas { curr: (&self.first_entry) as *const MemArea
                 , last: ((self as *const MemMapTag as u32) +
                         self.tag.length - self.entry_size)
                         as *const MemArea
                 , size: self.entry_size
                 }
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemAreaType { Available = 1
                     , ACPI      = 3
                     , Preserve  = 4
                     }

#[repr(C)]
pub struct MemArea { pub base: u64
                   , pub length: u64
                   , ty: MemAreaType
                   , _pad: u32
                   }

impl MemArea {
    #[inline] pub fn address(&self) -> usize {
        (self.base + self.length - 1) as usize
    }
}

#[allow(raw_pointer_derive)]
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
