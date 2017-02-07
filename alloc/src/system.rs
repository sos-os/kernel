use spin::Mutex;
use super::{Address, Allocator, AllocErr, Layout, AllocResult};

extern crate params;

use self::params::InitParams;

use core::ops::{Deref, DerefMut};
use core::ptr::Unique;
#[cfg(feature = "bump_ptr")]
use bump_ptr::BumpPtr;

#[cfg(feature = "buddy")]
use buddy::Heap as BuddyHeap;

pub enum Tier<'a> {
    Uninitialized
    , #[cfg(feature = "bump_ptr")]
      Bump(BumpPtr)
    , #[cfg(feature = "buddy")]
      Buddy(BuddyHeap<'a>)
}
#[cfg(all(feature = "bump_ptr", feature="buddy"))]
impl Deref for Tier<'static> {
    type Target = Allocator + 'static ;
    fn deref(&self) -> &Self::Target{
        match self {
            &Tier::Bump(ref alloc) => alloc
          , &Tier::Buddy(ref alloc) => alloc
          , _ => panic!("no allocator!")
        }
    }
}

#[cfg(all(feature = "bump_ptr", feature = "buddy"))]
unsafe impl<'a> Allocator for Tier<'a> {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> AllocResult<Address> {
        match *self {
            Tier::Bump(ref mut alloc) => alloc.alloc(layout)
          , Tier::Buddy(ref mut alloc) => alloc.alloc(layout)
          , _ => Err(AllocErr::Unsupported {
                    details: "System allocator uninitialized!"
                })
        }
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, ptr: Address, layout: Layout) {
        match *self {
            Tier::Bump(ref mut alloc) => alloc.dealloc(ptr, layout)
          , Tier::Buddy(ref mut alloc) => alloc.dealloc(ptr, layout)
          , _ =>  {
              // just leak it? not sure if we should panic here...
          }
        }
    }

}

pub struct SystemAllocator(Mutex<Tier<'static>>);

impl SystemAllocator {

    /// Borrow a raw allocation from the system allocator
    ///
    /// The borrowed allocation handle will automagically deallocate the
    /// allocation at the end of its lifetime
    pub fn borrow_ptr<'alloc>(&'alloc self, layout: Layout)
                      -> AllocResult<BorrowedPtr<'alloc, Tier<'static>>> {
        let ptr = unsafe { self.0.lock().alloc(layout.clone())? };
        Ok(BorrowedPtr { ptr: unsafe { Unique::new(ptr) }
                       , layout: layout
                       , allocator: &self.0 })
    }

    /// Borrow an object allocation from the system allocator.
    ///
    /// The borrowed allocation handle will automagically deallocate the
    /// allocated object at the end of its lifetime
    pub fn borrow<'alloc, T>(&'alloc self)
                        -> AllocResult<Borrowed<'alloc, Tier<'static>, T>> {
        let value = unsafe { self.0.lock().alloc_one::<T>()? };
        Ok(Borrowed { value: value
                    , allocator: &self.0 })
    }
}

/// A borrowed handle on a heap allocation with a specified lifetime.
///
/// This automatically deallocates the allocated object when the borrow's
/// lifetime ends. It also ensures that the borrow only lives as long as the
/// allocator that provided it, and that the borrow is dropped if the allocator
/// is dropped.
pub struct BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
    ptr: Unique<u8>
  , layout: Layout
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A> Deref for BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
    type Target = *mut u8;
    fn deref(&self) ->  &Self::Target { &(*self.ptr) }
}
//
// impl<'alloc, A> ops::DerefMut for BorrowedPtr<'alloc, A>
// where A: Allocator
//     , A: 'alloc {
//     fn deref_mut(&mut self) ->  &mut Self::Target { &mut self.frame }
// }

impl<'alloc, A> Drop for BorrowedPtr<'alloc, A>
where A: Allocator
    , A: 'alloc {
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
pub struct Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    value: Unique<T>
  , allocator: &'alloc Mutex<A>
}

impl<'alloc, A, T> Deref for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    type Target = T;
    fn deref(&self) ->  &Self::Target { unsafe { self.value.get() } }
}

impl<'alloc, A, T> DerefMut for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    fn deref_mut(&mut self) ->  &mut Self::Target {
        unsafe { self.value.get_mut() }
    }
}

impl<'alloc, A, T> Drop for Borrowed<'alloc, A, T>
where A: Allocator
    , A: 'alloc {
    fn drop(&mut self) {
        unsafe {
            self.allocator.lock()
                .dealloc( *self.value as *mut u8
                        , Layout::for_value(self.value.get()))
        }
    }
}
