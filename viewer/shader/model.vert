    #version 330 core

//------------------------------------------
// INPUTS
//------------------------------------------

layout(location = 0) in vec3 in_position;

//------------------------------------------
// UNIFORMS
//------------------------------------------

uniform mat4 uniform_combined_mat;
uniform mat4 model_view_mat;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec3 var_pos;

void main() {
    var_pos = vec3(model_view_mat * vec4(in_position, 1.0));
    gl_Position = uniform_combined_mat * vec4(in_position, 1.0);
}
