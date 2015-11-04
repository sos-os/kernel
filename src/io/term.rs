use ::arch::vga;
use core::ptr::Unique;
use core::mem;

struct Point { x: usize, y: usize };

impl Point {
    fn next(&self) -> Self {
        unimplemented!()
    }
}

pub struct Terminal { buffer: Unique<vga::Buffer>
                    , pos: Point
                    , colors: vga::Palette
                    }

impl Terminal {

    pub fn clear(&mut self) {
        unsafe { *(self.buffer.get_mut()) = mem::zeroed(); }
    }

}
