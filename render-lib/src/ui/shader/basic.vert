#version 330 core

//------------------------------------------
// INPUTS
//------------------------------------------

layout(location = 0) in vec2 in_position;

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform mat3 uniform_transform_mat;

//------------------------------------------
// OUTPUT
//------------------------------------------

void main() {
    vec2 pos = vec2(uniform_transform_mat * vec3(in_position, 1.0));
    gl_Position = vec4(pos, 0.0, 1.0);
}
