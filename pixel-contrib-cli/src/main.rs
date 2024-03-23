mod options;

use std::{io::Write, time::Instant};

use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use options::{Options, PixelContribColorMap};
use pixel_contrib::{
    compute_contribution_map, CameraConfig, GrayScaleColorMap, PixelContributionMapFile, PixelContributionMapImageExport, PixelContributionOptions, TurboColorMap
};
use pixel_contrib_types::PixelContributionMaps;
use rasterizer::{simple_rasterizer::SimpleRasterizer, Scene, Stats, StatsNodeTrait};

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(filter)
        .init();
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

    let camera_configs: Vec<CameraConfig> = options
        .camera
        .iter()
        .map(|fovy| {
            if *fovy > 0f32 {
                pixel_contrib::CameraConfig::Perspective { fovy: *fovy }
            } else {
                pixel_contrib::CameraConfig::Orthographic
            }
        })
        .collect();

    info!("Camera configs");
    for camera_config in &camera_configs {
        info!("  {}", camera_config.to_string());
    }

    let mut render_stats = Default::default();

    let mut contrib_maps = PixelContributionMaps::new();
    for camera_config in camera_configs.iter() {
        let contrib_option = PixelContributionOptions {
            render_options: render_options.clone(),

            num_threads: options.num_threads,
            contrib_map_size: options.size_pixel_contrib,
            camera_config: *camera_config,
        };

        let contrib_map = compute_contribution_map::<SimpleRasterizer>(
            scene,
            Stats::root(),
            &contrib_option,
            &mut render_stats,
        );

        let image_file_name = format!("contrib_map_angle_{}.png", camera_config.angle());

        match options.color_map {
            PixelContribColorMap::Grayscale => {
                info!("Write contribution images Grayscale '{}'", image_file_name);
                contrib_map.write_image(&image_file_name, GrayScaleColorMap::new())?;
            }
            PixelContribColorMap::Rgb => {
                info!("Write contribution images RGB '{}'", image_file_name);
                contrib_map.write_image(&image_file_name, TurboColorMap::new())?;
            }
        }

        contrib_maps.add_map(contrib_map);
    }

    let duration = start.elapsed();
    let secs = duration.as_secs_f64();
    print_stats(secs, render_options.frame_size, num_triangles);

    info!("Write contribution maps");
    contrib_maps.write_file("contrib_maps.bin")?;

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
