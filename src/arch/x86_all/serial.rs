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
//! that port, `Some(SerialPort)` will be returned. Otherwise, if the system
//! does not have that port number (such as `COM3` and `COM4`) on many machines,
//! `PortNum::new()` returns `None`.
//!
//! See [the OS Dev wiki](http://wiki.osdev.org/Serial_Ports) for more
//! information.

use spin::Mutex;

use super::bda;

use ::io::{Read, Write, Port};
use ::util;

/// Address of the BIOS Data Area (BDA)
/// where the serial port addresses are stored.
const BDA_ADDR: usize = 0x400;

lazy_static! {
    static ref BDA_SERIAL_INFO: [u16; 4]
        = unsafe { *(BDA_ADDR as *const [u16; 4]) };

    pub static ref COM1: Option<Mutex<SerialPort>>
        = PortNum::Com1.initialize().map(Mutex::new);

    pub static ref COM2: Option<Mutex<SerialPort>>
        = PortNum::Com2.initialize().map(Mutex::new);

    pub static ref COM3: Option<Mutex<SerialPort>>
        = PortNum::Com3.initialize().map(Mutex::new);

    pub static ref COM4: Option<Mutex<SerialPort>>
        = PortNum::Com4.initialize().map(Mutex::new);
}

/// Available serial ports.
///
/// These are used to create new serial ports, so that serial ports
/// cannot be created for an arbitrary memory location.
///
#[derive(Debug, Copy, Clone)]
#[repr(usize)]
enum PortNum { Com1 = 0
             , Com2 = 1
             , Com3 = 2
             , Com4 = 3
             }

impl PortNum {
    #[inline]
    fn get_port_addr(&self) -> Option<u16> {
        match bda::PORTS.com_ports[*self as usize] {
            n if n > 0 => Some(n)
          , _ => None
        }
    }

    /// Returns `Some(SerialPort)` if this port exists, `None` if it does not
    fn initialize(&self) -> Option<SerialPort> {
        self.get_port_addr().map(|port| {
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
        })
    }
}


/// A serial port
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
