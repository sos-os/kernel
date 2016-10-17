//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Serial port driver
//!
//! Create new serial ports by calling `PortNum::new()`. If the system supports
//! that port, `Some(Port)` will be returned. Otherwise, if the system
//! does not have that port number (such as `COM3` and `COM4`) on many machines,
//! `PortNum::new()` returns `None`.
//!
//! See [the OS Dev wiki](http://wiki.osdev.org/Serial_Ports) for more
//! information.

use spin::Mutex;

use core::fmt;

use ::arch::bda;
use ::io::Port;
use ::io;
use ::util;
//
// /// Address of the BIOS Data Area (BDA)
// /// where the serial port addresses are stored.
// const BDA_ADDR: usize = 0x400;

lazy_static! {
    // static ref BDA_SERIAL_INFO: [u16; 4]
    //     = unsafe { *(BDA_ADDR as *const [u16; 4]) };
    // TODO: serial ports are currently behind a mutex; can they be read-write
    //       locked instead? I think multiple threads should be able to read
    //       from a serial port at the same time without causing trouble?
    //          - eliza, 10/9/2016
    pub static ref COM1: Option<Mutex<SerialPort>>
        = bda::ports::com1().map(SerialPort::new).map(Mutex::new);

    pub static ref COM2: Option<Mutex<SerialPort>>
        = bda::ports::com2().map(SerialPort::new).map(Mutex::new);

    pub static ref COM3: Option<Mutex<SerialPort>>
        = bda::ports::com3().map(SerialPort::new).map(Mutex::new);

    pub static ref COM4: Option<Mutex<SerialPort>>
        = bda::ports::com4().map(SerialPort::new).map(Mutex::new);
}



/// A serial port
pub struct SerialPort { data_port: Port<u8>
                      , status_port: Port<u8>
                      }

impl SerialPort {

    fn new(port: u16) -> SerialPort {
         // Disable all interrupts
        Port::<u8>::new(port + 1).write(0x00);
        // Enable DLAB (set baud rate divisor)
        Port::<u8>::new(port + 3).write(0x80);
        // Set divisor to 38400 baud
        Port::<u8>::new(port + 0).write(0x03); // divisor hi byte
        Port::<u8>::new(port + 1).write(0x00); // divisor lo byte
        // 8 bits, no parity, one stop bit
        Port::<u8>::new(port + 3).write(0x03);
        // Enable FIFO, clear them, with 14-byte threshold
        Port::<u8>::new(port + 2).write(0xC7);
        // IRQs enabled, RTS/DSR set
        Port::<u8>::new(port + 4).write(0x0B);
        Port::<u8>::new(port + 1).write(0x01);

        SerialPort { data_port: Port::<u8>::new(port)
                   , status_port: Port::<u8>::new(port + 5)
                   }
    }

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

impl io::Read for SerialPort {
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

impl io::Write for SerialPort {
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

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // TODO: more robust error handling here
            self.write_byte(byte);
        }
        Ok(())
    }
}
