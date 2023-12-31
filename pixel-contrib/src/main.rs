mod options;

use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use options::Options;
use rasterizer::{Scene, Stats, StatsNodeTrait};

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    simple_logging::log_to(std::io::stdout(), filter);
}

/// Runs the program.
fn run_program() -> Result<()> {
    let _t = Stats::root().register_timing();

    let options = parse_args()?;
    initialize_logging(LevelFilter::from(options.log_level));

    info!("Options:");
    options.dump_to_log();
    info!("-------");

    let scene = {
        let _t = Stats::root().get_child("load").register_timing();

        info!(
            "Load input file '{}'...",
            options.input_file.to_string_lossy()
        );
        let scene = Scene::new(&options.input_file)?;
        info!(
            "Load input file '{}'...DONE",
            options.input_file.to_string_lossy()
        );

        scene
    };

    scene.print_scene_stats();

    Ok(())
}

fn main() {
    match run_program() {
        Ok(()) => {
            info!("Stat:");
            info!("{}", format!("{}", *Stats::root().lock().unwrap()));
            info!("SUCCESS");
        }
        Err(err) => {
            error!("Error: {}", err);
            error!("FAILED");

            std::process::exit(-1);
        }
    }
}
