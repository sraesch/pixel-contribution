using Pkg

Pkg.add("Plots")
using Plots

Pkg.add("PlotlyJS")

plotlyjs()

Pkg.develop(path="./AnalyzePixelContrib")

using AnalyzePixelContrib


"""
Visualizes the pixel contribution map as a heatmap.
"""
function visualize_heatmap(contrib_map::ContribMap)
    heatmap(contrib_map.values, aspect_ratio=1, c=:viridis)
end

"""
Visualizes as heightmap using the surface function.
"""
function visualize_3d(contrib_map::ContribMap)
    surface(contrib_map.values, c=:viridis)
end