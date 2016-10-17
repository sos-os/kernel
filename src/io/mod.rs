//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Kernel IO.
//!
//! This module should eventually abstract over architecture-specific
//! implementation.
pub mod term;
pub mod keyboard;

use arch::cpu;

use core::{ ops, fmt };
use core::marker::PhantomData;

// macro_rules! println {
//     ($fmt:expr) => (print!(concat!($fmt, "\n")));
//     ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
// }
//
// macro_rules! print {
//     ($($arg:tt)*) => ({
//             use core::fmt::Write;
//             $crate::io::term::CONSOLE.lock()
//                                      .write_fmt(format_args!($($arg)*))
//                                      .unwrap();
//     });
// }

/// The `Read` trait allows for reading bytes from a source.
///
/// Implementors of the `Read` trait are sometimes called 'readers'.
///
/// Readers are defined by one required method, `read()`. Each call to `read`
/// will attempt to pull bytes from this source into a provided buffer. A
/// number of other methods are implemented in terms of `read()`, giving
/// implementors a number of ways to read bytes while only needing to implement
/// a single method.
///
/// This is basically a brain-dead reimplementation of the standard
/// library's `Read` trait. Most of the methods available on the
/// standard lib's `Read` are not yet implemented.
pub trait Read {
    type Error;
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// This function does not provide any guarantees about whether it blocks
    /// waiting for data, but if an object needs to block for a read but cannot
    /// it will typically signal this via an `Err` return value.
    ///
    /// If the return value of this method is `Ok(n)`, then it must be
    /// guaranteed that `0 <= n <= buf.len()`. A nonzero `n` value indicates
    /// that the buffer `buf` has been filled in with `n` bytes of data from
    /// this source. If `n` is `0`, then it can indicate one of two scenarios:
    ///
    /// 1. This reader has reached its "end of file" and will likely no longer
    ///    be able to produce bytes. Note that this does not mean that the
    ///    reader will *always* no longer be able to produce bytes.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// No guarantees are provided about the contents of `buf` when this
    /// function is called, implementations cannot rely on any property of the
    /// contents of `buf` being true. It is recommended that implementations
    /// only write data to `buf` instead of reading its contents.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O or other error, an error
    /// variant will be returned. If an error is returned then it must be
    /// guaranteed that no bytes were read.
    fn read(&mut self, buf: &mut [u8])     -> Result<usize, Self::Error>;

    /// Read all bytes until EOF in this source, placing them into `buf`.
    ///
    /// All bytes read from this source will be appended to the specified buffer
    /// `buf`. This function will continuously call `read` to append more data
    /// to `buf` until `read` returns either `Ok(0)` or an error.
    ///
    /// If successful, this function will return the total number of bytes read.
    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// A trait for objects which are byte-oriented sinks.
///
/// Implementors of the `Write` trait are sometimes called 'writers'.
///
/// Writers are defined by two required methods, `write()` and `flush()`:
///
/// * The `write()` method will attempt to write some data into the object,
///   returning how many bytes were successfully written.
///
/// * The `flush()` method is useful for adaptors and explicit buffers
///   themselves for ensuring that all buffered data has been pushed out to the
///   'true sink'.
///
/// This is basically a brain-dead reimplementation of the standard
/// library's `Write` trait. Most of the methods available on the
/// standard lib's `Write` are not yet implemented.
pub trait Write {
    type Error;

    /// Write a buffer into this object, returning how many bytes were written.
    ///
    /// This function will attempt to write the entire contents of `buf`, but
    /// the entire write may not succeed, or the write may also generate an
    /// error. A call to `write` represents *at most one* attempt to write to
    /// any wrapped object.
    ///
    /// Calls to `write` are not guaranteed to block waiting for data to be
    /// written, and a write which would otherwise block can be indicated through
    /// an `Err` variant.
    ///
    /// If the return value is `Ok(n)` then it must be guaranteed that
    /// `0 <= n <= buf.len()`. A return value of `0` typically means that the
    /// underlying object is no longer able to accept bytes and will likely not
    /// be able to in the future as well, or that the buffer provided is empty.
    ///
    /// # Errors
    ///
    /// Each call to `write` may generate an I/O error indicating that the
    /// operation could not be completed. If an error is returned then no bytes
    /// in the buffer were written to this writer.
    ///
    /// It is **not** considered an error if the entire buffer could not be
    /// written to this writer.
    fn write(&mut self, buf: &[u8])       -> Result<usize, Self::Error>;
}

impl<'a, 'b, E> ops::Shl<&'a [u8]> for &'b mut Write<Error=E>
where E: fmt::Debug {
    type Output = Self;

    /// Fakes the C++ `<<` operator for IOStreams on Write.
    fn shl(self, rhs: &'a [u8]) -> Self::Output {
        self.write(rhs)
            .unwrap();
        self
    }
}

/// Safe typed port wrapper
pub struct Port<T> { raw_port: cpu::Port
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
                        Port { raw_port: cpu::Port::new(number)
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
