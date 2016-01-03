/// A physical address is a 64-bit unsigned integer
use ::memory::VAddr;
use alloc::PAGE_SIZE;

pub type PAddr = u64;
pub mod table;
pub use self::table::{Table, PML4};

pub struct Page { pub number: usize }
pub const N_ENTRIES: usize = 512;
