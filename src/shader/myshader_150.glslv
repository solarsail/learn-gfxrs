#version 150 core

in vec3 a_Pos;
in vec2 a_TexCoord;

uniform Transform {
    mat4 u_Transform;
};

out vec2 v_TexCoord;

void main() {
    gl_Position = u_Transform * vec4(a_Pos, 1.0);
    v_TexCoord = a_TexCoord;
}