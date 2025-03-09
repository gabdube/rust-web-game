#version 300 es

in vec2 in_position;
in vec4 in_instance_position;
in vec4 in_instance_texcoord;
in int in_instance_data;

uniform vec2 view_position;
uniform vec2 view_size;

out vec2 uv;
flat out vec4 texcoord_bounds;
flat out int data;

void main() {
    data = in_instance_data;
    
    float flipped = float(in_instance_data & 2);
    vec2 uv_offset = in_instance_texcoord.xy;
    vec2 uv_size = in_instance_texcoord.zw;

    if (flipped > 0.0) {
        uv = vec2(
            uv_offset.x + ((1.0 - in_position.x) * uv_size.x),  
            uv_offset.y + (in_position.y * uv_size.y)
        );
    } else {
        uv = vec2(
            uv_offset.x + (in_position.x * uv_size.x),  
            uv_offset.y + (in_position.y * uv_size.y)
        );
    }

    texcoord_bounds = vec4(
        in_instance_texcoord.x,
        in_instance_texcoord.y,
        in_instance_texcoord.x + in_instance_texcoord.z,
        in_instance_texcoord.y + in_instance_texcoord.w
    );

    vec4 positions = vec4(view_position, 0.0, 0.0) + in_instance_position;

    positions = (positions / vec4(view_size.x, view_size.y, view_size.x, view_size.y)) * 2.0;
    float x = (positions.x - 1.0) + (in_position.x * positions.z);
    float y = (positions.y - 1.0) + (in_position.y * positions.w);
    gl_Position = vec4(x, -y, 0.0, 1.0);
}
