use ::memory::VAddr;
use alloc::PAGE_SIZE;

pub mod paddr_impls;
pub use self::paddr_impls::*;

pub mod table;
pub use self::table::{Table, PML4, Entry};
use self::table::{HUGE_PAGE};

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

        let huge_page = || pdpt.and_then(|pdpt| {
            let pdpt_entry = &pdpt[self.pdpt_index()];

            if pdpt_entry.flags().contains(HUGE_PAGE) {
                // If the PDPT entry contains the huge page flag, and the
                // entry points to the start frame of a page, then the pointed
                // frame is a 1GB huge page
                pdpt_entry.pointed_frame()
                    .map(|start_frame| {
                        assert!( start_frame as usize % table::N_ENTRIES == 0
                               , "Start frame must be aligned on a \
                                  1GB boundary!");
                        (start_frame as usize + self.pd_index()
                                              + self.pt_index()) as *mut u8
                    })

            } else {
                pdpt.next_table(self.pdpt_index())
                    .and_then(|pd| {
                        let pd_entry = &pd[self.pd_index()];

                        if pd_entry.flags().contains(HUGE_PAGE) {
                            pd_entry.pointed_frame()
                                .map(|start_frame|{
                                    assert!( (start_frame as usize %
                                             table::N_ENTRIES) == 0
                                         , "Start frame must be aligned!");
                                    (start_frame as usize + self.pt_index())
                                        as *mut u8
                                })
                        } else {
                            None
                        }
                    })
            }
        });

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
