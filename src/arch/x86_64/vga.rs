//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the system's VGA buffer.

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
    pub const fn new(fg: Color, bg: Color) -> Self {
        Palette( (bg as u8) << 4 | (fg as u8) )
    }
}

/// A colored VGA character.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Char { pub ascii: u8
                , pub colors: Palette
                }

pub const X_MAX: usize = 80;
pub const Y_MAX: usize = 25;

pub type Buffer = [[Char; X_MAX]; Y_MAX];
