#version 150 core

in vec2 v_TexCoord;

uniform sampler2D u_Texture;

out vec4 Target0;

void main() {
    Target0 = texture(u_Texture, v_TexCoord);
}