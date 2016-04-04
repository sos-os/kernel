//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use multiboot2::Tag;

use elf::section::{Sections, HeaderRepr};

#[cfg(target_pointer_width = "32")]
pub type Word = u32;
#[cfg(target_pointer_width = "64")]
pub type Word = u64;

#[derive(Debug)]
#[repr(packed)]
pub struct SectionsTag { tag: Tag
                       , pub n_sections: u32
                       , section_size: u32
                       , stringtable_idx: u32
                       , first_section: HeaderRepr<Word>
                       }

impl SectionsTag {
    pub fn sections(&'static self) -> Sections<'static, Word> {
        Sections::new( &self.first_section
                     , self.n_sections - 1
                     , self.section_size
                     )
    }
}
