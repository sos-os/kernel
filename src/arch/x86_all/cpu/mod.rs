//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Common functionality for `x86` and `x86_64` CPUs
//!
//! Note that while we support both the `x86` and `x86_64` platforms,
//! we only support 32-bit `x86` machines. SOS is a protected mode or
//! long mode OS, and will not run on early x86 machines such as 286s.
use ::{io,util};

macro_rules! cpu_flag {
    ($doc:meta, $flag:ident, $get:ident, $set:ident) => {
        #[$doc]
        pub unsafe fn $get() -> bool {
            read().contains($flag)
        }
        #[$doc]
        pub unsafe fn $set(set: bool) {
            let mut flags: Flags = read();
            if set {
                flags.insert($flag);
            } else {
                flags.remove($flag);
            }
            unsafe { write(flags) }
        }
    };
    ($doc:meta, $flag:ident, $get:ident) => {
        #[$doc]
        pub unsafe fn $get() -> bool {
            read().contains($flag)
        }
    }
}

pub mod control_regs;
pub mod segment;
pub mod dtable;
pub mod flags;
pub mod timer;

/// Represents an x86 privilege level.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
#[repr(u16)]
pub enum PrivilegeLevel { /// Ring 0 is the most privileged ring
                          KernelMode = 0
                        , Ring1 = 1
                        , Ring2 = 2
                        , /// Ring 3 is the least privileged ring
                          UserMode = 3

}

impl PrivilegeLevel {
    /// Returns the current I/O Privilege Level from `%eflags`/`%rflags`.
    #[inline] pub fn current_iopl() -> Self { flags::read().iopl() }
}

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
                &mut []                  => 0
                // otherwise, read one byte into the head of the buffer
              , &mut [ref mut head, _..] => { *head = self.in8(); 1 }
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
