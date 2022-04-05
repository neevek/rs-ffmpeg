#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
mod ff_error;

mod codec;
mod format;
mod frame;
mod packet;

#[macro_use]
mod macros;

pub use codec::Decoder;
pub use format::{Input, Stream};
pub use frame::Frame;
pub use packet::Packet;
use std::sync::Once;

extern crate pretty_env_logger;

static INIT_LOGGER_ONCE: Once = Once::new();

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn init_logger(log_level: &str) {
    INIT_LOGGER_ONCE.call_once(|| LogHelper::init_logger(log_level.as_ref()));
}

#[cfg(not(target_os = "android"))]
macro_rules! colored_log {
    ($buf:ident, $record:ident, $term_color:literal, $level:literal) => {{
        let filename = $record.file().unwrap_or("unknown");
        let filename = &filename[filename.rfind('/').map(|pos| pos + 1).unwrap_or(0)..];
        writeln!(
            $buf,
            concat!($term_color, "{} [{}:{}] [", $level, "] {}\x1B[0m"),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
            filename,
            $record.line().unwrap_or(0),
            $record.args()
        )
    }};
}

struct LogHelper {}
impl LogHelper {
    #[cfg(not(target_os = "android"))]
    pub fn init_logger(log_level_str: &str) {
        use std::io::Write;
        let log_level_filter;
        match log_level_str.as_ref() {
            "D" => log_level_filter = log::LevelFilter::Debug,
            "I" => log_level_filter = log::LevelFilter::Info,
            "W" => log_level_filter = log::LevelFilter::Warn,
            "E" => log_level_filter = log::LevelFilter::Error,
            _ => log_level_filter = log::LevelFilter::Trace,
        }

        pretty_env_logger::formatted_timed_builder()
            .format(|buf, record| match record.level() {
                log::Level::Trace => colored_log!(buf, record, "\x1B[0m", "T"),
                log::Level::Debug => colored_log!(buf, record, "\x1B[92m", "D"),
                log::Level::Info => colored_log!(buf, record, "\x1B[34m", "I"),
                log::Level::Warn => colored_log!(buf, record, "\x1B[93m", "W"),
                log::Level::Error => colored_log!(buf, record, "\x1B[31m", "E"),
            })
            .filter(None, log_level_filter)
            .init();
    }

    #[cfg(target_os = "android")]
    pub fn init_logger(log_level_str: &str) {
        let log_level;
        match log_level_str.as_ref() {
            "D" => log_level = log::Level::Debug,
            "I" => log_level = log::Level::Info,
            "W" => log_level = log::Level::Warn,
            "E" => log_level = log::Level::Error,
            _ => log_level = log::Level::Trace,
        }

        android_logger::init_once(
            android_logger::Config::default()
                .with_min_level(log_level)
                .with_tag("rs-ffmpeg"),
        );
    }
}
