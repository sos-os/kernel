//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Arch-specific VGA port port driver

use vga::{Palette, Color, Terminal};
use spin::Mutex;

// extern {
//     #[link_section = ".__vga_buffer"]
//     static mut BUFFER: vga::Buffer;
// }

/// The system's global VGA terminal
pub static CONSOLE: Mutex<Terminal>
    = Mutex::new(unsafe { Terminal::new(
         Palette::new(Color::LightGrey, Color::Black )
       , 0x8000
    )});

pub fn clear() {
    CONSOLE.lock().clear();
}
