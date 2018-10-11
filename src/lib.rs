//! Logger implementation for low level kernel log (using `/dev/kmsg`)
//!
//! Usually intended for low level implementations, like [systemd generators][1],
//! which have to use `/dev/kmsg`:
//!
//! > Since syslog is not available (see above) write log messages to /dev/kmsg instead.
//!
//! [1]: http://www.freedesktop.org/wiki/Software/systemd/Generators/
//!
//! # Examples
//!
//! ```toml
//! [dependencies]
//! log = "*"
//! kernlog = "*"
//! ```
//! 
//! ```rust
//! #[macro_use]
//! extern crate log;
//! extern crate kernlog;
//! 
//! fn main() {
//!     kernlog::init().unwrap();
//!     warn!("something strange happened");
//! }
//! ```
//! Note you have to have permissions to write to `/dev/kmsg`,
//! which normal users (not root) usually don't.
//! 
//! If compiled with nightly it can use libc feature to get process id
//! and report it into log. This feature is unavailable for stable release
//! for now. To enable nightly features, compile with `--features nightly`:
//!
//! ```toml
//! [dependencies.kernlog]
//! version = "*"
//! features = ["nightly"]
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate log;

use std::fs::{OpenOptions, File};
use std::io::Write;
use std::sync::Mutex;

use log::{Log, Metadata, Record, Level, LevelFilter, SetLoggerError};

/// Kernel logger implementation
pub struct KernelLog {
    kmsg: Mutex<File>,
    maxlevel: LevelFilter
}

impl KernelLog {
    /// Create new kernel logger
    pub fn new() -> KernelLog {
        KernelLog::with_level(LevelFilter::Info)
    }

    /// Create new kernel logger with error level filter
    pub fn with_level(filter: LevelFilter) -> KernelLog {
        KernelLog {
            kmsg: Mutex::new(OpenOptions::new().write(true).open("/dev/kmsg").unwrap()),
            maxlevel: filter
        }
    }

}

impl Log for KernelLog {
    fn enabled(&self, meta: &Metadata) -> bool {
        meta.level() <= self.maxlevel
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level: u8 = match record.level() {
            Level::Error => 3,
            Level::Warn => 4,
            Level::Info => 5,
            Level::Debug => 6,
            Level::Trace => 7,
        };

        let mut buf = Vec::new();
        writeln!(buf, "<{}>{}: {}", level, record.target(), record.args()).unwrap();

        if let Ok(mut kmsg) = self.kmsg.lock() {
            let _ = kmsg.write(&buf);
            let _ = kmsg.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut kmsg) = self.kmsg.lock() {
            let _ = kmsg.flush();
        }
    }
}

/// Setup kernel logger as a default logger
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Trace)
}

/// init KernLog with level
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    let logger = KernelLog::with_level(level.to_level_filter());
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{init};

    #[test]
    fn log_to_kernel() {
        init().unwrap();
        debug!("hello, world!");
    }
}
