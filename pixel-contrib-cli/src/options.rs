use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use log::{info, LevelFilter};
use rasterizer::RenderOptions;

/// The color map for the pixel contribution.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum PixelContribColorMap {
    /// The color map is a grayscale image.
    Grayscale,

    /// The color map is a RGB image.
    Rgb,
}

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

    /// The input file to perform the pixel contribution analysis on.
    #[arg(short, long)]
    pub input_file: PathBuf,

    /// The number of threads used. By default, the number is being estimated.
    #[arg(short, long, default_value_t = 0usize)]
    pub num_threads: usize,

    /// The size of the quadratic frame buffer
    #[arg(short, long, default_value_t = 512usize)]
    pub size_buffer: usize,

    /// The size of the map where the pixel contribution is stored for each pixel (i.e. view).
    #[arg(short = 'p', long, default_value_t = 256usize)]
    pub size_pixel_contrib: usize,

    /// The color map for encoding the pixel contribution
    #[arg(short, value_enum, long, default_value_t = PixelContribColorMap::Rgb)]
    pub color_map: PixelContribColorMap,

    /// The list of field of views for the camera in radians. 0 means orthographic camera.
    #[arg(short = 'a', long, value_parser, num_args = 0.., default_value = "1.5708")]
    pub camera: Vec<f32>,
}

impl Options {
    /// Extracts and returns the rendering options based on the program options.
    pub fn get_render_options(&self) -> RenderOptions {
        let num_threads: usize = if self.num_threads == 0 {
            (std::thread::available_parallelism().unwrap()).into()
        } else {
            self.num_threads
        };

        RenderOptions {
            num_threads,
            frame_size: self.size_buffer,
        }
    }

    /// Dumps the options parameter to the log.
    pub fn dump_to_log(&self) {
        info!("Log-Level: {:?}", self.log_level);

        info!("input_file: {:?}", self.input_file);

        info!("num_threads: {}", self.num_threads);
        info!("size_buffer: {}", self.size_buffer);
    }
}
