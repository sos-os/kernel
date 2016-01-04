use core::fmt;

use ::memory::VAddr;
use alloc::PAGE_SIZE;

pub mod table;
pub use self::table::{Table, PML4};

pub struct Page { pub number: usize }

impl Page {
    /// Create a new `Page` containing the given virtual address
    pub fn containing_addr(addr: VAddr) -> Page {
        assert!( addr.as_usize() < 0x0000_8000_0000_0000 ||
                 addr.as_usize() >= 0xffff_8000_0000_0000
               , "invalid address: 0x{:x}", addr );
        Page { number: addr.as_usize() / PAGE_SIZE }
    }
    /// Return the start virtual address of this page
    pub fn start_addr(&self) -> VAddr {
        VAddr::from_usize(self.number * PAGE_SIZE)
    }
}

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

impl fmt::Binary for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
