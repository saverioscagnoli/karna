#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform float elapsed; 

void main() {
    // Apply a wobble effect using a sine wave
    float wobbleAmount = 0.05; // Adjust the wobble amount as needed
    vec2 wobble = vec2(sin(aPos.y * 10.0 + elapsed) * wobbleAmount, sin(aPos.x * 10.0 + elapsed) * wobbleAmount);
    vec2 wobbledPos = aPos + wobble;

    gl_Position = vec4(wobbledPos.x, wobbledPos.y, 0.0, 1.0);
    TexCoord = aTexCoord;
}