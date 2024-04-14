using Pkg

Pkg.add("Plots")
using Plots

Pkg.add("PlotlyJS")

plotlyjs()

include("AnalyzePixelContrib/src/AnalyzePixelContrib.jl")


"""
Visualizes the pixel contribution map as a heatmap.
"""
function visualize_heatmap(map::Array{Float32,2})
    heatmap(map, aspect_ratio=1, c=:viridis)
end

"""
Visualizes as heightmap using the surface function.
"""
function visualize_3d(map::Array{Float32,2})
    surface(map, c=:viridis)
end