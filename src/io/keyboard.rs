//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! PS/2 keyboard driver

bitflags! {
    flags Modifiers: u8 { const L_SHIFT  = 0b1000_0000
                        , const R_SHIFT  = 0b0100_0000
                        , const SHIFT    = L_SHIFT.bits | R_SHIFT.bits
                        , const R_CTRL   = 0b0010_0000
                        , const L_CTRL   = 0b0001_0000
                        , const CTRL     = L_CTRL.bits | R_CTRL.bits
                        , const R_ALT    = 0b0000_1000
                        , const L_ALT    = 0b0000_0100
                        , const ALT      = L_ALT.bits | R_ALT.bits
                        , const CAPSLOCK = 0b0000_0010
                        }
}

impl Modifiers {
    #[inline] fn is_shifted(&self) -> bool {
        self.intersects(SHIFT)
    }

    #[inline] fn is_uppercase(&self) -> bool {
        self.is_shifted() ^ self.contains(CAPSLOCK)
    }
}
