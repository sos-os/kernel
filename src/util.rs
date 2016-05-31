//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! General purpose stuff I couldn't find a better home for.
use core::fmt;

/// The unreachable Void type.
pub enum Void {}
impl fmt::Debug for Void {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}
