#version 450

layout(set = 2, binding = 0) uniform sampler2D texture_atlas;

// One block, not several: the engine treats a material's uniform payload as
// a single opaque byte blob (MaterialDesc::uniforms), which is what makes
// materials hashable and dedupable. Separate blocks would need a
// slot-indexed payload on the Rust side.
//
// uv_rect is (min.xy, max.xy), matching Image::uv_min / Image::uv_max.
// pbr is (metallic, unused, unused, unused) — vec4-padded so more PBR
// inputs can land here later without reshuffling this block.
layout(set = 3, binding = 0) uniform Material {
    vec4 base_color;
    vec4 uv_rect;
    vec4 pbr;
};

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_view_dir;

layout(location = 0) out vec4 frag_color;

void main() {
    vec3 n = normalize(v_normal);
    vec3 v = normalize(v_view_dir);
    vec3 light_dir = normalize(vec3(0.35, 0.8, -0.5));
    vec3 half_dir = normalize(light_dir + v);

    float metallic = clamp(pbr.x, 0.0, 1.0);

    // fract() keeps tiling uvs inside the sub-rect. A repeating sampler could
    // not do this — it would bleed into whatever else shares the atlas page.
    vec2 uv = mix(uv_rect.xy, uv_rect.zw, fract(v_uv));
    vec4 sampled = base_color * texture(texture_atlas, uv);
    vec3 albedo = sampled.rgb;

    float ndotl = max(dot(n, light_dir), 0.0);
    float ndoth = max(dot(n, half_dir), 0.0);

    // Metallic-roughness convention, minus roughness (not wired yet, so a
    // fixed glossiness stands in for it): metals keep no diffuse response
    // and tint their reflectance by albedo instead of staying grey.
    vec3 diffuse = albedo * (1.0 - metallic);
    vec3 f0 = mix(vec3(0.04), albedo, metallic);

    float shininess = 48.0;
    vec3 specular = f0 * pow(ndoth, shininess);

    // No environment map to reflect, so a metal lit from directly behind
    // would otherwise go flat black. A small albedo-tinted ambient term
    // keeps it legible instead of pretending there's real IBL.
    vec3 ambient = albedo * mix(0.25, 0.08, metallic);

    vec3 color = ambient + diffuse * ndotl * 0.75 + specular;

    frag_color = vec4(color, sampled.a);
}
