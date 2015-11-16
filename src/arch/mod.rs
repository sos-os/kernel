
#[cfg(target_arch="x86_64")]
mod x86_64;

#[cfg(target_arch="x86_64")]
pub use self::x86_64::{ cpu, drivers };

#[cfg(target_arch = "x86")]
mod x86;

#[cfg(target_arch = "x86")]
pub use self::x86::{ cpu, drivers };
