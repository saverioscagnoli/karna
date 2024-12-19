#version 330 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D screenTexture;

void main() {   
    vec2 uv = TexCoord;
    vec3 col = texture(screenTexture, uv).rgb;
    FragColor = vec4(col, 1.0);
}


