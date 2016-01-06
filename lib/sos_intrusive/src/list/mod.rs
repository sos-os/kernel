//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! A linked-list implementation using `RawLink`s.
use super::rawlink::RawLink;

use core::mem;
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;
use core::intrinsics::forget;
use core::ptr::Unique;
#[cfg(test)] mod test;

pub unsafe trait OwnedRef<T> {
    unsafe fn from_raw(ptr: *mut T) -> Self;
    unsafe fn take(self);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait Node: Sized {
    fn next(&self) -> &RawLink<Self>;
    fn prev(&self) -> &RawLink<Self>;

    fn next_mut(&mut self) -> &mut RawLink<Self>;
    fn prev_mut(&mut self) -> &mut RawLink<Self>;
}

pub struct ListNode<T, N>
where T: OwnedRef<N>
    , N: Node {
    head: RawLink<N>
  , tail: RawLink<N>
  , _ty_marker: PhantomData<T>
 }

 // impl<T> Node for ListNode<T>
 // where T: OwnedRef
 //     , T: Node {
 //
 //    fn next(&self) -> &RawLink<Self> { &self.head }
 //    fn prev(&self) -> &RawLink<Self> { &self.tail }
 //
 //    fn next_mut(&mut self) -> &mut RawLink<Self> { self.head }
 //    fn prev_mut(&mut self) -> &mut RawLink<Self> { self.tail }
 // }
impl<T, N> ListNode<T, N>
where T: OwnedRef<N>
    , N: Node {

    pub const fn new() -> Self {
        ListNode { head: RawLink::none()
                 , tail: RawLink::none()
                 , _ty_marker: PhantomData }
    }

    pub fn front(&self) -> Option<&N> {
        unsafe { self.head.resolve() }
    }

    pub fn back(&self) -> Option<&N> {
        unsafe { self.tail.resolve() }
    }

    pub fn front_mut(&mut self) -> Option<&mut N> {
        unsafe { self.head.resolve_mut() }
    }

    pub fn back_mut(&mut self) -> Option<&mut N> {
        unsafe { self.tail.resolve_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn push_front(&mut self, mut item: T) {
        unsafe {
            match self.head.resolve_mut() {
                None => {
                    // If this node's head is empty, set the pushed item's
                    // links to None, and make this node's tail point to the
                    // pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::none();
                    self.tail = RawLink::some(item.get_mut());
                }
              , Some(head) => {
                    // If this node is not empty, set the pushed item's tail
                    // to point at the head node, and make the head node's tail
                    // point to the pushed item
                    *item.get_mut().next_mut() = RawLink::some(head);
                    *item.get_mut().prev_mut() = RawLink::none();
                    *head.prev_mut() = RawLink::some(item.get_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.head = RawLink::some(item.get_mut());
            item.take()
        }
    }

    pub fn push_back(&mut self, mut item: T) {
        unsafe {
            match self.tail.resolve_mut() {
                None => {
                    // If this node's tail is empty, set the pushed item's
                    // links to  None, and make this node's head point to the
                    // pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::none();
                    self.head = RawLink::some(item.get_mut());
                }
              , Some(tail) => {
                    // If this node is not empty, set the pushed item's head
                    // to point at the tail node, and make the tail node's head
                    // point to the pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::some(tail);
                    *tail.next_mut() = RawLink::some(item.get_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.tail = RawLink::some(item.get_mut());
            item.take()
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.head.take().resolve_mut()
                .map(|head| {
                    // mem::swap( &mut self.head
                    //          , head.next_mut().resolve_mut()
                    //                .map(|next| next.prev_mut())
                    //                .unwrap_or(&mut RawLink::none()) );
                    match head.next_mut().resolve_mut() {
                        None => self.tail = RawLink::none()
                      , Some(next) => {
                            *next.prev_mut() = RawLink::none();
                            self.head = RawLink::some(next);
                        }
                    }
                    T::from_raw(head)
                })
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.take().resolve_mut()
                .map(|tail| {
                    mem::swap( &mut self.tail
                             , tail.prev_mut().resolve_mut()
                                   .map(|prev| prev.next_mut())
                                   .unwrap_or(&mut RawLink::none()) );
                    T::from_raw(tail)
                })
        }
    }
}
//
// unsafe impl<T> OwnedRef for Unique<T> where T: Node {
//
//     #[inline]
//     fn take(self) {}
//
//     unsafe fn from_raw(ptr: *mut T) -> Self {
//         Unique::new(ptr)
//     }
// }
//
// unsafe impl<'a, T> OwnedRef<T> for &'a mut T {
//     #
//     #[inline] unsafe fn from_raw(raw: *mut T) -> &'a mut T {
//         &mut *raw
//     }
//
//     #[inline] unsafe fn take(self) {
//         forget(self);
//     }
// }
//

unsafe impl<T> OwnedRef<T> for Unique<T>  {
    #[inline]
    fn get(&self) -> &T { unsafe { self.get() } }

    #[inline]
    fn get_mut(&mut self) -> &mut T { unsafe { self.get_mut() } }

    #[inline]
    unsafe fn take(self) {}

    unsafe fn from_raw(ptr: *mut T) -> Self {
        Unique::new(ptr)
    }
}

#[cfg(any(test, feature = "use-std"))]
unsafe impl<T> OwnedRef<T> for ::std::boxed::Box<T> {

    fn get(&self) -> &T { &**self }
    fn get_mut(&mut self) -> &mut T { &mut **self }

    #[inline] unsafe fn take(self) {
        ::std::boxed::Box::into_raw(self);
    }

    unsafe fn from_raw(ptr: *mut T) -> Self {
        ::std::boxed::Box::from_raw(ptr)
    }
}
