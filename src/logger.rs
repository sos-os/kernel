//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};
use arch::drivers::serial;

use core::fmt::Write;

struct SerialLogger;

pub fn initialize() -> Result<(), log::SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            max_log_level.set(LogLevelFilter::Trace);
            &SerialLogger
        })
    }
}
pub fn shutdown() -> Result<(), log::ShutdownLoggerError> {
    log::shutdown_logger_raw().map(|_logger| {
    })
}


#[cfg(debug_assertions)]
impl log::Log for SerialLogger {

    #[inline] fn enabled(&self, _metadata: &LogMetadata) -> bool {
        true // TODO: for now?
    }

    #[inline]
    fn log(&self, record: &LogRecord) {
        let meta = record.metadata();
        match record.level() {
            LogLevel::Trace if self.enabled(meta) => {
                let location = record.location();
                let _ = write!( *serial::COM1.lock()
                              , "[ TRACE ][ {}:{} ] {}: {}\n"
                              , location.module_path(), location.line()
                              , meta.target()
                              , record.args() );
            }
          , LogLevel::Debug if self.enabled(meta) => {
                let _ = write!( *serial::COM1.lock()
                              , "[ DEBUG ] {}: {}\n"
                              , meta.target()
                              , record.args() );
            }
          , level => {
                let target = meta.target();
                let args = record.args();
                let _ = write!( *serial::COM1.lock()
                              , "[ {} ] {}: {}\n"
                              , level, target, args );
                // println!("{}: {}", target, args );
            }
        }
    }

}

#[cfg(not(debug_assertions))]
impl log::Log for SerialLogger {

    #[inline] fn enabled(&self, metadata: &LogMetadata) -> bool {
        false // TODO: for now
    }

    #[inline]
    fn log(&self, record: &LogRecord) {
        let meta = record.metadata();
        if self.enabled(meta) {
            println!("{}: {}", target, args );
            // TODO: should we log for com1 also?
        }
    }

}
