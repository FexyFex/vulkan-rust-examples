#version 450


layout(push_constant) uniform PushConstants{ int dummy; };

layout (location = 0) out vec2 outTexCoords;

void main() {
    vec2 pos = vec2(1.0 ,1.0) * vec2(clamp(gl_VertexIndex, 0.0, 1.0), gl_VertexIndex % 2);
    outTexCoords = pos;
    gl_Position = vec4(pos, 0.5, 1.0);
}