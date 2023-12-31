use std::ops::Range;

use log::info;

use crate::{
    render::Page,
    scene::Scene,
    spatial::PageReference,
    stats::{StatsNode, StatsNodeTrait},
};

use super::{Chunk, GeometrySlice, PageReferences};

/// Creates pages, where each page is a single object.
///
/// # Arguments
/// * `scene` - The scene to create the pages for.
/// * `stats` - The stats node to log the timing for creating the pages.
pub fn compute_sorting(scene: &Scene, stats: StatsNode) -> Vec<Page> {
    let _t = stats.register_timing();

    let page_references = create_simple_pages(scene);

    page_references.create_pages()
}

/// Creates and returns simple page references by turning each instance into a single page.
///
/// # Arguments
/// * `scene` - The scene whose instances are turned into pages.
fn create_simple_pages(scene: &Scene) -> PageReferences<'_> {
    info!("Chunkify...");
    let chunks = create_simple_chunks(scene);
    info!("Chunkify...DONE");

    info!("Create page references...");
    let num_chunks = chunks.len();
    let pages = (0..num_chunks)
        .map(|index| PageReference {
            chunks: Range {
                start: index,
                end: index + 1,
            },
        })
        .collect();

    let page_references = PageReferences::new(chunks, pages);
    info!("Create page references...DONE");

    page_references
}

/// Creates simple chunks by turning each instance into a single chunk.
///
/// # Arguments
/// * `scene` - The instance of the scene are turned into chunks.
fn create_simple_chunks(scene: &Scene) -> Vec<Chunk<'_>> {
    let instances = scene.get_instances();
    let geometries = scene.get_geometries();

    let chunks = instances
        .iter()
        .enumerate()
        .map(|(instance_index, instance)| {
            let geometry = &geometries[instance.geometry_index];
            let id = instance_index as u32;
            let transform = instance.transform;

            let vertex_range = Range {
                start: 0,
                end: geometry.positions.len(),
            };

            let triangle_range = Range {
                start: 0,
                end: geometry.triangles.len(),
            };

            let geo_slice = GeometrySlice {
                geometry,
                vertex_range,
                triangle_range,
            };

            Chunk {
                geo_slice,
                id,
                transform,
            }
        })
        .collect();

    chunks
}
