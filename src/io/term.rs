use ::arch::vga;
use core::ptr::Unique;
use core::mem;
use core::fmt::{Write, Result};

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


    pub fn clear(&mut self) {
        unsafe { *(self.buffer) = mem::zeroed(); }
    }


    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.x = 0;
            self.y += 1;
        } else {
            let ch = vga::Char { ascii: byte
                               , color: self.colors };
            self.buffer()[pos.x][pos.y];
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
            self.y = vga:Y_MAX- 1;
        }

    }

}

impl Write for Terminal {

    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.as_bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }

}