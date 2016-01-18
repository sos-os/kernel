//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//

extern {
    pub static __vga_buffer: u8;
}
/// The system's global VGA terminal
pub use vga::CONSOLE;


// pub static CONSOLE: Mutex<Terminal>
//     = Mutex::new(unsafe { Terminal::new(
//          Palette::new(Color::LightGrey, Color::Black )
//        , 0xB8000
//     )});
