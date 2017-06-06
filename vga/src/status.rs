use super::{Terminal, Color};
use core::fmt::{Result, Write};

pub trait Status {
    fn okay(&mut self) -> Result ;
    fn fail(&mut self) -> Result ;
}

impl Status for Terminal {
    fn okay(&mut self) -> Result {
        while self.x_position() < 71 { self.write_byte(b' '); }
        self.write_str("[ ")?;
        self.set_colors(Color::Green, Color::Black, );
        self.write_str("OKAY")?;
        self.set_colors(Color::LightGrey, Color::Black);
        self.write_str(" ]\n")?;
        Ok(())
    }
    fn fail(&mut self) -> Result {
        while self.x_position() < 71 { self.write_byte(b' '); }
        self.write_str("[ ")?;
        self.set_colors(Color::Red, Color::Black);
        self.write_str("FAIL")?;
        self.set_colors(Color::LightGrey, Color::Black);
        self.write_str(" ]\n")?;
        Ok(())
    }
}
