use memory::{VAddr, PAddr};

pub trait Mapper {
    type Page;
    type Flags;

    /// Translates a virtual address to the corresponding physical address.
    ///
    /// # Return
    ///     - `Some(PAddr)` containing the physical address corresponding to
    ///       `vaddr`, if it is mapped.
    ///     - `None`: if the address is not mapped.
    fn translate(&self, vaddr: VAddr) -> Option<PAddr>;
    fn translate_page(&self, page: Self::Page) -> Option<*mut u8>;
}
