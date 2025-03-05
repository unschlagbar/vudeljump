#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inSize;
layout(location = 2) in uint inId;
layout(location = 3) in uint inUvStart;
layout(location = 4) in uint inUvSize;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out uint fragUvStart;
layout(location = 2) out uint fragUvSize;

void main() {
    vec2 uv = vec2(((gl_VertexIndex << 1) & 2) >> 1, (gl_VertexIndex & 2) >> 1);
    gl_Position = ubo.proj * ubo.view * vec4(uv * inSize + inPosition, 0.0, 1.0);
    fragTexCoord = uv;
    fragUvStart = inUvStart;
    fragUvSize = inUvSize;
}