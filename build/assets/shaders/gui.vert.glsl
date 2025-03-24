#version 300 es
    
in vec2 in_position;
in vec2 in_uv;

uniform vec2 view_size;

out vec2 uv;

void main() {
    uv = in_uv;
    gl_Position = vec4((in_position / view_size * 2.0) - vec2(1.0), 0.0, 1.0);
}
