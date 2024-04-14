module AnalyzePixelContrib

include("octahedron.jl")
include("contrib_maps.jl")

# function index_from_camera_dir(dir_x::Float32, dir_y::Float32, dir_z::Float32)::UInt32



#     let uv = encode_octahedron_normal(dir) * self.map_size as f32 - Vec2::new(0.5, 0.5);

#         let u = clamp(uv.x.round() as usize, 0, self.map_size - 1);
#         let v = clamp(uv.y.round() as usize, 0, self.map_size - 1);

#         v * self.map_size + u
# end

end