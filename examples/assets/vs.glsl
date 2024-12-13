#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;


out vec2 TexCoord;

void main() {
    // make the edges 'shake'
    float shake = 0.01;
    vec2 pos = aPos;
    pos.x += sin(aPos.y * 10.0) * shake;
    pos.y += cos(aPos.x * 10.0) * shake;
    gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);

    TexCoord = aTexCoord;
}
