//! Memory parameters

use memory::PAddr;
use core::ops::Range;
use core::slice::Iter;
/// A memory map is an iterator over memory areas
pub type Map<'a> = Iter<'a, Area>;

/// A memory area
#[derive(Debug, Copy, Clone)]
pub struct Area {
    /// The start address of this memory area
    pub start_addr: PAddr
  , /// The end address of this memory area
    pub end_addr: PAddr
  , /// Whether or not the memory area is usable
    pub is_usable: bool
}
