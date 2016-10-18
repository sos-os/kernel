// 64-bit x86_64 (long mode)
#[cfg(target_arch="x86_64")] mod x86_64;
#[cfg(target_arch="x86_64")] pub use self::x86_64::*;

// 32-bit x86 (protected mode)
// TODO: NYI
#[cfg(target_arch = "x86")] mod x86;
#[cfg(target_arch = "x86")] pub use self::x86::*;
