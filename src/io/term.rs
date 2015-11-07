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

pub struct Terminal { buffer: Unique<vga::Buffer>
                    , x: usize
                    , y: usize
                    , colors: vga::Palette
                    }

impl Terminal {

    #[inline]
    fn buffer(&mut self) -> &mut vga::Buffer {
        unsafe { self.buffer.get_mut() }
    }

    /// Set the color palette used for writing subsequent characters.
    pub fn set_colors(&mut self, bg: vga::Color, fg: vga::Color)
                     -> &mut Self
    {
        self.colors = vga::Palette::new(bg,fg);
        self
    }

    /// Scrolls the terminal one row.
    fn scroll(&mut self) {
        // construct an iterator over the whole buffer
        let mut rows = self.buffer()
                           .iter_mut();

        // the current row in the buffer
        let mut current = rows.next()
                           .unwrap();

        while let Some(next) = rows.next() {
            // while there are rows remaining in the iterator, swap the
            // next row with the current one (moving it back by one)
            mem::swap(current, next);
            // and advance our pointer to the current row.
            current = next;
        }

        // empty the last line in the buffer
        unsafe { *current = mem::zeroed() }
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> &mut Self {
        // to clear the terminal, we just zero out the whole buffer.
        unsafe { *(self.buffer()) = mem::zeroed(); }
        self
    }

    /// Write the given byte to the terminal, and advance the cursor position.
    pub fn write_byte(&mut self, byte: u8) -> &mut Self {
        if byte == b'\n' {
            // if the byte is a newline, we just advance to the next line
            // and reset the column position.
            self.x = 0;
            self.y += 1;
        } else {
            // otherwise, it's a regular character, so we just set the
            // byte at the current position in the buffer to that
            // character (with the current color palette)
            self.buffer()[self.y][self.x]
                = vga::Char { ascii: byte
                            , colors: self.colors };
            // and advance our column position by one
            self.x += 1;

            if self.x >= vga::X_MAX {
                // if we've reached the end of the line, advance to the next
                self.x = 0;
                self.y += 1;
            }
        }
        if self.y >= vga::Y_MAX {
            // if we've reached the bottom of the terminal, scroll.
            self.scroll();
            self.y = vga::Y_MAX- 1;
        }
        self
    }

    fn handle_ansi_escape(&self, escape_code: &str) -> Result {
        match escape_code.as_bytes() {
            // `\x1b[3Nm` sets the foreground color to N.
            [0x1b, b'[', b'3', n, b'm'] => {
                unsafe { self.colors
                             .set_foreground(mem::transmute(n - 48)); }
                Ok(())
            }
            // `\x1b[4Nm` sets the background color to N
          , [0x1b, b'[', b'4', n, b'm'] => {
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

struct AnsiEscapeIter<'a> { curr_slice: &'a str
                          , in_escape: bool
                          }

impl<'a> AnsiEscapeIter<'a> {

    pub fn new(s: &'a str) -> Self {
        AnsiEscapeIter { curr_slice: s
                       , in_escape: false
                       }
    }
}

impl<'a> Iterator for AnsiEscapeIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_slice.len() == 0 {
            // if the remaining string is empty, we just return `None`
            None
        } else {
            // otherwise, find the next index to chunk on.
            let maybe_idx
                = if self.in_escape {
                     // if we're in an escape code, we split the chunk at the
                     // index of the next 'm' character, adding 1 so that the
                     // 'm' is in the escape code chunk.
                    self.curr_slice.find('m')
                        .map(|idx| idx + 1)
                } else {
                    // otherwise, split at the next ANSI escape sequence
                    self.curr_slice.find(ANSI_ESCAPE)
                };

            // if we found another index to chunk on, map over that index;
            // otherwise, we just yield the rest of the string
            maybe_idx.map_or(
                Some(self.curr_slice) // remainder (if no index to chunk on)
              , |idx| { // otherwise, chunk along that index...
                    let (chunk, next_slice) = self.curr_slice
                                                  .split_at(idx);
                    self.curr_slice = next_slice; // update current chunk
                    Some(chunk)                   // return the chunk
                })
        }


    }
}

impl Write for Terminal {

    fn write_str(&mut self, s: &str) -> Result {

        if s.contains(ANSI_ESCAPE) {
            // if the segment contains an ANSI escape, construct an iterator
            // over each chunk containing either an escape sequence or text
            for segment in AnsiEscapeIter::new(s) {
                if segment.starts_with(ANSI_ESCAPE) {
                    // if the current segment is an ANSI escape code,
                    // try to handle the escape and fail if it is malformed
                    try!(self.handle_ansi_escape(segment))
                } else {
                    // otherwise, just write each chunk in the string.
                    for byte in segment.as_bytes() {
                        self.write_byte(*byte);
                    }
                }
            }
        } else {
            // otherwise, if there are no ANSI escape codes,
            // we can just write each byte in the string.
            for byte in s.as_bytes() {
                self.write_byte(*byte);
            }
        }
        Ok(())
    }

}
