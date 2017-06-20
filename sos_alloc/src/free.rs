use core::ptr::Unique;
use core::mem;

use intrusive::stack::{Stack, Node};
use intrusive::rawlink::RawLink;

/// A `FreeList` is a list of unique free blocks
pub type List = Stack<Unique<Block>, Block>;

/// A free block header stores a pointer to the next and previous free blocks.
///
/// A `Block` can be any size, as long as
pub struct Block { next: RawLink<Block> }
impl Block {
    #[inline] pub unsafe fn as_ptr(&self) -> *mut u8 { mem::transmute(self) }
}

impl Node for Block {
    #[inline] fn next(&self) -> &RawLink<Block> {
        &self.next
    }
    #[inline] fn next_mut(&mut self) -> &mut RawLink<Block> {
        &mut self.next
    }
}
