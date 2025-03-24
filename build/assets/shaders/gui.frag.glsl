#version 300 es

precision highp float;

in vec2 uv;

out vec4 outColor;

uniform sampler2D u_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec2 texcoord = uv / vec2(textureSize(u_texture, 0));
    vec3 msdf = texture(u_texture, texcoord).rgb;
    float dist = median(msdf.r, msdf.g, msdf.b);

    float dx = dFdx(uv.x);
    float dy = dFdy(uv.y);
    float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    float w = fwidth(dist) / 1.5;
    float opacity = smoothstep(0.5 - w, 0.5 + w, dist);

    outColor = vec4(opacity);
}
