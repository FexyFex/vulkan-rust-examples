#version 450


layout (location = 0) in vec2 inTexCoords;

layout(push_constant) uniform PushConstants{int dummy;};

layout (location = 0) out vec4 outColor;


void main() {
    outColor = vec4(inTexCoords, 0.5, 1.0);
}