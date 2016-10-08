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

const PORTS_ADDR: 0x0400;

/// BIOS Data Area that stores the addresses of serial and parallel ports
pub static PORTS: *const Ports
    = unsafe { PORTS_ADDR as *const PORTS };

/// Addresses of ports stored in the BIOS Data Area.
///
#[repr(C)]
pub struct Ports {
    /// Addresses of the `COM1`, `COM2`, `COM3`, and `COM4` serial ports
    pub com_ports: [u16; 4]
  , /// Addresses of the `LPT1`, `LPT2`, and `LPT3` parallel ports
    pub lpt_ports: [u16, 3]
}
