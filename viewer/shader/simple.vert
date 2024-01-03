    #version 330 core

layout(location = 0) in vec2 in_position;

out vec2 texCoord;

void main() {
    texCoord = in_position;
    gl_Position = vec4(in_position * 2.0 - vec2(1.0, 1.0), 0.0, 1.0);
}