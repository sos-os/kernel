//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Macros for kernel-level logging

#[macro_export]
macro_rules! kinfo {
    ( dots: $dots:expr, target: $target:expr, $status:expr ) => {
        // {
        //     use core::fmt::Write;
        //
        //     // suppress warnings because we don't care if there's no serial port
        //     let _ = write!( $crate::arch::drivers::serial::COM1.lock()
        //                   , "[info] {}: {} {}"
        //                   , module_path!()
        //                   , $msg, $status);
            print!("{:<38}{:>40}", concat!($dots, $target), $status );
            info!(target: $target, $status);
        // }
    };
    ( dots: $dots:expr, $($args:tt)* ) => {
        // {
        //     use core::fmt::Write;
        //
        //     // suppress warnings because we don't care if there's no serial port
        //     let _ = write!( $crate::arch::drivers::serial::COM1.lock()
        //                   , "[info] {}: {}"
        //                   , module_path!()
        //                   , format_args!($($args)*));
            print!("{}{}", $dots, format_args!($($args)*));
            info!( ($args)* );
        // }
    };
    ( $($args:tt)* ) => {
    //     {
    //         use core::fmt::Write;
    //
    //         // suppress warnings because we don't care if there's no serial port
    //         let _ = write!( $crate::arch::drivers::serial::COM1.lock()
    //                       , "[info] {}: {}"
    //                       , module_path!()
    //                       , format_args!($($args)*));
            print!( $($args)* );
            info!( $($args)* );
        // }
    };
}
#[macro_export]
macro_rules! kinfoln {
    ( dots: $dots:expr, target: $target:expr, $status:expr ) => {
        // {
        //     use core::fmt::Write;
        //
        //     // suppress warnings because we don't care if there's no serial port
        //     let _ = write!( $crate::arch::drivers::serial::COM1.lock()
        //                   , "[info] {}: {} {}"
        //                   , module_path!()
        //                   , $msg, $status);
            println!("{:<38}{:>40}", concat!($dots, $target), $status );
            info!(target: $target, $status);
        // }
    };
    ( dots: $dots:expr, $($args:tt)* ) => {
        // {
        //     use core::fmt::Write;
        //
        //     // suppress warnings because we don't care if there's no serial port
        //     let _ = write!( $crate::arch::drivers::serial::COM1.lock()
        //                   , "[info] {}: {}"
        //                   , module_path!()
        //                   , format_args!($($args)*));
            println!("{}{}", $dots, format_args!($($args)*));
            info!( $($args)* );
        // }
    };
    ( $($args:tt)* ) => {
    //     {
    //         use core::fmt::Write;
    //
    //         // suppress warnings because we don't care if there's no serial port
    //         let _ = write!( $crate::arch::drivers::serial::COM1.lock()
    //                       , "[info] {}: {}"
    //                       , module_path!()
    //                       , format_args!($($args)*));
            println!( $($args)* );
            info!( $($args)* );
        // }
    };
}
