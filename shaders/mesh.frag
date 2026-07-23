#version 450

layout(set = 2, binding = 0) uniform sampler2D texture_atlas;

layout(set = 3, binding = 0) uniform Material {
    vec4 base_color;
};

layout(set = 3, binding = 1) uniform TexRegion {
    vec4 uv_rect;
};

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 frag_color;

void main() {
    vec3 n = normalize(v_normal);
    vec3 light_dir = normalize(vec3(0.35, 0.8, -0.5));
    float diffuse = max(dot(n, light_dir), 0.0);

    vec2 uv = uv_rect.xy + fract(v_uv) * uv_rect.zw;
    vec4 albedo = base_color * texture(texture_atlas, uv);

    frag_color = vec4(albedo.rgb * (0.25 + 0.75 * diffuse), albedo.a);
}
