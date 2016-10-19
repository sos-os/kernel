//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza eisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
#![crate_name = "cpu"]
#![feature(const_fn)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(slice_patterns)]
#![no_std]

#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
extern crate spin;

extern crate util;
#[macro_use] extern crate vga;
#[macro_use] extern crate lazy_static;

use core::marker::PhantomData;

// 64-bit x86_64 (long mode)
#[cfg(target_arch="x86_64")] mod x86_64;
#[cfg(target_arch="x86_64")] pub use self::x86_64::*;

// 32-bit x86 (protected mode)
// TODO: NYI
#[cfg(target_arch = "x86")] mod x86;
#[cfg(target_arch = "x86")] pub use self::x86::*;

// ARM v7
// TODO: NYI
#[cfg(target_arch = "armv7")] mod armv7;
#[cfg(target_arch = "armv7")] pub use self::x86::*;

/// Safe typed port wrapper
pub struct Port<T> { raw_port: UnsafePort
                   , typ: PhantomData<T>
                   }

macro_rules! make_ports {
    ( $( $t:ty, $read:ident, $out:ident ),+ ) => {
        $(
            impl Port<$t> {
                #[inline]
                pub const fn new(number: u16) -> Self {
                    // TODO: can we check if the port number is valid
                    unsafe {
                        Port { raw_port: UnsafePort::new(number)
                             , typ: PhantomData::<$t>
                             }
                    }
                }

                #[inline]
                pub fn read(&self) -> $t {
                    unsafe { self.raw_port.$read() }
                }

                #[inline]
                pub fn write(&self, data: $t) {
                    unsafe { self.raw_port.$out(data) }
                }
            }
        )+
    }
}

make_ports! { u8, in8, out8
            , u16, in16, out16
            , u32, in32, out32
            }

#[cfg(arch="x86_64")]
make_ports! { u64, in64, out64 }
