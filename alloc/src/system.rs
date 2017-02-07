use spin::Mutex;
use super::{Address, Allocator, AllocErr, Layout, AllocResult};
use core::ops::Deref;

#[cfg(feature = "borrow")]
use borrow::{Borrowed, BorrowedPtr};

extern crate params;

use self::params::InitParams;

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

#[cfg(feature = "borrow")]
impl SystemAllocator {

    /// Borrow a raw allocation from the system allocator
    ///
    /// The borrowed allocation handle will automagically deallocate the
    /// allocation at the end of its lifetime
    pub fn borrow_ptr<'alloc>(&'alloc self, layout: Layout)
                      -> AllocResult<BorrowedPtr<'alloc, Tier<'static>>> {
        let ptr = unsafe { self.0.lock().alloc(layout.clone())? };
        Ok(BorrowedPtr::new(ptr, layout, &self.0))
    }

    /// Borrow an object allocation from the system allocator.
    ///
    /// The borrowed allocation handle will automagically deallocate the
    /// allocated object at the end of its lifetime
    pub fn borrow<'alloc, T>(&'alloc self)
                        -> AllocResult<Borrowed<'alloc, Tier<'static>, T>> {
        let value = unsafe { self.0.lock().alloc_one::<T>()? };
        Ok(Borrowed::new(value, &self.0 ))
    }
}
