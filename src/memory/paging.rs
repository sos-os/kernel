use memory::{VAddr, PAddr};
use arch::memory::{ PAGE_SHIFT, PAGE_SIZE };

// for now - allocator may soon move into the kernel crate.
use alloc::Allocator;

pub trait Mapper {
    type Flags;
    type Frame;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    ///  + `Some(PAddr)` containing the physical address corresponding to
    ///       `vaddr`, if it is mapped.
    ///  + `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;

    fn translate_page(&self, page: Page) -> Option<*mut u8>;

    /// Modifies the page tables so that `page` maps to `frame`.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `frame`: the physical `Frame` that `Page` should map to.
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to<A>( &mut self, page: Page, frame: Self::Frame
                , flags: Self::Flags, alloc: &mut A )
    where A: Allocator;

    /// Identity map a given `frame`.
    ///
    /// # Arguments
    /// + `frame`: the physical `Frame` to identity map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn identity_map<A>( &mut self, frame: Self::Frame
                      , flags: Self::Flags, alloc: &mut A )
    where A: Allocator;

    /// Map the given `page` to any free frame.
    ///
    /// This is like the fire and forget version of `map_to`: we just pick the
    /// first available free frame and map the page to it.
    ///
    /// # Arguments
    /// + `page`: the virtual `Page` to map
    /// + `flags`: the page table entry flags.
    /// + `alloc`: a memory allocator
    fn map_to_any<A>(&mut self, page: Page, flags: Self::Flags, alloc: &mut A)
    where A: Allocator;

}


macro_rules! table_idx {
    ( $($name:ident >> $shift:expr)* ) => {$(
        pub fn $name(&self) -> usize {
            (self.number >> $shift) & 0o777
        }
    )*};
}

/// A virtual page
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Page { pub number: usize }

impl Page {
    /// Create a new `Page` containing the given virtual address.
    //  TODO: rewrite this as `up`/`down` using the page shift, instead.
    pub fn containing(addr: VAddr) -> Page {
        assert!( addr < VAddr::from(0x0000_8000_0000_0000) ||
                 addr >= VAddr::from(0xffff_8000_0000_0000)
               , "invalid address: 0x{:x}", addr );
        Page { number: addr.as_usize() / PAGE_SIZE as usize }
    }

    /// Return the start virtual address of this page
    #[inline]
    pub fn base_addr(&self) -> VAddr {
        VAddr::from(self.number << PAGE_SHIFT)
    }

    /// Flush the page from memory
    pub unsafe fn flush(&self) {
        asm!( "invlpg [$0]"
            :
            : "{rax}"(self.base_addr())
            : "memory"
            : "intel", "volatile")
    }

    table_idx!{
        pml4_index >> 27
        pdpt_index >> 18
        pd_index >> 9
        pt_index >> 0
    }

}
