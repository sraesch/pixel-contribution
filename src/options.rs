use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use log::{info, LevelFilter};

/// Workaround for parsing the different log level
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

/// CLI interface for determining the pixel contribution of the geometry from all views.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// The log level
    #[arg(short, value_enum, long, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// The input file to perform the occlusion culling on
    #[arg(short, long)]
    pub input_file: PathBuf,

    /// The number of threads used. By default, the number is being estimated.
    #[arg(short, long, default_value_t = 0usize)]
    pub num_threads: usize,

    /// The size of the quadratic frame buffer
    #[arg(short, long, default_value_t = 512usize)]
    pub size_buffer: usize,
}

impl Options {
    /// Dumps the options parameter to the log.
    pub fn dump_to_log(&self) {
        info!("Log-Level: {:?}", self.log_level);

        info!("input_file: {:?}", self.input_file);

        info!("num_threads: {}", self.num_threads);
        info!("size_buffer: {}", self.size_buffer);
    }
}
