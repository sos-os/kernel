use core::fmt;

/// A virtual address is a machine-sized unsigned integer
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct VAddr(usize);

impl fmt::Debug for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}


/// A physical address is a 64-bit nsigned integer
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct PAddr(u64);

impl fmt::Debug for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

// pub struct
