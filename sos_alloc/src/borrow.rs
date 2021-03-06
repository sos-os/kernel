use super::{Address, Allocator, Layout};
use ptr::Unique;
use ops::{Deref, DerefMut};
use spin::Mutex;

pub trait Lender {
    type Borrowed;
    fn borrow(&self) -> Self::Borrowed;
}

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
// TODO: can we replace this with one generic implementation that works for
//       borrowed pointers, objects, and arrays? can that code be reused for
//       BorrowedFrame/BorrowedFrameRange as well?
//          - eliza, 02/21/2017
pub struct BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
    ptr: Unique<u8>
  , layout: Layout
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A> BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {

    #[inline]
    pub fn new( ptr: Address
                    , layout: Layout
                    , allocator: &'alloc Mutex<A>)
                    -> Self {
        BorrowedPtr { ptr: unsafe { Unique::new(ptr) }
                    , layout: layout
                    , allocator: allocator
                    }

    }
}

impl<'alloc, A> Deref for BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
    type Target = Unique<u8>;
    fn deref(&self) ->  &Self::Target { &self.ptr }
}

impl<'alloc, A> Drop for BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock().dealloc(self.ptr.as_ptr(), self.layout.clone())
        }
    }
}

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
pub struct Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    value: Unique<T>
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A, T> Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {

    #[inline]
    pub fn new( value: Unique<T>, allocator: &'alloc Mutex<A>)
                    -> Self {
        Borrowed { value: value
                 , allocator: allocator
                 }

    }
}

impl<'alloc, A, T> Deref for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    type Target = T;
    fn deref(&self) ->  &Self::Target { unsafe { self.value.as_ref() } }
}

impl<'alloc, A, T> DerefMut for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    fn deref_mut(&mut self) ->  &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<'alloc, A, T> Drop for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    fn drop(&mut self) {
        use mem::drop;
        let address = self.value.as_ptr() as Address;
        // ensure we drop the object _before_ deallocating it, so that
        // the object's destructor gets run first
        // i hope this is correct...
        drop(self.value.as_ptr());
        unsafe {
            self.allocator.lock()
                .dealloc( address
                        , Layout::for_value(self.value.as_ref()))
        }
    }
}
