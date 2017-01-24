/// This module contains the static GDT and its pointer.
///
/// These have to live in the main kernel crate rather than in `cpu`,
/// since they need to be statically linked to the boot crate.
use cpu::segment::*;
use cpu::dtable;

#[linkage = "external"]
#[link_name = ".gdt64"]
pub static GDT: Gdt
// todo: make not ugly
    = [ Descriptor::null()
      , /// 64-bit code segment descriptor
        #[link_name = ".gdt64.code"]
        Descriptor { base_high: 0
                   , flags: Flags::from_raw(0b1001100000100000)
                   , base_mid: 0
                   , base_low: 0
                   , limit: 0}
      , /// Data segment descriptor
        #[link_name = ".gdt64.data"]
        Descriptor { base_high: 0
                   , flags: Flags::from_raw(0b1001001011001111)
                   , base_mid: 0
                   , base_low: 0
                   , limit: 0xFFFF}
      ];

#[linkage = "external"]
#[link_name = ".gdt64.ptr"]
pub static GDT_PTR: dtable::Pointer<Gdt> =
    dtable::Pointer { limit: 24
                    , base: unsafe { &GDT as *const Gdt}
                     };
