    #version 330 core

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform sampler2D uniform_texture;

//------------------------------------------
// INPUTS
//------------------------------------------

in vec3 var_pos;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec4 frag_color;

float wrap_octahedron_normal_value(float v1, float v2) {
    return (1.0 - abs(v2)) * (v1 >= 0.0 ? 1.0 : -1.0);
}

/// Consumes a normal and returns the encoded octahedron normal as a 2D vector in the range [0, 1].
///
/// # Arguments
/// * `normal` - The normal to encode
vec2 encode_octahedron_normal(vec3 normal) {
    normal = normalize(normal);
    float abs_sum = abs(normal.x) + abs(normal.y) + abs(normal.z);

    normal.x /= abs_sum;
    normal.y /= abs_sum;

    if(normal.z < 0.0) {
        float tmp = normal.x;
        normal.x = wrap_octahedron_normal_value(normal.x, normal.y);
        normal.y = wrap_octahedron_normal_value(normal.y, tmp);
    }

    return vec2(normal.x * 0.5 + 0.5, normal.y * 0.5 + 0.5);
}

void main() {
    // compute texture coordinates
    vec2 texCoord = encode_octahedron_normal(normalize(var_pos));

    vec3 c = vec3(texture(uniform_texture, texCoord));
    frag_color = vec4(c, 1.0);
}