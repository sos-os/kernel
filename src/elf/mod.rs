//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

use core::mem;
use core::intrinsics;
use core::slice;

pub mod section;
pub mod header;

pub type Section<'a> = section::Header<'a>;
pub type FileHeader = header::Header;

pub struct Binary<'a> {
    pub header: &'a FileHeader
  , binary: &'a [u8]
}

unsafe fn extract_from_slice<'a, T: Sized>( data: &'a [u8]
                                          , offset: usize
                                          , n: usize)
                                          -> &'a [T] {
    assert!( data.len() - offset >= mem::size_of::<T>() * n
           , "Slice too short to contain {} objects of type {}"
           , n
           , intrinsics::type_name::<T>()
           );
    slice::from_raw_parts(data[offset..].as_ptr() as *const T, n)
}
