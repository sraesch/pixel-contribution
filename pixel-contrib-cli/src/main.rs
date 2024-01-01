mod options;

use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use options::Options;
use pixel_contrib::{compute_contribution_map, PixelContributionOptions};
use rasterizer::{simple_rasterizer::SimpleRasterizer, Scene, Stats, StatsNodeTrait};

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    simple_logging::log_to(std::io::stdout(), filter);
}

/// Prints statistics about the overall computation.
///
/// # Arguments
/// * `secs` - The number of seconds the computation took.
/// * `size` - The size of the frame in pixels.
/// * `num_triangles` - The number of triangles in the scene.
fn print_stats(secs: f64, size: usize, num_triangles: usize) {
    let total_pixel = (size * size) as f64;

    let pixels_per_second = total_pixel / secs;
    let triangles_per_second = (num_triangles as f64) / secs;

    info!("Pixels per second: {}", pixels_per_second);
    info!("Triangles per second: {}", triangles_per_second);
}

/// Executes the pixel contribution program.
///
/// # Arguments
/// * `options` - The options for the program.
/// * `scene` - The scene to render.
fn execute_pixel_contribution_program(options: &Options, scene: &Scene) -> Result<()> {
    let start = Instant::now();

    // determine the overall number of triangles in the scene
    let num_triangles = {
        let geometries = scene.get_geometries();

        scene
            .get_instances()
            .iter()
            .map(|i| geometries[i.geometry_index].triangles.len())
            .sum()
    };

    let render_options = options.get_render_options();

    let contrib_options = PixelContributionOptions {
        render_options: render_options.clone(),
        contrib_map_size: options.size_pixel_contrib,
        fovy: 90f32.to_radians(),
    };

    let mut render_stats = Default::default();
    let contrib_map = compute_contribution_map::<SimpleRasterizer>(
        scene,
        Stats::root(),
        &contrib_options,
        &mut render_stats,
    );
    contrib_map.write_image("contrib_map.png")?;

    let duration = start.elapsed();
    let secs = duration.as_secs_f64();
    print_stats(secs, render_options.frame_size, num_triangles);

    Ok(())
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

    execute_pixel_contribution_program(&options, &scene)?;

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
