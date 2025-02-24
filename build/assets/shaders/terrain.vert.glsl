#version 300 es
    
in vec2 in_position;
in vec2 in_uv;

uniform vec2 view_position;
uniform vec2 view_size;
uniform vec2 chunk_position;

out vec2 uv;

void main() {
    float CELL_SIZE = 64.0;

    uv = in_uv;

    vec2 pos = view_position + chunk_position + (in_position * vec2(CELL_SIZE));
    pos = (pos / vec2(view_size.x, view_size.y)) * 2.0;
    pos -= vec2(1.0, 1.0);
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
