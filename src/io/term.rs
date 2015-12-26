//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use vga;
use core::ptr::Unique;
use spin::Mutex;

const ANSI_ESCAPE: &'static str = "\x1b";

/// The system's global VGA terminal
pub static CONSOLE: Mutex<Terminal>
    = Mutex::new(Terminal {
        colors: vga::Palette::new( vga::Color::LightGreen
                                 , vga::Color::Black
                                 )
      , x: 0
      , y: 0
      , buffer: unsafe { Unique::new(0xB8000 as *mut _) },
    });
