use log::{Level, LevelFilter};
use std::io::Write;

use env_logger::fmt::style::AnsiColor;

fn get_log_level_color(level: Level) -> AnsiColor {
    match level {
        Level::Trace => AnsiColor::Cyan,
        Level::Debug => AnsiColor::Green,
        Level::Info => AnsiColor::Blue,
        Level::Warn => AnsiColor::Yellow,
        Level::Error => AnsiColor::Red,
    }
}

/// Initializes the program logging
pub fn initialize_logging(filter: LevelFilter) {
    env_logger::Builder::new()
        .format(|buf, record| {
            let log_level_color = get_log_level_color(record.level());
            let log_style = log_level_color.on_default();

            writeln!(
                buf,
                "{}:{} {} [{}{}{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                log_style.render(),
                record.level(),
                log_style.render_reset(),
                record.args()
            )
        })
        .filter_level(filter)
        .init();
}
