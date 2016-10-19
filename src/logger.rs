use log;
use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter};
use arch::drivers::serial;

use core::fmt::Write;

struct SerialLogger;

pub fn initialize() -> Result<(), log::SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            static LOGGER: SerialLogger = SerialLogger;
            max_log_level.set(LogLevelFilter::Trace);
            &SerialLogger
        })
    }
}
pub fn shutdown() -> Result<(), log::ShutdownLoggerError> {
    log::shutdown_logger_raw().map(|logger| {
        let logger = unsafe { &*(logger as *const SerialLogger) };
        // logger.flush();
    })
}


#[cfg(debug_assertions)]
impl log::Log for SerialLogger {

    #[inline] fn enabled(&self, metadata: &LogMetadata) -> bool {
        true // TODO: for now?
    }

    #[inline]
    fn log(&self, record: &LogRecord) {
        let meta = record.metadata();
        match record.level() {
            LogLevel::Trace if self.enabled(meta) => {
                let location = record.location();
                write!( *serial::COM1.lock()
                      , "[ TRACE ][ {}:{} ] {}: {}\n"
                      , location.module_path(), location.line()

                      , meta.target()
                      , record.args() );
            }
          , LogLevel::Debug if self.enabled(meta) => {
                write!( *serial::COM1.lock()
                      , "[ DEBUG ] {}: {}\n"
                      , meta.target()
                      , record.args() );
            }
          , level => {
                let target = meta.target();
                let args = record.args();
                write!( *serial::COM1.lock()
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
