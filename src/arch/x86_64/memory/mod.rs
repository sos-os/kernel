use ::memory::VAddr;
use alloc::PAGE_SIZE;

pub mod paddr_impls;
pub use self::paddr_impls::*;

pub mod table;
pub use self::table::{Table, PML4};

/// A physical (linear) memory address is a 64-bit unsigned integer
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);

impl PAddr {
    #[inline] pub const fn from_u64(u: u64) -> Self {
        PAddr(u)
    }
    #[inline] pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

trait Translate {
    type To;
    fn translate(&self) -> Option<Self::To>;
}

impl Translate for VAddr {
    type To = PAddr;

    fn translate(&self) -> Option<PAddr> {
        Page::containing_addr(*self)
            .translate()
            .map(|frame| {
                let offset = self.as_usize() % PAGE_SIZE;
                PAddr::from_u64(frame as u64 + offset as u64)
            })
    }
}

impl Translate for Page {
    type To = *mut u8;

    fn translate(&self) -> Option<*mut u8> {
        let pdpt = unsafe { &*table::PML4 }.next_table(self.pml4_index());

        let huge_page = || {
            unimplemented!()
        };

        pdpt.and_then(|pdpt| pdpt.next_table(self.pdpt_index()))
            .and_then(|pd| pd.next_table(self.pd_index()))
            .and_then(|pt| pt[self.pt_index()].pointed_frame())
            .or_else(huge_page)

    }
}

macro_rules! table_idx {
    ( $($name:ident >> $shift:expr)* ) => {$(
        pub fn $name(&self) -> usize {
            (self.number >> $shift) & 0o777
        }
    )*};
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Page { pub number: usize }

impl Page {
    /// Create a new `Page` containing the given virtual address
    pub fn containing_addr(addr: VAddr) -> Page {
        assert!( addr.as_usize() < 0x0000_8000_0000_0000 ||
                 addr.as_usize() >= 0xffff_8000_0000_0000
               , "invalid address: 0x{:x}", addr );
        Page { number: (addr / PAGE_SIZE).as_usize() }
    }

    /// Return the start virtual address of this page
    pub fn start_addr(&self) -> VAddr {
        VAddr::from_usize(self.number * PAGE_SIZE)
    }

    table_idx!{
        pml4_index >> 27
        pdpt_index >> 18
        pd_index >> 9
        pt_index >> 0
    }


}
