#version 300 es
    
in vec2 in_position;

uniform vec2 screen_size;
uniform vec2 chunk_position;

out vec2 uv;

void main() {
    float CELL_SIZE = 64.0;
    vec2 pos = chunk_position + (in_position * vec2(CELL_SIZE));
    pos = (pos / vec2(screen_size.x, screen_size.y)) * 2.0;
    pos -= vec2(1.0, 1.0);
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
