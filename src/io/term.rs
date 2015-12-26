//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use vga::{Terminal, Palette, Color};
use spin::Mutex;

/// The system's global VGA terminal
pub static CONSOLE: Mutex<Terminal>
    = Mutex::new(unsafe { Terminal::new(
         Palette::new(Color::LightGreen, Color::Black )
       , 0xB8000
    )});
