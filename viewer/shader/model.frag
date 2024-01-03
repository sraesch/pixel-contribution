    #version 330 core

//------------------------------------------
// UNIFORMS
//------------------------------------------

//------------------------------------------
// INPUTS
//------------------------------------------

in vec3 var_pos;

//------------------------------------------
// OUTPUT
//------------------------------------------

out vec4 frag_color;

void main() {
    // compute flat normal
    vec3 xTangent = dFdx(var_pos);
    vec3 yTangent = dFdy(var_pos);
    vec3 faceNormal = normalize(cross(xTangent, yTangent));

    frag_color = vec4(faceNormal / 2.0 + 0.5, 1.0);
}