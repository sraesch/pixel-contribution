# Script in Julia to Analyze
Simple script in Julia to analyze pixel contribution data. In order to use the script, you need to install Julia first. Then you can load the maps and visualize them like this:

```julia
include("analyze.jl")

# Load a set of maps
maps = load_pixel_contrib("../test_data/contrib_maps/duck_contrib_map.bin");

# Visualize one map as a 3D surface
visualize_3d(maps[1][2])
```