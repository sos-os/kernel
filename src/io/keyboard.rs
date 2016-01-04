//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! PS/2 keyboard driver
use arch::cpu::Port;
use spin::Mutex;

/// PS/2 keyboard scancode
#[derive(Copy,Clone,Debug)]
pub struct Scancode(u8);

impl Scancode {
    fn to_ascii(&self) -> Option<u8> {
        match self.0 {
            0x01 ... 0x1c => Some(TO_ASCII_LOW[self.0 as usize - 0x01])
          , 0x1e ... 0x28 => Some(TO_ASCII_MID[self.0 as usize - 0x1e])
          , 0x2c ... 0x35 => Some(TO_ASCII_HIGH[self.0 as usize - 0x2c])
          , 0x39 => Some(b' ')
          , _ => None
        }
    }
}

/// A PS/2 keyboard state
pub struct Keyboard { /// Port for reading data from the keyboard
                      data_port: Port
                    // , /// Port for sending control signals to the keyboard
                    //   control_port: Port
                    , /// The keyboard's modifier keys
                      pub state: Modifiers
                    }

impl Keyboard {
    #[inline] pub fn read_scancode(&self) -> Scancode {
        unsafe { Scancode(self.data_port.in8()) }
    }
}

/// Scancodes range 0x01 ... 0x1c
const TO_ASCII_LOW: &'static [u8; 31]
    = b"\x1B1234567890-=\0x02\tqwertyuiop[]\r";

/// Scancodes range 0x1E ... 0x28
const TO_ASCII_MID: &'static [u8; 11] = b"asdfghjkl;'";

/// Scancodes range 0x2C ... 0x35
const TO_ASCII_HIGH: &'static [u8; 10] = b"zxcvbnm,./";

impl Keyboard {

}


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
                        , const NUMLOCK  = 0b0000_0001
                        }
}

impl Modifiers {
    const fn new() -> Self {
        Modifiers { bits: 0b0000_0000 }
    }


    /// Returns true if either shift key is pressed.
    #[inline] pub fn is_shifted(&self) -> bool {
        self.contains(SHIFT)
    }

    /// Returns true if the keyboard's state is currently uppercase.
    #[inline] pub fn is_uppercase(&self) -> bool {
        self.is_shifted() ^ self.contains(CAPSLOCK)
    }

    /// Updates the modifiers state from a given scancode.
    fn update(&mut self, scancode: Scancode) {
        match scancode { Scancode(0x1D) => self.insert(L_CTRL)
                       , Scancode(0x2A) => self.insert(L_SHIFT)
                       , Scancode(0x36) => self.insert(R_SHIFT)
                       , Scancode(0x38) => self.insert(L_ALT)
                         // Caps lock toggles on leading edge
                       , Scancode(0x3A) => self.toggle(CAPSLOCK)
                       , Scancode(0x9D) => self.remove(L_CTRL)
                       , Scancode(0xAA) => self.remove(L_SHIFT)
                       , Scancode(0xB6) => self.remove(R_SHIFT)
                       , Scancode(0xB8) => self.remove(L_ALT)
                       , _    => {}
        }
    }

    /// Apply the keyboard's modifiers to an ASCII scancode.
    fn modify(&self, ascii: u8) -> u8 {
        match ascii {
            b'a' ... b'z' if self.is_uppercase() => ascii - b'a' + b'A'
          , b'1' ... b'9' if self.is_shifted()   => ascii - b'1' + b'!'
          , b'0' if self.is_shifted()            => b')'
          , _ => ascii
        }
    }
}

/// Our global keyboard state, protected by a mutex.
static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    data_port: unsafe { Port::new(0x60) }
  , state: Modifiers::new()
});

pub fn read_char() -> Option<char> {
    let mut lock = KEYBOARD.lock();

    let code = lock.read_scancode();
    lock.state.update(code);

    code.to_ascii()
        .map(|ascii| lock.state.modify(ascii) as char)
}
