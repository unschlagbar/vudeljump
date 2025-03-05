#version 450

layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in flat uint fragUvStart;
layout(location = 2) in flat uint fragUvSize;

layout(location = 0) out vec4 outColor;

void main() {
    vec2 uv_start = vec2(fragUvStart & 0xFFFF, (fragUvStart >> 16) & 0xFFFF);
    vec2 uv_size = vec2(fragUvSize & 0xFFFF, (fragUvSize >> 16) & 0xFFFF);
    // Ausgabe der Farbe mit Alpha = 1.0
    outColor = texture(texSampler, (fragTexCoord * uv_size + uv_start) / 1024);
}