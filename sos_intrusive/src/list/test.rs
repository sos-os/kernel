//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use list::Node;
use rawlink::RawLink;

#[derive(Debug)]
pub struct NumberedNode {
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

mod boxed {
    use std::boxed::Box;

    use list::List;
    use super::*;

    type TestList = List<Box<NumberedNode>, NumberedNode>;

    #[test]
    fn not_empty_after_push() {
        let mut list = TestList::new();

        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);

        assert!(list.is_empty());

        list.push_front(box NumberedNode::new(1));

        assert!(!list.is_empty());
    }

    #[test]
    fn contents_after_first_push() {
        let mut list = TestList::new();

        list.push_front(box NumberedNode::new(1));

        assert_eq!(list.front().unwrap().number, 1);
    }


    #[test]
    fn head_tail_same_first_push() {
        let mut list = TestList::new();

        list.push_front(box NumberedNode::new(1));

        assert_eq!(list.front().unwrap().number, 1);
        assert_eq!(list.back().unwrap().number, 1);
        assert_eq!(list.front().unwrap(), list.back().unwrap());
    }

    #[test]
    fn head_tail_not_same_second_push() {
        let mut list = TestList::new();

        list.push_front(box NumberedNode::new(0));
        list.push_front(box NumberedNode::new(1));

        assert!(list.front().unwrap() != list.back().unwrap());
    }


    #[test]
    fn contents_after_pushes() {
        let mut list = TestList::new();

        list.push_front(box NumberedNode::new(0));
        list.push_front(box NumberedNode::new(1));

        assert_eq!(list.back().unwrap().number, 0);
        assert_eq!(list.front().unwrap().number, 1);

        list.push_back(box NumberedNode::new(2));
        assert_eq!(list.back().unwrap().number, 2);
        assert_eq!(list.front().unwrap().number, 1);

        list.push_back(box NumberedNode::new(3));
        assert_eq!(list.back().unwrap().number, 3);
        assert_eq!(list.front().unwrap().number, 1);

        assert!(!list.is_empty());
    }

    #[test]
    fn test_pop_front() {
        let mut list = TestList::new();

        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
        assert!(list.is_empty());

        list.push_front(Box::new(NumberedNode::new(2)));

        assert!(!list.is_empty());
        assert_eq!(list.front(), list.back());

        list.push_front(Box::new(NumberedNode::new(1)));
        list.push_front(Box::new(NumberedNode::new(0)));

        assert_eq!(list.front().unwrap().number, 0);
        assert_eq!(list.back().unwrap().number, 2);

        list.push_back(Box::new(NumberedNode::new(3)));
        assert_eq!(list.back().unwrap().number, 3);

        list.push_back(Box::new(NumberedNode::new(4)));
        assert_eq!(list.back().unwrap().number, 4);

        assert!(!list.is_empty());

        assert_eq!(list.pop_front().unwrap().number, 0);
        assert_eq!(list.pop_front().unwrap().number, 1);
        assert_eq!(list.pop_front().unwrap().number, 2);
        assert_eq!(list.pop_front().unwrap().number, 3);
        assert_eq!(list.pop_front().unwrap().number, 4);

        assert!(list.is_empty());
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_pop_back() {
        let mut list = TestList::new();

        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
        assert!(list.is_empty());

        list.push_front(Box::new(NumberedNode::new(2)));

        assert!(!list.is_empty());
        assert_eq!(list.front(), list.back());

        list.push_front(Box::new(NumberedNode::new(1)));
        list.push_front(Box::new(NumberedNode::new(0)));

        assert_eq!(list.front().unwrap().number, 0);
        assert_eq!(list.back().unwrap().number, 2);

        list.push_back(Box::new(NumberedNode::new(3)));
        assert_eq!(list.back().unwrap().number, 3);

        list.push_back(Box::new(NumberedNode::new(4)));
        assert_eq!(list.back().unwrap().number, 4);

        assert!(!list.is_empty());

        assert_eq!(list.pop_back().unwrap().number, 4);
        assert_eq!(list.pop_back().unwrap().number, 3);
        assert_eq!(list.pop_back().unwrap().number, 2);
        assert_eq!(list.pop_back().unwrap().number, 1);
        assert_eq!(list.pop_back().unwrap().number, 0);

        assert!(list.is_empty());
        assert_eq!(list.pop_back(), None);
    }


}

// mod mut_ptr {
//     use list::List;
//     use super::*;
//
//     type TestList<'a> = List<&'a mut NumberedNode, NumberedNode>;
//
//     #[test]
//     fn not_empty_after_push() {
//         let mut list = TestList::new();
//
//         assert_eq!(list.front(), None);
//         assert_eq!(list.back(), None);
//
//         assert!(list.is_empty());
//
//         list.push_front(&mut NumberedNode::new(1));
//
//         assert!(!list.is_empty());
//     }
//
//     #[test]
//     fn contents_after_first_push() {
//         let mut list = TestList::new();
//
//         list.push_front(&mut NumberedNode::new(1));
//
//         assert_eq!(list.front().unwrap().number, 1);
//     }
//
//
//     #[test]
//     fn head_tail_same_first_push() {
//         let mut list = TestList::new();
//
//         list.push_front(&mut NumberedNode::new(1));
//
//         assert_eq!(list.front().unwrap().number, 1);
//         assert_eq!(list.back().unwrap().number, 1);
//         assert_eq!(list.front().unwrap(), list.back().unwrap());
//     }
//
//     #[test]
//     fn head_tail_not_same_second_push() {
//         let mut list = TestList::new();
//
//         list.push_front(&mut NumberedNode::new(0));
//         list.push_front(&mut NumberedNode::new(1));
//
//         assert!(list.front().unwrap() != list.back().unwrap());
//     }
//
//
//     #[test]
//     fn contents_after_pushes() {
//         let mut list = TestList::new();
//
//         list.push_front(&mut NumberedNode::new(0));
//         list.push_front(&mut NumberedNode::new(1));
//
//         assert_eq!(list.back().unwrap().number, 0);
//         assert_eq!(list.front().unwrap().number, 1);
//
//         list.push_back(&mut NumberedNode::new(2));
//         assert_eq!(list.back().unwrap().number, 2);
//         assert_eq!(list.front().unwrap().number, 1);
//
//         list.push_back(&mut NumberedNode::new(3));
//         assert_eq!(list.back().unwrap().number, 3);
//         assert_eq!(list.front().unwrap().number, 1);
//
//         assert!(!list.is_empty());
//     }
//
// }
