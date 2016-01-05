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

use core::ops::DerefMut;
use core::marker::PhantomData;

#[cfg(test)] mod test;

pub unsafe trait OwnedPtr: DerefMut {
    unsafe fn from_raw(ptr: *mut Self::Target) -> Self;
    unsafe fn take(self);
}

pub trait Node: Sized {
    fn next(&self) -> &RawLink<Self>;
    fn prev(&self) -> &RawLink<Self>;

    fn next_mut(&mut self) -> &mut RawLink<Self>;
    fn prev_mut(&mut self) -> &mut RawLink<Self>;
}

pub struct ListNode<T, N>
where T: OwnedPtr<Target=N>
    , N: Node {
    head: RawLink<N>
  , tail: RawLink<N>
  , _ty_marker: PhantomData<T>
 }

 // impl<T> Node for ListNode<T>
 // where T: OwnedPtr
 //     , T: Node {
 //
 //    fn next(&self) -> &RawLink<Self> { &self.head }
 //    fn prev(&self) -> &RawLink<Self> { &self.tail }
 //
 //    fn next_mut(&mut self) -> &mut RawLink<Self> { self.head }
 //    fn prev_mut(&mut self) -> &mut RawLink<Self> { self.tail }
 // }
impl<T, N> ListNode<T, N>
where T: OwnedPtr<Target=N>
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
                    *item.next_mut() = RawLink::none();
                    *item.prev_mut() = RawLink::none();
                    self.tail = RawLink::some(item.deref_mut());
                }
              , Some(head) => {
                    // If this node is not empty, set the pushed item's tail
                    // to point at the head node, and make the head node's tail
                    // point to the pushed item
                    *item.next_mut() = RawLink::none();
                    *item.prev_mut() = RawLink::some(head);
                    *head.prev_mut() = RawLink::some(item.deref_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.head = RawLink::some(item.deref_mut());
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
                    *item.next_mut() = RawLink::none();
                    *item.prev_mut() = RawLink::none();
                    self.head = RawLink::some(item.deref_mut());
                }
              , Some(tail) => {
                    // If this node is not empty, set the pushed item's head
                    // to point at the tail node, and make the tail node's head
                    // point to the pushed item
                    *item.next_mut() = RawLink::some(tail);
                    *item.prev_mut() = RawLink::none();
                    *tail.next_mut() = RawLink::some(item.deref_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.tail = RawLink::some(item.deref_mut());
            item.take()
        }
    }
}
