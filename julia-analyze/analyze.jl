#!/usr/bin/env julia
using Plots

# Install plotlyjs
import Pkg;
Pkg.add("PlotlyJS");

plotlyjs()

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

function visualize_heatmap(map::Array{Float32,2})
    heatmap(map, aspect_ratio=1, c=:viridis)
end

function visualize_3d(map::Array{Float32,2})
    surface(map, c=:viridis)
end

# function main()
#     # Parse the input arguments, i.e., the file name of the pixel contribution data.
#     # Return an error if the number of arguments is not 1 and print the usage.
#     if length(ARGS) != 1
#         println("Usage: analyze.jl <filename>")
#         return
#     end

#     filename = ARGS[1]

#     println("Loading pixel contribution data from $filename")
#     maps = load_pixel_contrib(filename)
#     num_maps = length(maps)
#     println("Loaded $num_maps maps")

#     println("Visualizing the first map")

#     plotlyjs()
#     p = visualize_heatmap(maps[1][2])

#     gui(p)

# end

# main()