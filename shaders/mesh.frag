#version 450

layout(set = 2, binding = 0) uniform sampler2D texture_atlas;

// One block, not two: the engine treats a material's uniform payload as a
// single opaque byte blob (MaterialDesc::uniforms), which is what makes
// materials hashable and dedupable. Two blocks would need a slot-indexed
// payload on the Rust side.
//
// uv_rect is (min.xy, max.xy), matching Image::uv_min / Image::uv_max.
layout(set = 3, binding = 0) uniform Material {
    vec4 base_color;
    vec4 uv_rect;
};

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 frag_color;

void main() {
    vec3 n = normalize(v_normal);
    vec3 light_dir = normalize(vec3(0.35, 0.8, -0.5));
    float diffuse = max(dot(n, light_dir), 0.0);

    // fract() keeps tiling uvs inside the sub-rect. A repeating sampler could
    // not do this — it would bleed into whatever else shares the atlas page.
    vec2 uv = mix(uv_rect.xy, uv_rect.zw, fract(v_uv));
    vec4 albedo = base_color * texture(texture_atlas, uv);

    frag_color = vec4(albedo.rgb * (0.25 + 0.75 * diffuse), albedo.a);
}
