mod camera_data;
mod options;
mod ppm;

use std::{path::PathBuf, str::FromStr, time::Instant};

use anyhow::Result;
use camera_data::CameraData;
use clap::Parser;
use log::{error, info, LevelFilter};
use nalgebra_glm::Mat4;
use options::Options;
use rasterizer::{
    simple_rasterizer::SimpleRasterizer, Aabb, Frame, Histogram, RenderOptions, RenderStats,
    Renderer, Scene, Stats, StatsNodeTrait,
};

use crate::ppm::{write_depth_buffer, write_id_buffer};

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    simple_logging::log_to(std::io::stdout(), filter);
}

/// Creates a model-view matrix and perspective matrix with s.t. the given volume is fitted into the
/// defined view.
///
/// # Arguments
/// * `aabb` - The volume for which the view will be fitted to.
fn compute_fit_view(aabb: &Aabb) -> (Mat4, Mat4) {
    let mut camera_data = CameraData::new();
    camera_data.focus(aabb).unwrap();

    let model_view_matrix = camera_data.get_model_matrix();
    let projection_matrix = camera_data.get_projection_matrix();

    (model_view_matrix, projection_matrix)
}

/// Prints the given histogram to the log.
///
/// # Arguments
/// * `histogram` - The histogram to print.
/// * `render_options` - The render options used to compute the histograms.
fn print_histogram(histogram: &Histogram, render_options: &RenderOptions) {
    info!("Histogram:");
    let mut entries: Vec<_> = histogram.iter().cloned().enumerate().collect();
    entries.sort_by_key(|e| u32::MAX - e.1);

    let total = (render_options.frame_size * render_options.frame_size) as f64;
    for (id, num) in entries.iter() {
        if *num != 0 {
            let f = ((*num as f64) / total * 100000f64).round() / 1000f64;
            info!("ID={}, coverage={} %", id, f);
        }
    }
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

/// Renders a single image and saves it to the HDD as test.
///
/// # Arguments
/// * `scene` - The scene that is being used for computation.
/// * `program_options` - The options for the program.
/// * `culler_options` - The options for the culler to initialize.
fn render_and_save_single_image<R: Renderer>(
    scene: Scene,
    program_options: Options,
    render_options: RenderOptions,
) -> Result<()> {
    let occ_timings = Stats::root().get_child("render_image");
    let _t = occ_timings.register_timing();

    let mut renderer = R::new(Stats::root().get_child("render_image"));
    info!("Renderer: {}", renderer.get_name());

    // determine the overall number of triangles in the scene
    let num_triangles = {
        let geometries = scene.get_geometries();

        scene
            .get_instances()
            .iter()
            .map(|i| geometries[i.geometry_index].triangles.len())
            .sum()
    };

    let view = compute_fit_view(&scene.compute_aabb());

    // initialize the renderer
    {
        let _t = occ_timings.get_child("init").register_timing();

        info!("Initialize renderer...");
        let start = Instant::now();
        renderer.initialize(scene, render_options.clone())?;
        let duration = start.elapsed();
        info!("Initialize renderer...DONE in {} s", duration.as_secs_f32());
    }

    // compute the occlusion image
    let (stats, histograms, duration) = {
        let start = Instant::now();
        info!("render image...");

        let mut frame = Frame::new_empty(program_options.size_buffer, true);

        let mut stats = RenderStats::default();
        let mut histogram = Histogram::new();

        let s = {
            let _t = occ_timings.get_child("compute").register_timing();

            renderer.render_frame(&mut histogram, Some(&mut frame), view.0, view.1)?
        };

        // writing results to HDD
        {
            let _t = occ_timings.get_child("write").register_timing();

            let id_file = PathBuf::from_str("frame_id.ppm").unwrap();
            let depth_file = PathBuf::from_str("frame_depth.pgm").unwrap();

            info!("Write id buffer...");
            write_id_buffer(&id_file, &frame)?;
            info!("Write id buffer...DONE");

            if frame.get_depth_buffer().is_some() {
                info!("Write depth buffer...");
                write_depth_buffer(&depth_file, &frame)?;
                info!("Write depth buffer...DONE");
            }
        }

        stats += s;

        let duration = start.elapsed();

        info!(
            "Compute occlusion frame...DONE in {} s",
            duration.as_secs_f32()
        );

        (stats, histogram, duration.as_secs_f64())
    };

    info!("Stats:");
    info!("Processed Triangles: {}", stats.num_triangles);
    info!("Total Scene Triangles: {}", num_triangles);

    info!("Print histograms...");
    print_histogram(&histograms, &render_options);

    print_stats(duration, render_options.frame_size, num_triangles);

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

    scene.print_scene_stats();

    let render_options = options.get_render_options();
    render_and_save_single_image::<SimpleRasterizer>(scene, options, render_options)?;

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