//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! An intrusive singly-linked list implementation using `RawLink`s.
//!
//! An _intrusive_ list is a list structure wherein the type of element stored
//! in the list holds references to other nodes. This means that we don't have
//! to store a separate node data type that holds the stored elements and
//! pointers to other nodes, reducing the amount of memory allocated. We can
//! use intrusive lists in code that runs without the kernel memory allocator,
//! like the allocator implementation itself, since each list element manages
//! its own memory.
use ::{RawLink, OwnedRef};

use core::marker::PhantomData;
use core::iter;
#[cfg(test)] mod test;


/// This trait defines a node in an intrusive list.
///
/// A Node must be capable of providing mutable and immutable references to
/// the next node in the stack
pub trait Node: Sized {
    fn next(&self) -> &RawLink<Self>;
    fn next_mut(&mut self) -> &mut RawLink<Self>;
}

/// The `Stack` struct is our way of interacting with an intrusive list.
///
/// It stores a pointer to the head of the stack, the length of the
/// list, and a `PhantomData` marker for the list's `OwnedRef` type. It
/// provides the methods for pushing, popping, and indexing the list.
pub struct Stack<T, N>
where T: OwnedRef<N>
    , N: Node {
    head: RawLink<N>
  , _ty_marker: PhantomData<T>
  , length: usize
 }

impl<T, N> Stack<T, N>
where T: OwnedRef<N>
    , N: Node {

    /// Construct a new `Stack<T, N>` with zero elements
    pub const fn new() -> Self {
        Stack { head: RawLink::none()
             , _ty_marker: PhantomData
             , length: 0 }
    }

    /// Returns the length of the list
    #[inline] pub fn len(&self) -> usize {
        self.length
    }

    /// Borrows the first element of the list as an `Option`
    ///
    /// # Returns
    ///   - `Some(&N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn peek(&self) -> Option<&N> {
        unsafe { self.head.resolve() }
    }


    /// Mutably borrows the first element of the list as an `Option`
    ///
    /// # Returns
    ///   - `Some(&mut N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn peek_mut(&mut self) -> Option<&mut N> {
        unsafe { self.head.resolve_mut() }
    }

    /// Returns true if the list is empty.
    #[inline] pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push an element to the front of the stack
    pub fn push(&mut self, mut item: T) {
        // set the pushed item to point to the head element of the stack
        *item.get_mut().next_mut() = self.head.take();
        // then, set this node's head pointer to point to the pushed item
        self.head = RawLink::some(item.get_mut());
        unsafe { item.take(); };
        self.length += 1;
    }

    /// Removes and returns the element at the front of the list.
    ///
    /// # Returns
    ///   - `Some(T)` containing the element at the front of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            self.head.take().resolve_mut()
                .map(|head| {
                    if let Some(next) = head.next_mut().resolve_mut() {
                        self.head = RawLink::some(next);
                    }
                    self.length -= 1;
                    T::from_raw(head)
                })
        }
    }

}

impl<T, N> iter::FromIterator<T> for Stack<T, N>
where T: OwnedRef<N>
    , N: Node {
    fn from_iter<I: IntoIterator<Item=T>>(iterator: I) -> Self {
        let mut list: Self = Stack::new();
        for item in iterator { list.push(item) }
        list
    }
}

impl<T, N> iter::Extend<T> for Stack<T, N>
where T: OwnedRef<N>
    , N: Node {

    fn extend<I: IntoIterator<Item=T>>(&mut self, iterator: I) {
        for item in iterator { self.push(item) }
    }
}
