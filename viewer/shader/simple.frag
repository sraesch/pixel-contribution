    #version 330 core

out vec4 frag_color;

in vec2 texCoord;

uniform sampler2D uniform_texture;

void main() {
    vec3 c = vec3(texture(uniform_texture, texCoord));
    frag_color = vec4(c, 1.0);
}