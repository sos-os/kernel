//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! An intrusive linked list implementation using `RawLink`s.
//!
//! An _intrusive_ list is a list structure wherein the type of element stored
//! in the list holds references to other nodes. This means that we don't have
//! to store a separate node data type that holds the stored elements and
//! pointers to other nodes, reducing the amount of memory allocated. We can
//! use intrusive lists in code that runs without the kernel memory allocator,
//! like the allocator implementation itself, since each list element manages
//! its own memory.
use super::rawlink::RawLink;

use core::marker::PhantomData;
use core::ptr::Unique;
use core::iter;
#[cfg(test)] mod test;

pub unsafe trait OwnedRef<T> {
    unsafe fn from_raw(ptr: *mut T) -> Self;
    unsafe fn take(self);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

/// This trait defines a node in an intrusive list.
///
/// A Node must be capable of providing mutable and immutable references to
/// the previous and next nodes in the list.
pub trait Node: Sized {
    fn next(&self) -> &RawLink<Self>;
    fn prev(&self) -> &RawLink<Self>;

    fn next_mut(&mut self) -> &mut RawLink<Self>;
    fn prev_mut(&mut self) -> &mut RawLink<Self>;
}

/// The `List` struct is our way of interacting with an intrusive list.
///
/// It stores a pointer to the head and tail of the list, the length of the
/// list, and a `PhantomData` marker for the list's `OwnedRef` type. It
/// provides the methods for pushing, popping, and indexing the list.
pub struct List<T, N>
where T: OwnedRef<N>
    , N: Node {
    head: RawLink<N>
  , tail: RawLink<N>
  , _ty_marker: PhantomData<T>
  , length: usize
 }

 // impl<T> Node for List<T>
 // where T: OwnedRef
 //     , T: Node {
 //
 //    fn next(&self) -> &RawLink<Self> { &self.head }
 //    fn prev(&self) -> &RawLink<Self> { &self.tail }
 //
 //    fn next_mut(&mut self) -> &mut RawLink<Self> { self.head }
 //    fn prev_mut(&mut self) -> &mut RawLink<Self> { self.tail }
 // }
impl<T, N> List<T, N>
where T: OwnedRef<N>
    , N: Node {

    /// Construct a new `List<T, N>` with zero elements
    pub const fn new() -> Self {
        List { head: RawLink::none()
             , tail: RawLink::none()
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
    #[inline] pub fn front(&self) -> Option<&N> {
        unsafe { self.head.resolve() }
    }


    /// Borrows the last element of the list as an `Option`
    ///
    /// # Returns
    ///   - `Some(&N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn back(&self) -> Option<&N> {
        unsafe { self.tail.resolve() }
    }

    /// Mutably borrows the first element of the list as an `Option`
    ///
    /// # Returns
    ///   - `Some(&mut N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn front_mut(&mut self) -> Option<&mut N> {
        unsafe { self.head.resolve_mut() }
    }

    /// Mutably borrows the last element of the list as an `Option`
    ///
    /// # Returns
    ///   - `Some(&mut N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn back_mut(&mut self) -> Option<&mut N> {
        unsafe { self.tail.resolve_mut() }
    }

    /// Returns true if the list is empty.
    #[inline] pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push an element to the front of the list.
    // TODO: should this really be called "prepend"?
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
            item.take();
            self.length += 1;
        }
    }

    /// Push an element to the back of the list.
    //  TODO: should this really be called "append"?
    //  (the Rust standard library uses `append` to refer to the "drain all the
    //  elements of another list and push them to this list" operation, but I
    //  think that that function is more properly called `concat`...)
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
            item.take();
            self.length += 1;
        }
    }

    /// Removes and returns the element at the front of the list.
    ///
    /// # Returns
    ///   - `Some(T)` containing the element at the front of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
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
                    self.length -= 1;
                    T::from_raw(head)
                })
        }
    }

    /// Removes and returns the element at the end of the list.
    ///
    /// # Returns
    ///   - `Some(T)` containing the element at the end of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.take().resolve_mut()
                .map(|tail| {
                    match tail.prev_mut().resolve_mut() {
                        None => self.head = RawLink::none()
                      , Some(prev) => {
                            *prev.next_mut() = RawLink::none();
                            self.tail = RawLink::some(prev);
                        }
                    }
                    self.length -= 1;
                    T::from_raw(tail)
                })
        }
    }

    /// Borrows the element at the front of the list
    ///
    /// # Returns
    ///   - `Some(&T)` containing the element at the end of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn peek_front(&self) -> Option<&N> {
        unsafe { self.tail.resolve() }
    }

    /// Returns a cursor for iterating over or modifying the list.
    pub fn cursor_mut<'a>(&'a mut self) -> ListCursorMut<'a, T, N> {
        ListCursorMut { list: self
                      , current: RawLink::none() }
    }

}

impl<T, N> iter::FromIterator<T> for List<T, N>
where T: OwnedRef<N>
    , N: Node {
        fn from_iter<I: IntoIterator<Item=T>>(iterator: I) -> Self {
            let mut list: Self = List::new();
            for item in iterator { list.push_front(item) }
            list
        }
}

pub trait Cursor {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
    fn prev(&mut self) -> Option<Self::Item>;
    fn get(&self) -> Option<Self::Item>;
    fn seek_forward(&mut self, n: usize) -> Option<Self::Item>;
    fn seek_backward(&mut self, n: usize) -> Option<Self::Item>;

}

/// A cursor for an intrusive linked list.
///
/// A cursor functions similarly to an iterator, except that it can seek back
/// and forth rather than just advancing through the list, and it can mutate
/// the element "under" the cursor.
///
/// A cursor begins before the first element in the list, and once it has been
/// advanced past the last element of the list, it "loops around" back to the
/// first element.
// TODO: can we implement `Iterator` for cursors?
pub struct ListCursorMut<'a, T, N>
where T: OwnedRef<N>
    , T: 'a
    , N: Node
    , N: 'a {
        list: &'a mut List<T, N>
      , current: RawLink<N>
}

impl<'a, T, N> ListCursorMut<'a, T, N>
where T: OwnedRef<N>
    , T: 'a
    , N: Node
    , N: 'a {

    /// Advances the cursor to the next element and borrows it mutably.
    ///
    /// If the cursor is at the end of the list, this advances it back to the
    /// first element.
    ///
    /// # Returns
    ///   - `Some(&mut N)` if the list is not empty
    ///   - `None` if the list is empty
    pub fn next(&mut self) -> Option<&mut N> {
        unsafe {
            match self.current.take().resolve_mut() {
                // The cursor has no current element, so we are sitting in the
                // cursor's start position. The next element should be the head
                // of the list...
                None => self.list.head.resolve_mut()
                            .and_then(|head| {
                                // if we resolved a head element, make it the
                                // current element and return a reference to it
                                self.current = RawLink::some(head);
                                self.current.resolve_mut()
                            })
                // The cursor did have a current element, so try to advance
                // to that item's next element
              , Some(thing) => {
                    // Set the current element under the cursor to either
                    // the element after the old current, or None if this is
                    // the last element.
                    self.current = match thing.next_mut().resolve_mut() {
                        None => RawLink::none()
                      , Some(other_thing) => RawLink::some(other_thing)
                    };
                    // and return it
                    self.current.resolve_mut()
                }
            }
        }
    }

    /// Steps back the cursor to the previous element and borrows it mutably.
    ///
    /// # Returns
    ///   - `Some(&mut N)` if the list is not empty
    ///   - `None` if the list is empty
    pub fn prev(&mut self) -> Option<&mut N> {
        unimplemented!()
    }

    /// Borrows the next element in the list without advancing the cursor.
    ///
    /// If the cursor is at the end of the list, this returns the first element
    /// instead.
    ///
    /// # Returns
    ///   - `Some(&N)` if the list is not empty
    ///   - `None` if the list is empty
    pub fn peek_next(&self) -> Option<&N> {
        unsafe {
            self.current.resolve()
                .map_or( self.list.front()
                       , |curr| curr.next().resolve())
        }
    }

    /// Borrows the previous element without stepping back the cursor.
    ///
    /// If the cursor is at the head of the list, this returns `None`.
    ///
    /// # Returns
    ///   - `Some(&N)` if the list is not empty
    ///   - `None` if the list is empty or if the cursor is at the head
    ///     of the list
    pub fn peek_prev(&self) -> Option<&N> {
        unsafe {
            self.current.resolve()
                .and_then(|curr| curr.prev().resolve())
        }
    }

    /// Removes the element currently under the cursor and returns it.
    ///
    /// # Returns
    ///   - `Some(T)` if the there is an element currently under the cursor
    ///     (i.e., the list is not empty)
    ///   - `None` if the list is empty.
    pub fn remove(&mut self) -> Option<T> {
        unsafe {
            match self.current.resolve_mut() {
                None    => self.list.pop_front()
              , Some(c) =>
                    c.next_mut().take().resolve_mut()
                     .map(|p| {
                        match p.next_mut().resolve_mut() {
                            None => self.list.tail = RawLink::some(c)
                          , Some(n) => {
                                *n.prev_mut() = RawLink::some(c);
                                *c.next_mut() = RawLink::some(n);
                            }
                        }
                        T::from_raw(p)
                    })
            }
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
        while self.peek_next().is_some() {
            if predicate(self.peek_next().unwrap()) == true {
                return self.remove()
            } else {
                self.next();
            }
        }
        None
    }

    /// Advances the cursor `n` elements and mutably borrows the final element.
    ///
    /// This will wrap the cursor around the list if `n` > the length of
    /// the list.
    pub fn seek_forward(&mut self, n: usize) -> Option<&mut N> {
        for _ in 0 .. (n - 1) { self.next(); }
        self.next()
    }

    /// Moves the cursor back `n` times and mutably borrows the final element.
    ///
    /// This will wrap the cursor around the list if `n` > the length of
    /// the list.
    pub fn seek_backward(&mut self, n: usize) -> Option<&mut N> {
        for _ in 0 .. (n - 1) { self.prev(); }
        self.prev()
    }

}

// impl<'a, T, N> Iterator for ListCursorMut<'a, T, N>
// where T: OwnedRef<N>
//     , T: 'a
//     , N: Node
//     , N: 'a {
//     type Item = &'a mut N;
//
//     fn next<'b: 'a>(&'b mut self) -> Option<&'a mut N> {
//         self.next()
//     }
// }

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
    fn get(&self) -> &T {
        unsafe { self.get() }
    }

    #[inline] fn get_mut(&mut self) -> &mut T {
        unsafe { self.get_mut() }
    }

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
