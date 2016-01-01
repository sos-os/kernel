//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for x86 and x86_64 CPUs
use ::{io,util};

pub mod control_regs;

pub struct Port(u16);

impl Port {

    pub const unsafe fn new(number: u16) -> Port { Port(number) }

    /// Read a byte (8 bits) from this port
    pub unsafe fn in8(&self) -> u8 {
        let result: u8;
        asm!(  "in al, dx"
            :  "={al}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }
    /// Read a word (16 bits) from this port
    pub unsafe fn in16(&self) -> u16 {
        let result: u16;
        asm!(  "in ax, dx"
            :  "={ax}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }

    /// Read a long word (32 bits) from this port
    pub unsafe fn in32(&self) -> u32 {
        let result: u32;
        asm!(  "in eax, dx"
            :  "={eax}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }

    pub unsafe fn out8(&self, value: u8) {
         asm!(  "out dx, al"
             :: "{dx}"(self.0)
              , "{al}"(value)
             :: "intel"
              , "volatile" );
    }

    pub unsafe fn out16(&self, value: u16) {
         asm!(  "out dx, ax"
             :: "{dx}"(self.0)
              , "{ax}"(value)
             :: "intel"
              , "volatile" );
    }

    pub unsafe fn out32(&self, value: u32) {
         asm!(  "out dx, eax"
             :: "{dx}"(self.0)
              , "{eax}"(value)
             :: "intel"
              , "volatile" );
    }
}


impl io::Read for Port {
    type Error = util::Void;

    /// Reads a single byte into the given buffer
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        unsafe {
            Ok(match &mut *buf {
                // if the length of the buffer is 0, then obviously
                // no bytes were read
                []                  => 0
                // otherwise, read one byte into the head of the buffer
              , [ref mut head, _..] => { *head = self.in8(); 1 }
            })
        }
    }

    /// Reads a new byte into each position in the buffer.
    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_bytes = 0;
        for idx in buf.iter_mut() {
            // for each index in the buffer, read another byte from the port
            unsafe { *idx = self.in8(); }
            // and increment the number of bytes read (this should be faster
            // than calling `buf.len()` later; as we only need 1 loop)
            read_bytes += 1;
        }
        Ok(read_bytes)
    }

}

impl io::Write for Port {
    type Error = util::Void;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut written_bytes = 0;
        for byte in buf {
            // write each byte in the buffer to the port
            unsafe { self.out8(*byte); }
            // and increment the number of bytes written (this should be faster
            // than calling `buf.len()` later; as we only need 1 loop)
            written_bytes += 1;
        }
        Ok(written_bytes)
    }
}

/// A pointer to a descriptor table (IDT or GDT)
#[repr(C, packed)]
pub struct DTablePtr<T> { pub limit: u16
                        , pub base: T
                        }

/// A descriptor table (IDT or GDT)
pub trait DTable {
    unsafe fn load(&self);
}
