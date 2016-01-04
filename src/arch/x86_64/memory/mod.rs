use ::memory::VAddr;
use alloc::PAGE_SIZE;

pub mod paddr;
pub use self::paddr::PAddr;

pub mod table;
pub use self::table::{Table, PML4};

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
}
