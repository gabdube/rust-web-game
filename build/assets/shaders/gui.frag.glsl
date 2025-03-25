#version 300 es

precision highp float;

in vec2 uv;
in vec4 color;

out vec4 outColor;

uniform sampler2D fonts_texture;
uniform sampler2D images_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    int flags = int(color.a * 255.0);
    int is_font = flags & 0x1;

    // Image sampling
    vec2 images_texcoord = uv / vec2(textureSize(images_texture, 0));
    vec4 color_sample = texture(images_texture, images_texcoord);
    
    // Msdf font sampling
    vec2 fonts_texcoord = uv / vec2(textureSize(fonts_texture, 0));
    vec4 font_sample = texture(fonts_texture, fonts_texcoord);

    float dist = median(font_sample.r, font_sample.g, font_sample.b);
    float dx = dFdx(uv.x);
    float dy = dFdy(uv.y);
    float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    float w = fwidth(dist) / 1.5;
    float opacity = smoothstep(0.5 - w, 0.5 + w, dist);

    if (is_font > 0) {
        outColor = vec4(opacity) * color;
    } else {
        outColor = vec4((color_sample.rgb * color.rgb), 1.0) * color_sample.a;
    }
}
