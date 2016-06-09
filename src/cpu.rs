pub use arch::cpu::*;

use memory::PAddr;

pub struct Stack { pub base: PAddr
                 , pub top: PAddr
                 }
