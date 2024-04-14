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

export load_pixel_contrib, ContribMaps, ContribMap