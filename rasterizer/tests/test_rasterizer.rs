use common::{load_two_cubes, test_culler};
use rasterizer::simple_rasterizer::SimpleRasterizer;

mod common;

#[test]
fn test_simple_rasterizer() {
    let scenario = load_two_cubes();
    test_culler::<SimpleRasterizer>(scenario);
}
