#version 300 es

precision highp float;

in vec2 uv;
flat in vec4 texcoord_bounds;
flat in int data;

out vec4 outColor;

uniform sampler2D u_texture;
    
void main() {
    vec2 tex_size = vec2(textureSize(u_texture, 0));
    vec2 tex_coord = uv / tex_size;
    vec4 color = texture(u_texture, tex_coord);
   
    // Outline options
    // Because textures are in an atlas, coordinates must be clamped in the sprite
    // or else we're going sample another texture
    vec2 pixel = vec2(3.0) / tex_size;
    vec4 bounds = texcoord_bounds / vec4(tex_size, tex_size);
    float outline = 0.0;
    outline += texture(u_texture, vec2(min(tex_coord.x + pixel.x, bounds.z), tex_coord.y)).a;
    outline += texture(u_texture, vec2(max(tex_coord.x - pixel.x, bounds.x), tex_coord.y)).a;
    outline += texture(u_texture, vec2(tex_coord.x, min(tex_coord.y + pixel.y, bounds.w))).a;
    outline += texture(u_texture, vec2(tex_coord.x, max(tex_coord.y - pixel.y, bounds.y))).a;

    float show_outline = float(data & 1);
    float sprite_mask = ceil(color.a - 0.7); // -0.7 remove the transparent parts of the sprite from the sprite mask
    float outline_mask = min(outline * (1.0 - sprite_mask), 1.0); // outline mask does not include sprite
    vec3 outline_color = vec3(1.0, 1.0, 1.0);
    color.rgb = mix(color.rgb, mix(color.rgb, outline_color, outline_mask), show_outline);
    color.a = mix(color.a, mix(color.a, 1.0, outline_mask), show_outline);

    outColor = color;
}
