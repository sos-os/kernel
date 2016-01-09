use super::{elf,Tag};
use core::mem;

#[derive(Debug)]
#[repr(packed)]
pub struct SectionsTag { tag: Tag
                       , pub n_sections: u32
                       , section_size: u32
                       , stringtable_idx: u32
                       , first_section: Section
                       }

impl SectionsTag {
    pub fn sections(&'static self) -> Sections {
        Sections { curr: &self.first_section
                 , remaining: self.n_sections - 1
                 , size: self.section_size
                 }
    }
}

/// Represents an ELF-64 section
#[derive(Debug)]
#[repr(C)]
pub struct Section { name: u32
                   , ty: elf::SectionType
                   , pub flags: u64
                   , pub address: u64
                   , offset: u64
                   , pub length: u64
                   , link: u32
                   , address_align: u32
                   , entry_length: u64
                   }

/// Iterator over ELF64 sections
#[derive(Clone,Debug)]
pub struct Sections { curr: &'static Section
                    , remaining: u32
                    , size: u32
                    }

impl Iterator for Sections {
    type Item = &'static Section;

    fn next(&mut self) -> Option<&'static Section> {
        if self.remaining == 0 {
            None
        } else {
            let current = self.curr;
            self.curr = unsafe {
                &*(((self.curr as *const Section) as u32 + self.size)
                    as *const Section)
            };
            self.remaining -= 1;
            if current.ty == elf::SectionType::Null {
                self.next()
            } else {
                Some(current)
            }
        }
    }
}
