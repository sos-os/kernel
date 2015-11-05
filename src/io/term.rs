use ::arch::vga;
use core::ptr::Unique;
use core::mem;
use core::fmt::{Write, Result};
use spin::Mutex;

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

    pub fn set_colors(&mut self, bg: vga::Color, fg :vga::Color)
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

}

impl Write for Terminal {

    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.as_bytes() {
            self.write_byte(*byte);
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
