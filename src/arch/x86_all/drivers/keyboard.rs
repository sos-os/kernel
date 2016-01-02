use super::super::cpu::Port;
use io::keyboard;

/// A PS/2 keyboard state
pub struct Keyboard { /// Port for reading data from the keyboard
                      data_port: Port
                    , /// Port for sending control signals to the keyboard
                      control_port: Port
                    , /// The keyboard's modifier keys
                      pub state: keyboard::Modifiers
                    }
