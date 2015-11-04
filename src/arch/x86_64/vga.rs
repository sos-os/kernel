/// VGA color codes
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Palette(u8);
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Ascii(u8);

impl Palette {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Palette( (bg as u8) << 4 | (fg as u8) )
    }
}

/// A colored VGA character.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Char { pub code: Ascii
                , pub colors: Palette
                }

const COLS: usize = 80;
const ROWS: usize = 25;

type Buffer = [[Char; COLS]; ROWS];
