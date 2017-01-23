//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
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

use Port;
use spin::Mutex;

use core::mem::transmute;

/// Starting offset for PIC1
const OFFSET: u8 = 0x20;
/// Command port for the follower PIC (PIC2)
const FOLLOWER_CMD_PORT: u16 = 0xA0;
/// Command port for the leader PIC (PIC1)
const LEADER_CMD_PORT: u16 = OFFSET as u16;

/// Commands to send to the PIC
#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Command { /// Command to set the PIC to 8086 mode
               Mode8086 = 0x01
             , /// Command to initialize a PIC
               Init     = 0x11
             , /// Command that ends an interrupt request
               EndIRQ   = 0x20
             , /// Command to read the Interrupt Request Register
               ReadIRR  = 0x0a
             , /// Command to read the Interrupt Service Register
               ReadISR  = 0x0b
             }

/// List of IRQs on the x86.
///
/// See [here](https://en.wikibooks.org/wiki/X86_Assembly/Programmable_Interrupt_Controller) for more info.
#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum IRQ { /// System timer IRQ
               Timer        = OFFSET
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


/// A 8259 Programmable Interrupt Controller.
pub struct PIC {
    /// The base offset to which interrupts on this PIC are mapped
    offset: u8
  , /// The port on the CPU that sends commands to this PIC.
    command_port: Port<u8>
  , /// The port that sends and recieves data from the PIC
    data_port: Port<u8>
}

impl PIC {

    /// Construct a new leader PIC
    pub const fn leader() -> PIC {
        PIC { offset: OFFSET
            , command_port: Port::<u8>::new(LEADER_CMD_PORT)
            , data_port: Port::<u8>::new(LEADER_CMD_PORT + 1)
            }
    }

    /// Construct a new follower PIC
    pub const fn follower() -> PIC {
        PIC { offset: OFFSET + 8
            , command_port: Port::<u8>::new(FOLLOWER_CMD_PORT)
            , data_port: Port::<u8>::new(FOLLOWER_CMD_PORT + 1)
            }
    }

    /// Returns true if this PIC is the leader PIC
    #[inline]
    pub fn is_leader(&self) -> bool {
        self.offset == OFFSET
    }

    /// Send a command to the PIC
    #[inline]
    fn send_command(&self, command: Command) {
        self.command_port
            .write(command as u8)
    }

    /// Send a byte of data to the PIC
    #[inline]
    pub fn send_data(&self, data: u8) {
        self.data_port
            .write(data)
    }

    /// Send the "initialize" command to this PIC
    #[inline]
    pub fn initialize(&self) {
         self.send_command(Command::Init)
    }

    /// Read the contents of the ISR (Interrupt Service Register) from this PIC
    #[inline]
    pub fn read_isr(&self) -> u8 {
            self.send_command(Command::ReadISR);
            self.data_port.read()
    }

/// Read the contents of the IRR (Interrupt Request Register) from this PIC
    #[inline]
    pub fn read_irr(&self) -> u8 {
        self.send_command(Command::ReadIRR);
        self.data_port.read()
    }

}

/// Trait for something which is capable of handling a PIC IRQ
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

    fn end_interrupt(&self, _: IRQ) {
        let _ = self.send_command(Command::EndIRQ);
    }
}

/// A pair of PICs in cascade mode.
///
/// This is the standard configuration on all modern x86 systems.
struct BothPICs (PIC, PIC);

impl BothPICs {

    /// Constructs the system's pair of PICs
    const fn new() -> Self {
        BothPICs (PIC::leader(), PIC::follower())
    }

    /// Initialize the system's PICs.
    pub unsafe fn initialize(&mut self) {
        let wait_port = Port::<u8>::new(0x80);
        let wait = || { wait_port.write(0); };
        // helper macro to avoid writing repetitive code
        macro_rules! send {
            (pic0 => $data:expr) => {
                self.0.send_data($data);
                wait();
            };
            (pic1 => $data:expr) => {
                self.1.send_data($data);
                wait();
            };
        }

        // Read the default interrupt masks from PIC1 and PIC2
        let (saved_mask1, saved_mask2)
            = (self.0.data_port.read(), self.1.data_port.read());

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

impl IRQHandler for BothPICs {

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
static PICS: Mutex<BothPICs>
    = Mutex::new(BothPICs::new());

/// Initialize the system's Programmable Interrupt Controller
///
/// # Safety
///  - This should only ever be called by the kernel boot process.
///    Initializing the PICs once they have already been inicialized
///    will probably cause Bad Things to take place.
pub unsafe fn initialize() {
    PICS.lock()
        .initialize();
    kinfoln!(dots: " . . ", target: "Initializing PICs", "[ OKAY ]");

}

/// If an interrupt is being handled by the PICs, end that interrupt.
///
/// This is called by the interrupt handler at the end of all interrupts.
/// If the interrupt is not a PIC interrupt, it silently does nothing.
///
/// # Safety
///  - This should only be called by interrupt handler functions.
pub unsafe fn end_pic_interrupt(interrupt_id: u8) {
    let pics = PICS.lock();
    let irq: IRQ = transmute(interrupt_id);

    if pics.handles(irq) {
        pics.end_interrupt(irq)
    }
}
