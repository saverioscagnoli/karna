#version 330 core

out vec4 FragColor;

uniform float time;

void main()
{
    // Change color over time
    float red = sin(time) * 0.5 + 0.5;
    float green = cos(time) * 0.5 + 0.5;
    float blue = sin(time + 3.14) * 0.5 + 0.5;

    FragColor = vec4(red, green, blue, 1.0);
}