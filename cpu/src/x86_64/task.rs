//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Tasking
use ::segment;

/// A 64-bit Task State Descriptor
#[repr(packed)]
pub struct StateDescriptor { pub upper: segment::Descriptor
                           , pub lower: u64
                           }


/// A 64-bit Task State Segment
#[repr(packed)]
pub struct StateSegment {
    _reserved_1: u32
  , /// 64-bit values of the stack pointers (`%rsp`) for privilege rings 0-2
    //  TODO: should this be an array or just three u64s?
    pub rsp: [u64; 3]
  , _reserved_2: u32
  , /// 64-bit values of the interrupt stack table registers
    pub ist: [u64; 7]
  , _reserved_3: u64
  , _reserved_4: u16
  , /// the base offset of the IO map
    pub iomap_base_offset: u16
}

impl StateSegment {

    /// Returns a new, empty TSS
    pub const fn new() -> Self {
        StateSegment { _reserved_1: 0
                     , rsp: [ 0, 0, 0 ]
                     , _reserved_2: 0
                     , ist: [ 0, 0, 0, 0, 0, 0, 0, ]
                     , _reserved_3: 0
                     , _reserved_4: 0
                     , iomap_base_offset: 0
                     }
    }
}
