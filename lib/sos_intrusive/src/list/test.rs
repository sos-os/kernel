use core::ptr::Unique;

use std::boxed::Box;

use list::{ListNode, Node};
use rawlink::RawLink;

#[derive(Debug)]
struct NumberedNode {
    pub number: usize,
    prev: RawLink<NumberedNode>,
    next: RawLink<NumberedNode>,
}

impl NumberedNode {
    pub fn new(number: usize) -> Self {
        NumberedNode {
            number: number,
            prev: RawLink::none(),
            next: RawLink::none(),
        }
    }
}

impl Node for NumberedNode {
    fn prev(&self) -> &RawLink<Self> {
        &self.prev
    }

    fn next(&self) -> &RawLink<Self> {
        &self.next
    }

    fn prev_mut(&mut self) -> &mut RawLink<Self> {
        &mut self.prev
    }

    fn next_mut(&mut self) -> &mut RawLink<Self> {
        &mut self.next
    }
}

impl PartialEq for NumberedNode {
    fn eq(&self, rhs: &Self) -> bool { self.number == rhs.number }
}

//
// #[test]
//   fn test_unique() {
//       let one = Box::new(NumberedNode::new(1));
//       let two = Box::new(NumberedNode::new(2));
//       let three = Box::new(NumberedNode::new(3));
//
//       let one_ptr = Box::into_raw(one);
//       let two_ptr = Box::into_raw(two);
//       let three_ptr = Box::into_raw(three);
//
//       let one_unique = unsafe { Unique::new(one_ptr) };
//       let two_unique = unsafe { Unique::new(two_ptr) };
//       let three_unique = unsafe { Unique::new(three_ptr) };
//
//       let mut list = ListNode::<Unique<NumberedNode>, NumberedNode>::new();
//
//       list.push_front(three_unique);
//       list.push_front(two_unique);
//       list.push_front(one_unique);
//
//       unsafe {
//           assert_eq!(list.pop_back().unwrap().get_mut().number, 3);
//           assert_eq!(list.pop_back().unwrap().get_mut().number, 2);
//           assert_eq!(list.pop_back().unwrap().get_mut().number, 1);
//           assert!(list.pop_back().is_none());
//       }
//
//       // Cleanup
//       unsafe {
//           Box::from_raw(one_ptr);
//           Box::from_raw(two_ptr);
//           Box::from_raw(three_ptr);
//       }
//   }

#[test]
fn test_not_empty_after_push() {
    let mut list = ListNode::<Box<NumberedNode>, NumberedNode>::new();

    assert_eq!(list.front(), None);
    assert_eq!(list.back(), None);

    assert!(list.is_empty());

    list.push_front(Box::new(NumberedNode::new(1)));

    assert!(!list.is_empty());
}

#[test]
fn test_has_contents_after_push() {
    let mut list = ListNode::<Box<NumberedNode>, NumberedNode>::new();

    list.push_front(Box::new(NumberedNode::new(1)));

    assert_eq!(list.front().unwrap().number, 1);
}


#[test]
fn test_head_tail_same_first_push() {
    let mut list = ListNode::<Box<NumberedNode>, NumberedNode>::new();

    list.push_front(Box::new(NumberedNode::new(1)));

    assert_eq!(list.front().unwrap().number, 1);
    assert_eq!(list.back().unwrap().number, 1);
    assert_eq!(list.front().unwrap(), list.back().unwrap());
}



#[test]
fn test_contents_after_pushes() {
    let mut list = ListNode::<Box<NumberedNode>, NumberedNode>::new();

    list.push_front(Box::new(NumberedNode::new(0)));
    list.push_front(Box::new(NumberedNode::new(1)));

    assert_eq!(list.back().unwrap().number, 0);
    assert_eq!(list.front().unwrap().number, 1);

    list.push_back(Box::new(NumberedNode::new(2)));
    assert_eq!(list.back().unwrap().number, 2);
    assert_eq!(list.front().unwrap().number, 1);

    list.push_back(Box::new(NumberedNode::new(3)));
    assert_eq!(list.back().unwrap().number, 3);
    assert_eq!(list.front().unwrap().number, 1);

    assert!(!list.is_empty());
}
