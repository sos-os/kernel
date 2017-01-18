//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! BIOS Data Area
//!
//! see [the OS Dev Wiki]
//! (http://wiki.osdev.org/Memory_Map_(x86)#BIOS_Data_Area_.28BDA.29)
//! for more information.

type Word = u16;

pub mod ports {
    use super::Word;
    const PORTS_ADDR: usize = 0x0400;

    lazy_static! {
        /// BIOS Data Area that stores the addresses of serial and parallel ports
        static ref PORTS: &'static Ports
            = unsafe { &*(PORTS_ADDR as *const Ports) };
    }

    unsafe impl Send for Ports {}
    unsafe impl Sync for Ports {}

    /// Addresses of ports stored in the BIOS Data Area.
    ///
    #[repr(C)]
    struct Ports {
        /// Port address of the `COM1` serial port
        com1: Word
      , /// Port address of the `COM2` serial port
        com2: Word
      , /// Port address of the `COM3` serial port
        com3: Word
      , /// Port address of the `COM4` serial port
        com4: Word
      , /// Port address of the `LPT1` parallel port
        lpt1: Word
      , /// Port address of the `LPT2` parallel port
        lpt2: Word
      , /// Port address of the `LPT3` parallel port
        lpt3: Word
    }
    macro_rules! port_addr {
        ( $($name: ident),+ ) => { $(
            #[inline]
            pub fn $name() -> Option<u16> {
                    match PORTS.$name {
                        0 => None
                      , n => Some(n)
                    }
                }
        )+ }
    }

    port_addr! { com1, com2, com3, com4, lpt1, lpt2, lpt3 }


}
