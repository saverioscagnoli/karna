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

    // Check if the distorted coordinates are outside the valid range
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        FragColor = vec4(0.0, 0.0, 0.0, 1.0); // Set color to black
    } else {
        // Apply scanlines with smoother interpolation
        float scanline = sin(uv.y * 800.0) * 0.05; // Scanline effect
        vec3 color = texture(screenTexture, uv).rgb;
        color = mix(color, color - scanline, 0.5); // Blend the scanline effect

        // Apply the edge fade
        color *= edgeFade;

        // Apply vignette with elapsed dependent color
        float vignette = smoothstep(0.5, 0.8, dist);
        vec3 vignetteColor = vec3(
            0.5 + 0.5 * sin(elapsed * 0.5),
            0.5 + 0.5 * sin(elapsed * 0.7),
            0.5 + 0.5 * sin(elapsed * 0.9)
        );
        color = mix(color, vignetteColor, vignette);

        FragColor = vec4(color, 1.0);
    }
}