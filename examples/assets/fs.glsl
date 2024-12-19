#version 330 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D screenTexture;
uniform float elapsed;

void main() {   
    vec2 uv = TexCoord;

    // Apply barrel distortion
    vec2 center = uv - 0.5;
    float dist = length(center);
    uv = uv + center * dist * dist * 0.1;

    // Edge smoothing factor using smoothstep with a thinner range
    float edgeFade = smoothstep(0.0, 0.01, min(uv.x, 1.0 - uv.x)); // Horizontal edges
    edgeFade *= smoothstep(0.0, 0.01, min(uv.y, 1.0 - uv.y));      // Vertical edges

    // Calculate background color based on elapsed time
    vec3 bgColor = vec3(sin(elapsed), cos(elapsed), sin(elapsed * 0.5));

    // Apply scanlines with smoother interpolation
    float scanline = sin(uv.y * 800.0) * 0.05; // Scanline effect
    vec3 color = texture(screenTexture, uv).rgb;
    color = mix(color, color - scanline, 0.5); // Blend the scanline effect

    // Apply the edge fade
    color *= edgeFade;

    // Blend the texture color with the background color
    color = mix(bgColor, color, edgeFade);

    // Set the final color to the background color based on elapsed time
    FragColor = vec4(mix(bgColor, color, edgeFade), 1.0);
}