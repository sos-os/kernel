//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Support for the 8259 Programmable Interrupt Controller.
//!
//! The 8259 PIC controls all of our IO IRQs. The PIC recieves IRQ requests
//! and feeds them to the CPU in order. An x86 system typically has two PICs,
//! PIC1 and PIC2, each of which provides 8 IRQs. Two PICs provide us with 15
//! unique IRQs, because one interrupt on the leader PIC is linked to interrupts
//! on the follower PIC.
//!
//! Mistakes were made, and Intel boneheadedly decided that it was a good idea
//! to map the vector offset of PIC1 to 8, so that it maps to interrupts
//! 8 ... 15 in the IDT. This conflicts with some of the interrupt numbers
//! used by CPU exceptions. Therefore, we must remap the PIC vectors so that
//! PIC1 starts at 32 and PIC2 at 40.

use ::io::Write;
use super::super::Port;
use spin::Mutex;
use core::mem::transmute;

/// Starting offset for PIC1
const OFFSET: u8 = 0x20;

/// Commands to send to the PIC
#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Command { Mode8086 = 0x01
             , Init     = 0x11
             , EndIRQ   = 0x20
             , ReadIRR  = 0x0a
             , ReadISR  = 0x0b
             }

/// List of IRQs on the x86.
///
/// See here for more info:
/// https://en.wikibooks.org/wiki/X86_Assembly/Programmable_Interrupt_Controller
#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum IRQ { /// System timer IRQ
               Timer        = 0 + OFFSET
             , /// PS/2 keyboard controller
               PS2Keyboard  = 1 + OFFSET
             , /// PIC2 cascade IRQ
               Cascade      = 2 + OFFSET
             , /// COM2 serial port
               COM2         = 3 + OFFSET
             , /// COM1 serial port
               COM1         = 4 + OFFSET
             , /// Line printer 2
               LPT2         = 5 + OFFSET
             , /// Floppy disc controller
               Floppy       = 6 + OFFSET
             , /// Line printer 1
               LPT1         = 7 + OFFSET
             , /// CMOS clock
               RTCTimer     = 8 + OFFSET
             , /// PS/2 mouse controller
               PS2Mouse     = 12 + OFFSET
             , /// Floating-point Coprocessor
               FPU          = 13 + OFFSET
             , /// ATA channel 1
               PrimaryATA   = 14 + OFFSET
             , /// ATA channel 2
               SecondaryATA = 15 + OFFSET
             }



struct PIC {
    /// The base offset to which interrupts on this PIC are mapped
    offset: u8
  , /// The port on the CPU that sends commands to this PIC.
    command_port: Port
  , /// The port that sends and recieves data from the PIC
    data_port: Port
}

impl PIC {

    const fn leader() -> PIC {
        unsafe {
            PIC { offset: OFFSET
                , command_port: Port::new(0x20)
                , data_port: Port::new(0x21)
                }
        }
    }

    const fn follower() -> PIC {
        unsafe {
            PIC { offset: OFFSET + 8
                , command_port: Port::new(0xA0)
                , data_port: Port::new(0xA1)
                }
        }
    }

    #[inline]
    fn is_leader(&self) -> bool {
        self.offset == OFFSET
    }

    #[inline]
    fn send_command(&self, command: Command) {
        unsafe {
            self.command_port
                .out8(command as u8)
        }
    }

    #[inline]
    fn send_data(&self, data: u8) {
        unsafe {
            self.data_port
                .out8(data)
        }
    }

    #[inline]
    fn initialize(&self) {
        self.send_command(Command::Init)
    }

    #[inline]
    fn read_ISR(&self) -> u8 {
        self.send_command(Command::ReadISR);
        unsafe { self.data_port.in8() }
    }

    #[inline]
    fn read_IRR(&self) -> u8 {
        self.send_command(Command::ReadIRR);
        unsafe { self.data_port.in8() }
    }

}

trait IRQHandler {
    /// Returns whether or not this handler handles the given IRQ
    fn handles(&self, irq: IRQ) -> bool;
    /// End an interrupt request
    fn end_interrupt(&self, irq: IRQ);
}

impl IRQHandler for PIC {

    fn handles(&self, irq: IRQ) -> bool {
        self.offset <= (irq as u8) && (irq as u8) < self.offset + 8
    }

    fn end_interrupt(&self, irq: IRQ) {
        self.send_command(Command::EndIRQ)
    }
}

/// A pair of PICs in cascade mode.
///
/// This is the standard configuration on all modern x86 systems.
struct PICs (PIC, PIC);

impl PICs {
    const fn new() -> Self {
        PICs (PIC::leader(), PIC::follower())
    }

    /// Initialize the system's PICs.
    fn initialize(&mut self) {
        let mut wait_port = unsafe { Port::new(0x80) };
        let mut wait = || unsafe { wait_port.out8(0); };
        // helper macro to avoid writing repetitive code
        macro_rules! send {
            (pic0 => $data:expr) => {
                self.0.send_data($data)
                wait();
            };
            (pic1 => $data:expr) => {
                self.1.send_data($data)
                wait();
            };
        }

        // Read the default interrupt masks from PIC1 and PIC2
        let (saved_mask1, saved_mask2)
            = unsafe { (self.0.data_port.in8(), self.1.data_port.in8()) };

        // Send both PICs the 'initialize' command.
        self.0.initialize(); wait();
        self.1.initialize(); wait();

        // Each PIC then expects us to send it the following:
        // 1. the PIC's new vector offset
        send!(pic0 => self.0.offset);
        send!(pic1 => self.1.offset);
        // 2. number to configure PIC cascading
        send!(pic0 => 0x04);
        send!(pic1 => 0x02);
        // 3. command for 8086 mode
        send!(pic0 => Command::Mode8086 as u8);
        send!(pic1 => Command::Mode8086 as u8);
        // 4. finally, the mask we saved earlier
        send!(pic0 => saved_mask1);
        send!(pic1 => saved_mask2);
    }
}

impl IRQHandler for PICs {

    fn handles(&self, irq: IRQ) -> bool {
        self.0.handles(irq) ||
        self.1.handles(irq)
    }

    fn end_interrupt(&self, irq: IRQ) {
        if self.1.handles(irq) {
            self.1.end_interrupt(irq);
        }
        self.0.end_interrupt(irq);
    }

}

/// Global PIC instance and mutex
static PICS: Mutex<PICs> = Mutex::new(PICs::new());

/// Initialize the system's Programmable Interrupt Controller
pub fn initialize() {
    PICS.lock()
        .initialize()
}

/// If an interrupt is being handled by the PICs, end that interrupt.
///
/// This is called by the interrupt handler at the end of all interrupts.
/// If the interrupt is not a PIC interrupt, it silently does nothing.
pub unsafe fn end_pic_interrupt(interrupt_id: u8) {
    let pics = PICS.lock();
    let irq: IRQ = transmute(interrupt_id);

    if pics.handles(irq) {
        pics.end_interrupt(irq)
    }
}
