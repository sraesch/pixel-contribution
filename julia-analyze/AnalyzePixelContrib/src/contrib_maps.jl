struct ContribMap
    # The angle of the map
    angle::Float32

    # The contribution values of the map
    values::Matrix{Float32}
end

struct ContribMaps
    # The maps
    maps::Vector{ContribMap}
end


"""
Loads the pixel contribution data from the specified file.

### Input

- `filename` -- The name of the file to load the pixel contribution data from.

### Output

The pixel contribution maps.

"""
function load_pixel_contrib(filename::String)::ContribMaps
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
            values = Matrix{Float32}(undef, map_size, map_size)
            read!(file, values)

            push!(maps, ContribMap(angle, values))
        end
    end

    return ContribMaps(maps)
end

""""
Determines the two dimensional index in the contribution map based on the provided camera direction.

### Input

- `dir` -- The camera direction vector, i.e., the direction from the camera to the center of the
           bounding box.
- `map_size` -- The size of the contribution map, i.e., the number of pixels in one dimension.

### Output

The two dimensional index in the contribution map.

"""
function index_from_camera_dir(dir::Vector{Float64}, map_size::UInt32)::Vector{UInt32}
    if length(dir) != 3
        throw(ArgumentError("The input direction must have 3 components"))
    end

    if map_size <= 0
        throw(ArgumentError("The map size must be greater than 0"))
    end

    local uv = encode_octahedron_normal(dir) * Float64(map_size) - [0.5, 0.5]

    local u = clamp(round(UInt32, uv[1]), 0, map_size - 1)
    local v = clamp(round(UInt32, uv[2]), 0, map_size - 1)

    return [u + 1, v + 1]
end

"""
Determines the camera direction based on the provided two dimensional index in the contribution map.

### Input

- `in_uv` -- The two dimensional index in the contribution map.
- `map_size` -- The size of the contribution map, i.e., the number of pixels in one dimension.

### Output

The camera direction vector.

"""
function camera_dir_from_index(in_uv::Vector{UInt32}, map_size::UInt32)::Vector{Float64}
    if length(in_uv) != 2
        throw(ArgumentError("The input UV coordinates must have 2 components"))
    end

    if map_size <= 0
        throw(ArgumentError("The map size must be greater than 0"))
    end

    if in_uv[1] < 1 || in_uv[1] > map_size || in_uv[2] < 1 || in_uv[2] > map_size
        throw(ArgumentError("The UV coordinates must be in the range [1, map_size]"))
    end

    local uv = ([Float64(in_uv[1]), Float64(in_uv[2])] - [0.5, 0.5]) / Float64(map_size)
    @assert 0.0 <= uv[1] <= 1.0
    @assert 0.0 <= uv[2] <= 1.0

    return decode_octahedron_normal(uv)
end

export load_pixel_contrib, ContribMaps, ContribMap, index_from_camera_dir, camera_dir_from_index