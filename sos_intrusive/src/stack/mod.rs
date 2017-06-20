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

    /// Searches for and removes the first element matching a predicate.
    ///
    /// # Arguments
    ///   - `p`: A predicate (function of the form `&N -> bool`) that returns
    ///     true if the element should be removed and false if it should not.
    ///
    /// # Returns
    ///   - `Some(T)` if an element matching the predicate was found
    ///   - `None` if no elements matched the predicate (or if the list is
    ///     empty.)
    #[inline]
    pub fn find_and_remove<P>(&mut self, predicate: P) -> Option<T>
    where P: Fn(&N) -> bool {
        self.into_iter().find_and_remove(predicate)
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

pub struct Iter<'a, N>
where N: Node + 'a {
    current: Option<&'a N>
}

impl<'a, N> Iterator for Iter <'a, N>
where N: Node + 'a {

    type Item = &'a N;

    fn next(&mut self) -> Option<Self::Item> {
        self.current
            .map(|curr| {
                self.current = unsafe { curr.next().resolve() };
                curr
            })
    }
}

impl <'a, T, N> IntoIterator for &'a Stack<T, N>
where T: OwnedRef<N> + 'a
    , N: Node + 'a {

    type IntoIter = Iter<'a, N>;
    type Item = &'a N;

    fn into_iter(self) -> Self::IntoIter {
        Iter { current: self.peek() }
    }
}

pub struct IterMut<'a, T, N>
where N: Node + 'a
    , T: OwnedRef<N> + 'a {
    current: RawLink<N>
  , _lifetime: PhantomData<&'a T>
}

impl<'a, T, N> IterMut<'a, T, N>
where N: Node + 'a
    , T: OwnedRef<N> + 'a {
    /// Removes the element currently under the iterator and returns it.
    ///
    /// # Returns
    ///   - `Some(T)` if the there is an element currently under the cursor
    ///     (i.e., the list is not empty)
    ///   - `None` if the list is empty.
    pub fn remove(&mut self) -> Option<T> {
        unsafe {
            self.current.resolve_mut()
                .and_then(|curr| {
                    curr.next_mut().take().resolve_mut()
                        .map(|result| {
                            *curr.next_mut() =
                                result.next_mut().resolve_mut()
                                      .map(RawLink::some)
                                      .unwrap_or_else(RawLink::none);
                            T::from_raw(result)
                        })
                })
        }
    }

    fn peek_next(&self) -> Option<&N> {
        unsafe {
            self.current.resolve()
                .and_then(|curr| curr.next().resolve())
        }
    }

    /// Searches for and removes the first element matching a predicate.
    ///
    /// # Arguments
    ///   - `p`: A predicate (function of the form `&N -> bool`) that returns
    ///     true if the element should be removed and false if it should not.
    ///
    /// # Returns
    ///   - `Some(T)` if an element matching the predicate was found
    ///   - `None` if no elements matched the predicate (or if the list is
    ///     empty.)
    pub fn find_and_remove<P>(&mut self, predicate: P) -> Option<T>
    where P: Fn(&N) -> bool {
        // TODO: the implementation of this is somewhat ugly, it would
        //       be nice if it was a little bit less imperative...
        let mut found = false;
        while let Some(next) = self.peek_next() {
            if predicate(next) {
                found = true;
                break;
            }
        }
        if found {
            self.remove()
        } else {
            None
        }
    }
}

impl<'a, T, N> Iterator for IterMut<'a, T, N>
where N: Node + 'a
    , T: OwnedRef<N> + 'a {

    type Item = &'a mut N;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.current.take().resolve_mut()
                .map(|curr| {
                    self.current = match curr.next_mut().resolve_mut() {
                        None => RawLink::none()
                      , Some(other_thing) => RawLink::some(other_thing)
                    };
                    curr
                })
            }

    }
}

impl <'a, T, N> IntoIterator for &'a mut Stack<T, N>
where T: OwnedRef<N> + 'a
    , N: Node + 'a {

    type IntoIter = IterMut<'a, T, N>;
    type Item = &'a mut N;

    fn into_iter(self) -> Self::IntoIter {
        let curr = match self.peek_mut() {
            None => RawLink::none()
          , Some(other_thing) => RawLink::some(other_thing)
        };
        IterMut { current: curr
                , _lifetime: PhantomData }
     }
}
