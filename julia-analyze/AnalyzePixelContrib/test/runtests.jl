using AnalyzePixelContrib: encode_octahedron_normal, decode_octahedron_normal, camera_dir_from_index, index_from_camera_dir
using Test
using LinearAlgebra: dot

@testset "octahedron_encoding" begin
    local num = 20
    local pi = Float32(π)
    local pi_2 = Float32(π / 2)

    for i in 0:num+1
        # beta is angle between -PI/2 and +PI/2
        local beta = (i / num) * pi - pi_2

        # determine the radius on the 2D XY-plane
        local r2 = cos(beta)

        # determine value for Z
        local z = sin(beta)

        for j in 0:num
            # alpha is angle between 0 and 2 * PI
            local alpha = (j / num) * 2 * pi

            # determine value for X and Y
            local x = cos(alpha) * r2
            local y = sin(alpha) * r2

            local nrm = [x, y, z]

            # octahedron encoding
            local octahedron = encode_octahedron_normal(nrm)
            @assert 0 <= octahedron[1] <= 1
            @assert 0 <= octahedron[2] <= 1

            # octahedron decoding
            local nrm2 = decode_octahedron_normal(octahedron)

            # compute error
            local angle_error = abs(1 - dot(nrm, nrm2))

            @assert angle_error <= 1e-6 "Decoding error is too high"
        end
    end
end

@testset "test_camera_dir_index_mapping" begin
    local map_sizes::Vector{UInt32} = [16, 32, 64, 128, 256, 512, 1024]

    for map_size in map_sizes
        for u in 1:map_size
            for v in 1:map_size
                local uv::Vector{UInt32} = [u, v]

                local dir = camera_dir_from_index(uv, map_size)
                local uv2 = index_from_camera_dir(dir, map_size)

                @assert uv == uv2 "Index and UV coordinates do not match, expected $(uv) but got $(uv2)"
            end
        end
    end
end