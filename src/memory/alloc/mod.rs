
/// An `Allocator` implements a particular memory allocation strategy.
pub trait Allocator {

    /// Allocate a new block of size `size` on alignment `align`.
    ///
    /// # Arguments:
    ///   - `size`: the amount of memory to allocate (in bytes)
    ///   - `align`: the alignment for the allocation request
    ///
    /// # Returns:
    ///   - `Some(*mut u8)` if the request was allocated successfully
    ///   - `None` if the allocator is out of memory or if the request was
    ///     invalid.
    unsafe fn allocate( &mut self
                      , size: usize
                      , align: usize)
                      -> Option<*mut u8>;

    /// Release an allocated block of memory.
    ///
    /// The `size` and `align` parameters _must_ be the same as the original
    /// size and alignment of the frame being deallocated, otherwise our
    /// heap will become corrupted.
    ///
    /// # Arguments:
    ///   - `frame`: a pointer to the block of memory to deallocate
    ///   - `size`: the size of the block being deallocated
    ///   - `align`: the alignment of the block being deallocated
    unsafe fn deallocate( &mut self
                        , frame: *mut u8
                        , size: usize, align: usize);

    /// Reallocate `old_frame` from `old_size` bytes to `new_size` bytes
    ///
    /// The `old_size` and `align` parameters _must_ be the same as the
    /// original size and alignment of the frame being reallocated, otherwise
    /// our heap will become corrupted.
    ///
    /// # Arguments:
    ///   - `old_frame`: a pointer to the frame to be reallocated
    ///   - `old_size`: the size (in bytes) of the frame being reallocated
    ///   - `new_size`: the size to reallocate the frame to.
    ///   - `align`: the alignment for the allocation request
    ///
    /// # Returns:
    ///   - `Some(*mut u8)` if the frame was reallocated successfully
    ///   - `None` if the allocator is out of memory or if the request was
    ///     invalid.
    // TODO: Optimization: check if the reallocation request fits in
    // the old frame and return immediately if it does
    unsafe fn reallocate( &mut self
                        , old_frame: *mut u8
                        , old_size: usize
                        , new_size: usize
                        , align: usize )
                        -> Option<*mut u8> {
        // First, attempt to allocate a new frame...
        self.allocate(new_size, align)
            .map(|new_frame| {
                // If a new frame was allocated, copy all the data from the
                // old frame into the new frame.
                ptr::copy(new_frame, old_frame, min(old_size, new_size));
                // Then we can deallocate the old frame
                self.deallocate(old_frame, old_size, align);
                new_frame
            })
    }

    unsafe fn zero_alloc( &mut self
                        , _size: usize
                        , _align: usize)
                        -> Option<*mut u8> {
        unimplemented!()
    }
}
