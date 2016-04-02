#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub mod elf64;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub type Section = elf64::Section;

#[cfg(any(target_arch = "x86_64", target_arch = "armv7"))]
pub type SectionsTag = elf64::SectionsTag;
