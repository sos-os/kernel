//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Tasking
use ::segment;
use memory::VAddr;

/// A 64-bit Task State Descriptor
#[repr(packed)]
pub struct StateDescriptor { pub upper: segment::Descriptor
                           , pub lower: u64
                           }


/// A 64-bit Task State Segment
#[repr(C, packed)]
#[derive(Debug)]
pub struct StateSegment {
    _reserved_1: u32
  , /// 64-bit values of the stack pointers (`%rsp`) for privilege rings 0-2
    //  TODO: should this be an array or just three u64s?
    pub rsp: [VAddr; 3]
  , _reserved_2: u32
  , /// 64-bit values of the interrupt stack table registers
    pub ist: [VAddr; 7]
  , _reserved_3: u64
  , _reserved_4: u16
  , /// the base offset of the IO map
    pub iomap_base_offset: u16
}

impl StateSegment {

    /// Returns a new, empty TSS
    pub const fn new() -> Self {
        StateSegment { _reserved_1: 0
                     , rsp: [ VAddr::new(0); 3]
                     , _reserved_2: 0
                     , ist: [ VAddr::new(0); 7 ]
                     , _reserved_3: 0
                     , _reserved_4: 0
                     , iomap_base_offset: 0
                     }
    }
}
