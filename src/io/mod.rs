//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
pub mod term;
use core::ops;

/// This is basically a braindead reimplementation of the standard
/// library's `Read` trait. Most of the methods available on the
/// standard lib's `Read` are not yet implemented.
pub trait Read {
    type Error;
    fn read(&mut self, buf: &mut [u8])     -> Result<usize, Self::Error>;
    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait Write {
    type Error;
    fn write(&mut self, buf: &[u8])       -> Result<usize, Self::Error>;
}

impl<'a, 'b, E> ops::Shl<&'a [u8]> for &'b mut Write<Error=E> {
    type Output = Self;

    /// Fakes the C++ `<<` operator for IOStreams on Write.
    fn shl(self, _rhs: &'a [u8]) -> Self::Output {
        try!(self.write(_rhs));
        self
    }
}
