use super::{Read, Write, Port};
use ::util;

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Serial { Com1 = 0
                , Com2 = 1
                , Com3 = 2
                , Com4 = 3
                }

impl Serial {
    #[inline]
    fn get_port_addr(&self) -> Option<u16> {
        match BDA_SERIAL_INFO[*self as usize] {
            n if n > 0 => Some(n)
          , _ => None
        }
    }

    /// Returns `Some(SerialPort)` if this port exists, `None` if it does not
    pub fn new(&self) -> Option<SerialPort> {
        self.get_port_addr().map(|port| {
            Port::<u8>::new(port + 1).write(0x00);
            Port::<u8>::new(port + 3).write(0x80);
            Port::<u8>::new(port + 0).write(0x03);
            Port::<u8>::new(port + 1).write(0x00);
            Port::<u8>::new(port + 3).write(0x03);
            Port::<u8>::new(port + 2).write(0xC7);
            Port::<u8>::new(port + 4).write(0x0B);
            Port::<u8>::new(port + 1).write(0x01);

            SerialPort { data_port: Port::<u8>::new(port)
                       , status_port: Port::<u8>::new(port + 5)
                       }
        })
    }
}

/// Address of the BIOS Data Area (BDA)
/// where the serial port addresses are stored.
const BDA_ADDR: usize = 0x400;

lazy_static! {
    static ref BDA_SERIAL_INFO: [u16; 4]
        = unsafe { *(BDA_ADDR as *const [u16; 4]) };

}

pub struct SerialPort { data_port: Port<u8>
                      , status_port: Port<u8>
                      }

impl SerialPort {
    /// Returns true if the serial port has recieved data
    #[inline]
    pub fn has_byte(&self) -> bool {
        self.status_port.read() & 1 != 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.status_port.read() & 0x20 == 0x20
    }

    #[inline]
    pub fn read_byte(&self) -> u8 {
        while !self.has_byte() {};
        self.data_port.read()
    }

    #[inline]
    pub fn write_byte(&self, byte: u8) {
        while !self.is_empty() {};
        self.data_port.write(byte)
    }
}

impl Read for SerialPort {
    type Error = util::Void;

    /// Reads a single byte into the given buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(match &mut *buf {
            // if the length of the buffer is 0, then obviously
            // no bytes were read
            &mut []                  => 0
            // otherwise, read one byte into the head of the buffer
          , &mut [ref mut head, _..] => { *head = self.read_byte(); 1 }
        })
    }

    /// Reads a new byte into each position in the buffer.
    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_bytes = 0;
        for idx in buf.iter_mut() {
            // for each index in the buffer, read another byte from the port
            *idx = self.read_byte();
            // and increment the number of bytes read (this should be faster
            // than calling `buf.len()` later; as we only need 1 loop)
            read_bytes += 1;
        }
        Ok(read_bytes)
    }
}

impl Write for SerialPort {
    type Error = util::Void;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut written_bytes = 0;
        for byte in buf {
            // write each byte in the buffer to the port
            self.write_byte(*byte);
            // and increment the number of bytes written (this should be faster
            // than calling `buf.len()` later; as we only need 1 loop)
            written_bytes += 1;
        }
        Ok(written_bytes)
    }
}
