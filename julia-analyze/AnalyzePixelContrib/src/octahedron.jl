import LinearAlgebra: normalize

""""
Simple helper function to flip the xy-coordinates of the octahedron normal encoding.

### Input

- `v1` -- The reference coordinate to decide if and how to flip the other coordinate.
- `v2` -- The coordinate to flip.

### Output

The flipped coordinate.

"""
function wrap_octahedron_normal_value(v1::Float64, v2::Float64)::Float64
    return (1.0 - abs(v2)) * (
        if v1 >= 0.0
            1.0
        else
            -1.0
        end
    )
end

"""
Consumes a normal and returns the encoded octahedron normal as a 2D vector in the range [0, 1].

### Input

- `in_normal` -- The normal to encode

### Output

A 2 element vector representing the octahedron encoded normal.

"""
function encode_octahedron_normal(in_normal::Vector{Float64})::Vector{Float64}
    if length(in_normal) != 3
        throw(ArgumentError("The input normal must have 3 components"))
    end

    local normal = normalize(in_normal)
    local abs_sum = abs(normal[1]) + abs(normal[2]) + abs(normal[3])

    normal[1] /= abs_sum
    normal[2] /= abs_sum

    if normal[3] < 0.0
        local tmp = normal[1]
        normal[1] = wrap_octahedron_normal_value(normal[1], normal[2])
        normal[2] = wrap_octahedron_normal_value(normal[2], tmp)
    end

    return [normal[1] * 0.5 + 0.5, normal[2] * 0.5 + 0.5]
end

"""
Consumes a normal encoded as octahedron in the range [0,1] and returns the decoded normal.

### Input

- `in_octahedron` -- The normal encoded as octahedron

### Output

The decoded normal.

"""
function decode_octahedron_normal(in_octahedron::Vector{Float64})::Vector{Float64}
    if length(in_octahedron) != 2
        throw(ArgumentError("The input octahedron normal must have 2 components"))
    end

    local octahedron = in_octahedron * 2.0 - [1.0, 1.0]
    local z = 1.0 - abs(octahedron[1]) - abs(octahedron[2])

    local x = if z >= 0.0
        octahedron[1]
    else
        wrap_octahedron_normal_value(octahedron[1], octahedron[2])
    end

    local y = if z >= 0.0
        octahedron[2]
    else
        wrap_octahedron_normal_value(octahedron[2], octahedron[1])
    end

    return normalize([x, y, z])
end

export encode_octahedron_normal, decode_octahedron_normal