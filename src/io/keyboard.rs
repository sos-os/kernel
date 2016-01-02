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
    /// Returns true if either shift key is pressed.
    #[inline] fn is_shifted(&self) -> bool {
        self.intersects(SHIFT)
    }

    /// Returns true if the keyboard's state is currently uppercase.
    #[inline] fn is_uppercase(&self) -> bool {
        self.is_shifted() ^ self.contains(CAPSLOCK)
    }

    /// Updates the modifiers state from a given scancode.
    fn update(&mut self, scancode: u8) {
        match scancode { 0x1D => self.insert(L_CTRL)
                       , 0x2A => self.insert(L_SHIFT)
                       , 0x36 => self.insert(R_SHIFT)
                       , 0x38 => self.insert(L_ALT)
                         // Caps lock toggles on leading edge
                       , 0x3A => self.toggle(CAPSLOCK)
                       , 0x9D => self.remove(L_CTRL)
                       , 0xAA => self.remove(L_SHIFT)
                       , 0xB6 => self.remove(R_SHIFT)
                       , 0xB8 => self.remove(L_ALT)
                       , _    => {}
        }
    }

    /// Apply the keyboard's modifiers to an ASCII scancode.
    fn apply_to_ascii(&self, ascii: u8) -> u8 {
        match ascii {
            b'a' ... b'z' if self.is_uppercase() => ascii - b'a' + b'A'
          , b'1' ... b'9' if self.is_shifted()   => ascii - b'1' + b'!'
          , b'0' if self.is_shifted()            => b')'
          , _ => ascii
        }
    }
}
