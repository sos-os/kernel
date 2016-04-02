use multiboot2::Tag;
use ::elf::section;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub type Section = section::Header;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
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
            if current.ty == section::Type::Null {
                self.next()
            } else {
                Some(current)
            }
        }
    }
}
