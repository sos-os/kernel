//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use ::arch::vga;
use core::ptr::Unique;
use core::mem;
use core::fmt::{Write, Result};
use spin::Mutex;

const ANSI_ESCAPE: &'static str = r"\x1b";

pub struct Terminal { buffer: Unique<vga::Buffer>
                    , x: usize
                    , y: usize
                    , colors: vga::Palette
                    }


macro_rules! next_ansi_byte {
    ($b:expr) => { $b.next().expect("Unterminated ANSI escape sequence!") }
}

impl Terminal {

    #[inline]
    fn buffer(&mut self) -> &mut vga::Buffer {
        unsafe { self.buffer.get_mut() }
    }

    pub fn set_colors(&mut self, bg: vga::Color, fg: vga::Color)
                     -> &mut Self
    {
        self.colors = vga::Palette::new(bg,fg);
        self
    }
    fn scroll(&mut self) {
        let mut rows = self.buffer()
                           .iter_mut();

        let mut next = rows.next()
                           .unwrap();

        while let Some(thing) = rows.next() {
            mem::swap(next, thing);
            next = thing;
        }

        // empty last line
        unsafe { *next = mem::zeroed() }
    }


    pub fn clear(&mut self) -> &mut Self {
        unsafe { *(self.buffer()) = mem::zeroed(); }
        self
    }

    pub fn write_byte(&mut self, byte: u8) -> &mut Self {
        if byte == b'\n' {
            self.x = 0;
            self.y += 1;
        } else {
            // set character at position
            self.buffer()[self.y][self.x]
                = vga::Char { ascii: byte
                            , colors: self.colors };
            self.x += 1;

            // check for line wrapping
            if self.x >= vga::X_MAX {
                self.x = 0;
                self.y += 1;
            }
        }
        // check for scrolling
        if self.y >= vga::Y_MAX {
            self.scroll();
            self.y = vga::Y_MAX- 1;
        }
        self
    }

    fn handle_ansi_escape(&self, escape_code: &str) -> Result {
        match escape_code[4..].as_bytes() {
            [b'3', n @ u8, b'm'] => {
                unsafe { self.colors
                             .set_foreground(mem::transmute(n - 48)); }
                Ok(())
            }
          , [b'4', n @ u8, b'm'] => {
                unsafe { self.colors
                             .set_background(mem::transmute(n - 48)); }
                Ok(())
            }
          , _ => unimplemented!()
        }
        // let escape_seq: &str = bytes.take_while(|b| b != b'm')
        //                       .collect::<&str>();
        // match escape_seq {
        //     [b'3', n] => unsafe {
        //         self.colors.set_foreground(mem::transmute(n - 48))
        //     }
        // }
        // while let Some(byte) = bytes.next() {
        //     match *byte {
        //         // we've recieved an ANSI escape sequence.
        //         // this basically enters a mediocre FSM for matching ANSI
        //         // control codes.
        //         0x1b => match *next_ansi_byte!(bytes) {
        //             // handle multi-char ANSI escapes
        //             b'[' => match *next_ansi_byte!(bytes) {
        //                 // foreground color code
        //                 fg @ 30 ... 37 => {
        //                     if !(*next_ansi_byte!(bytes) == b'm') {
        //                         unsafe {
        //                             let color: vga::Color
        //                                 = mem::transmute(fg - 30);
        //                             self.colors
        //                                 .set_foreground(color);
        //                         }
        //
        //                     }
        //                 }
        //                 // background color code
        //               , 40 ... 47 => {
        //
        //                 }
        //               , _ => unimplemented!()
        //             }
        //           , _    => unimplemented!()
        //         }
        //         // otherwise, treat the byte as a normal ASCII char
        //       , b => { self.write_byte(b); }
        //     }
        // }

    }

}


impl Write for Terminal {

    fn write_str(&mut self, s: &str) -> Result {
        if s.contains(ANSI_ESCAPE) {
            let escape_idxes = s.match_indices(ANSI_ESCAPE);
            for (idx, _) in escape_idxes {
                let (segment, _) = s.split_at(idx);
                if segment.starts_with(ANSI_ESCAPE) {
                    try!(self.handle_ansi_escape(segment))
                } else {
                    for byte in segment.as_bytes() {
                        self.write_byte(*byte);
                    }
                }
            }
        } else {
            for byte in s.as_bytes() {
                self.write_byte(*byte);
            }
        }
        Ok(())
    }

}

/// The system's VGA terminal
pub static CONSOLE: Mutex<Terminal>
    = Mutex::new(Terminal {
        colors: vga::Palette::new( vga::Color::LightGreen
                                 , vga::Color::Black
                                 )
      , x: 0
      , y: 0
      , buffer: unsafe { Unique::new(0xB8000 as *mut _) },
    });
