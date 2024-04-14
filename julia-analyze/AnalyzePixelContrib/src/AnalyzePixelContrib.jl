module AnalyzePixelContrib

include("octahedron.jl")

"""
Loads the pixel contribution data from the specified file.
"""
function load_pixel_contrib(filename::String)::Vector{Tuple{Float32,Array{Float32,2}}}
    maps = []

    # Open the file in read-only and binary mode
    open(filename, "r") do file
        # Check if the first 4 bytes match "PCMP"
        magic = read(file, UInt32)
        if magic != 0x504D4350  # ASCII for PCMP
            throw("File does not start with 'PCMP' and magic number is $magic")
        end

        # Read version
        version = read(file, UInt32)
        if version != 1
            throw("Unsupported version $version")
        end

        num_maps = read(file, UInt32)

        # Read all the maps
        for i in 1:num_maps
            map_size = read(file, UInt32)
            angle = read(file, Float32)

            # Read the pixel contributions, which is an array of 32-bit floats of size map_size * map_size
            map = Array{Float32}(undef, map_size, map_size)
            read!(file, map)

            push!(maps, (angle, map))
        end
    end

    return maps
end

# function index_from_camera_dir(dir_x::Float32, dir_y::Float32, dir_z::Float32)::UInt32



#     let uv = encode_octahedron_normal(dir) * self.map_size as f32 - Vec2::new(0.5, 0.5);

#         let u = clamp(uv.x.round() as usize, 0, self.map_size - 1);
#         let v = clamp(uv.y.round() as usize, 0, self.map_size - 1);

#         v * self.map_size + u
# end

end