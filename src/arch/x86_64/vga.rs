//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the system's VGA buffer.

use core::mem::transmute;

const FG_MASK: u8 = 0b0000_1111;
const BG_MASK: u8 = 0b1111_0000;

/// VGA color codes
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Palette(u8);

impl Palette {
    /// Returns a `Palette` with the given foreground and background color.
    pub const fn new(fg: Color, bg: Color) -> Self {
        Palette( (bg as u8) << 4 | (fg as u8) )
    }

    /// Returns a new `Palette` with this palette's background color, and
    /// the specified foreground color.
    pub fn set_foreground(&self, fg: Color) -> Self {
        Palette( ( self.0 & BG_MASK) | (fg as u8 & FG_MASK) )
    }

    /// Returns a new `Palette` with this palette's foreground color, and
    /// the specified background color.
    pub fn set_background(&self, bg: Color) -> Self {
        Palette( ( (bg as u8) << 4 & BG_MASK) | (self.0 & FG_MASK) )
    }

    /// Returns this `Palette`'s foreground color.
    pub fn foreground(&self) -> Color {
        unsafe { transmute(self.0 & FG_MASK) }
    }

    /// Returns this `Palette`'s background color.
    pub fn background(&self) -> Color {
        unsafe { transmute((self.0 & BG_MASK) >> 4) }
    }

}

/// A colored VGA character.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Char { pub ascii: u8
                , pub colors: Palette
                }

// TODO: support varying VGA screen sizes?
pub const X_MAX: usize = 80;
pub const Y_MAX: usize = 25;

/// The type signature fot the actual VGA buffer
pub type Buffer = [[Char; X_MAX]; Y_MAX];
