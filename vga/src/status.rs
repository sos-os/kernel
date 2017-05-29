use super::{Terminal, Color};
use core::fmt::Write;

pub trait Status {
    fn okay(&mut self);
    fn fail(&mut self);
}

impl Status for Terminal {
    fn okay(&mut self) {
        while self.x_position() < 71 { self.write_byte(b' '); }
        self.write_str("[ ");
        self.set_colors(Color::Green, Color::Black, );
        self.write_str("OKAY");
        self.set_colors(Color::LightGrey, Color::Black);
        self.write_str(" ]\n");
    }
    fn fail(&mut self) {
        while self.x_position() < 71 { self.write_byte(b' '); }
        self.write_str("[ ");
        self.set_colors(Color::Red, Color::Black);
        self.write_str("FAIL");
        self.set_colors(Color::LightGrey, Color::Black);
        self.write_str(" ]\n");
    }
}
