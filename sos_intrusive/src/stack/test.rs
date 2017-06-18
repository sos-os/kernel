//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use stack::Node;
use rawlink::RawLink;

#[derive(Debug)]
pub struct NumberedNode {
    pub number: usize,
    next: RawLink<NumberedNode>,
}

impl NumberedNode {
    pub fn new(number: usize) -> Self {
        NumberedNode {
            number: number,
            next: RawLink::none(),
        }
    }
}

impl Node for NumberedNode {

    fn next(&self) -> &RawLink<Self> {
        &self.next
    }

    fn next_mut(&mut self) -> &mut RawLink<Self> {
        &mut self.next
    }
}

impl PartialEq for NumberedNode {
    fn eq(&self, rhs: &Self) -> bool { self.number == rhs.number }
}

mod boxed {
    use std::boxed::Box;

    use super::super::Stack;
    use super::*;

    type TestStack = Stack<Box<NumberedNode>, NumberedNode>;

    #[test]
    fn not_empty_after_push() {
        let mut list = TestStack::new();

        assert_eq!(list.peek(), None);

        assert!(list.is_empty());

        list.push(box NumberedNode::new(1));

        assert!(!list.is_empty());
    }

    #[test]
    fn contents_after_first_push() {
        let mut list = TestStack::new();

        list.push(box NumberedNode::new(1));

        assert_eq!(list.peek().unwrap().number, 1);
    }


    #[test]
    fn contents_after_pushes() {
        let mut list = TestStack::new();

        list.push(box NumberedNode::new(0));
        assert_eq!(list.peek().unwrap().number, 0);
        list.push(box NumberedNode::new(1));

        assert_eq!(list.peek().unwrap().number, 1);

        assert!(!list.is_empty());
    }

    #[test]
    fn test_pop_front() {
        let mut list = TestStack::new();

        assert_eq!(list.peek(), None);
        assert!(list.is_empty());

        list.push(Box::new(NumberedNode::new(4)));
        assert!(!list.is_empty());
        assert_eq!(list.peek().unwrap().number, 4);

        list.push(Box::new(NumberedNode::new(3)));
        assert!(!list.is_empty());
        assert_eq!(list.peek().unwrap().number, 3);

        list.push(Box::new(NumberedNode::new(2)));
        assert!(!list.is_empty());
        assert_eq!(list.peek().unwrap().number, 2);

        list.push(Box::new(NumberedNode::new(1)));
        assert!(!list.is_empty());
        assert_eq!(list.peek().unwrap().number, 1);

        list.push(Box::new(NumberedNode::new(0)));
        assert!(!list.is_empty());
        assert_eq!(list.peek().unwrap().number, 0);

        assert_eq!(list.pop().unwrap().number, 0);
        assert_eq!(list.pop().unwrap().number, 1);
        assert_eq!(list.pop().unwrap().number, 2);
        assert_eq!(list.pop().unwrap().number, 3);
        assert_eq!(list.pop().unwrap().number, 4);

        assert!(list.is_empty());
        assert_eq!(list.pop(), None);
    }


}
