#version 300 es

in vec2 in_position;
in vec4 in_instance_position;
in vec4 in_instance_texcoord;
in float in_instance_rotation;

uniform vec2 view_position;
uniform vec2 view_size;

out vec2 uv;

void main() {
    vec2 uv_offset = in_instance_texcoord.xy;
    vec2 uv_size = in_instance_texcoord.zw;

    uv = vec2(
        uv_offset.x + (in_position.x * uv_size.x),  
        uv_offset.y + (in_position.y * uv_size.y)
    );

    uv.x += in_instance_rotation * 0.0;

    float x = (in_position.x * in_instance_position.z) - (in_instance_position.z / 2.0);
    float y = (in_position.y * in_instance_position.w) - (in_instance_position.w / 2.0);
    float x2 = (x*cos(in_instance_rotation)) - (y*sin(in_instance_rotation));
    float y2 = (x*sin(in_instance_rotation)) + (y*cos(in_instance_rotation));

    vec2 pos = vec2(
        view_position.x + (in_instance_position.x + x2),
        view_position.y + (in_instance_position.y + y2)
    );

    pos = ((pos / view_size) * 2.0) - 1.0;

    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
