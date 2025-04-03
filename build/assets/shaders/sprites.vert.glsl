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
    
    float flipped = float((in_instance_data & 2) == 2);
    float pos_x = mix(in_position.x, 1.0 - in_position.x, flipped);

    vec2 uv_offset = in_instance_texcoord.xy;
    vec2 uv_size = in_instance_texcoord.zw;

    uv = vec2(
        uv_offset.x + (pos_x * uv_size.x),  
        uv_offset.y + (in_position.y * uv_size.y)
    );

    texcoord_bounds = vec4(
        in_instance_texcoord.x,
        in_instance_texcoord.y,
        in_instance_texcoord.x + in_instance_texcoord.z,
        in_instance_texcoord.y + in_instance_texcoord.w
    );

    vec2 pos = vec2(
        view_position.x + (in_instance_position.x + (in_position.x * in_instance_position.z)),
        view_position.y + (in_instance_position.y + (in_position.y * in_instance_position.w))
    );

    pos = ((pos / view_size) * 2.0) - 1.0;

    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
