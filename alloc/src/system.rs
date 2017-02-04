use spin::Mutex;
use super::{Allocator, Layout};

extern crate params;

use self::params::InitParams;

use core::ops::{Deref, DerefMut};
use core::ptr::Unique;
#[cfg(feature = "bump_ptr")]
use bump_ptr::BumpPtr;

#[cfg(feature = "buddy")]
use buddy::Heap as BuddyHeap;

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
// TODO: can this allocate pointers to _objects_ rather than `*mut u8`s?
//       - eliza, 1/23/2017
pub struct BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    ptr: Unique<u8>
  , layout: Layout
  , allocator: &'a Mutex<A>
}

impl<'a, A> Deref for BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    type Target = *mut u8;
    fn deref(&self) ->  &Self::Target { &(*self.ptr) }
}
//
// impl<'a, A> ops::DerefMut for BorrowedPtr<'a, A>
// where A: Allocator
//     , A: 'a {
//     fn deref_mut(&mut self) ->  &mut Self::Target { &mut self.frame }
// }

impl<'a, A> Drop for BorrowedPtr<'a, A>
where A: Allocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock().dealloc(*self.ptr, self.layout.clone())
        }
    }
}

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
pub struct Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    value: Unique<T>
  , allocator: &'a Mutex<A>
}

impl<'a, A, T> Deref for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    type Target = T;
    fn deref(&self) ->  &Self::Target { unsafe { self.value.get() } }
}

impl<'a, A, T> DerefMut for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    fn deref_mut(&mut self) ->  &mut Self::Target {
        unsafe { self.value.get_mut() }
    }
}

impl<'a, A, T> Drop for Borrowed<'a, A, T>
where A: Allocator
    , A: 'a {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock()
                .dealloc( *self.value as *mut u8
                        , Layout::for_value(self.value.get()))
        }
    }
}
