use std::hash::Hash;
use std::hash::Hasher;

use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::Color;
use crate::Image;
use crate::assets::Assets;

/// Anything that forces bind_graphics_pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassState {
    pub shader: gpu::ShaderRef,
    pub blend: gpu::BlendState,
    pub cull: Option<gpu::Cull>,
    pub topology: gpu::PrimitiveTopology,
    pub depth: Option<gpu::DepthState>,
}

impl Hash for PassState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.shader.hash(state);
        self.blend.hash(state);
        self.cull.hash(state);
        (self.topology as u32).hash(state);
        self.depth.hash(state);
    }
}

impl PassState {
    pub fn new(shader: gpu::ShaderRef) -> Self {
        Self {
            shader,
            blend: gpu::BlendState::ALPHA_BLENDING,
            cull: None,
            topology: gpu::PrimitiveTopology::TriangleList,
            depth: None,
        }
    }

    /// Defaults for opaque 3d geometry
    pub fn opaque_3d(shader: gpu::ShaderRef) -> Self {
        Self::new(shader)
            .with_cull(Some(gpu::Cull::Back))
            .with_depth(Some(gpu::DepthState::OPAQUE))
    }

    pub fn with_depth(mut self, depth: Option<gpu::DepthState>) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_blend(mut self, blend: gpu::BlendState) -> Self {
        self.blend = blend;
        self
    }

    pub fn with_cull(mut self, cull: Option<gpu::Cull>) -> Self {
        self.cull = cull;
        self
    }

    pub fn with_topology(mut self, topology: gpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn pipeline_desc(
        &self,
        layout: gpu::VertexLayout,
        format: gpu::TextureFormat,
    ) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: self.shader,
            vertex_layout: layout,
            blend: self.blend,
            topology: self.topology,
            cull: self.cull,
            format,
            depth: self.depth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask(f32),
    Blend,
}

impl Hash for AlphaMode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        if let Self::Mask(cutoff) = self {
            cutoff.to_bits().hash(state);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialDesc {
    pub base_color: Color,
    pub base_color_map: Option<Handle<Image>>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_map: Option<Handle<Image>>,
    pub reflectance: f32,

    pub normal_map: Option<Handle<Image>>,
    pub normal_scale: f32,
    pub occlusion_map: Option<Handle<Image>>,
    pub occlusion_strength: f32,

    pub emissive: Color,
    pub emissive_strength: f32,
    pub emissive_map: Option<Handle<Image>>,

    pub alpha_mode: AlphaMode,
    pub uv_offset: math::Vector2<f32>,
    pub uv_scale: math::Vector2<f32>,

    pub unlit: bool,
    pub double_sided: bool,
    pub vertex_colors: bool,

    pub shader: Option<gpu::ShaderRef>,
    pub depth_override: Option<gpu::DepthState>,
}

impl Default for MaterialDesc {
    fn default() -> Self {
        Self {
            base_color: Color::White,
            base_color_map: None,

            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_map: None,
            reflectance: 0.5,

            normal_map: None,
            normal_scale: 1.0,
            occlusion_map: None,
            occlusion_strength: 1.0,

            emissive: Color::Black,
            emissive_strength: 0.0,
            emissive_map: None,

            alpha_mode: AlphaMode::Opaque,
            uv_offset: math::Vector2::zero(),
            uv_scale: math::Vector2::one(),

            unlit: false,
            double_sided: false,
            vertex_colors: false,

            shader: None,
            depth_override: None,
        }
    }
}

/// Structural equality bit-cast through the float fields, so a
/// [`MaterialDesc`] can be used as a hash map key for interning without
/// pre-packing it into a byte blob.
impl PartialEq for MaterialDesc {
    fn eq(&self, other: &Self) -> bool {
        self.base_color == other.base_color
            && self.base_color_map == other.base_color_map
            && self.metallic.to_bits() == other.metallic.to_bits()
            && self.roughness.to_bits() == other.roughness.to_bits()
            && self.metallic_roughness_map == other.metallic_roughness_map
            && self.reflectance.to_bits() == other.reflectance.to_bits()
            && self.normal_map == other.normal_map
            && self.normal_scale.to_bits() == other.normal_scale.to_bits()
            && self.occlusion_map == other.occlusion_map
            && self.occlusion_strength.to_bits() == other.occlusion_strength.to_bits()
            && self.emissive == other.emissive
            && self.emissive_strength.to_bits() == other.emissive_strength.to_bits()
            && self.emissive_map == other.emissive_map
            && self.alpha_mode == other.alpha_mode
            && self.uv_offset == other.uv_offset
            && self.uv_scale == other.uv_scale
            && self.unlit == other.unlit
            && self.double_sided == other.double_sided
            && self.vertex_colors == other.vertex_colors
            && self.shader == other.shader
            && self.depth_override == other.depth_override
    }
}

impl Eq for MaterialDesc {}

fn hash_color<H: Hasher>(color: &Color, state: &mut H) {
    color.r.to_bits().hash(state);
    color.g.to_bits().hash(state);
    color.b.to_bits().hash(state);
    color.a.to_bits().hash(state);
}

impl Hash for MaterialDesc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_color(&self.base_color, state);
        self.base_color_map.hash(state);

        self.metallic.to_bits().hash(state);
        self.roughness.to_bits().hash(state);
        self.metallic_roughness_map.hash(state);
        self.reflectance.to_bits().hash(state);

        self.normal_map.hash(state);
        self.normal_scale.to_bits().hash(state);
        self.occlusion_map.hash(state);
        self.occlusion_strength.to_bits().hash(state);

        hash_color(&self.emissive, state);
        self.emissive_strength.to_bits().hash(state);
        self.emissive_map.hash(state);

        self.alpha_mode.hash(state);
        self.uv_offset.x.to_bits().hash(state);
        self.uv_offset.y.to_bits().hash(state);
        self.uv_scale.x.to_bits().hash(state);
        self.uv_scale.y.to_bits().hash(state);

        self.unlit.hash(state);
        self.double_sided.hash(state);
        self.vertex_colors.hash(state);

        self.shader.hash(state);
        self.depth_override.hash(state);
    }
}

impl MaterialDesc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.base_color = color.into();
        self
    }

    pub fn with_base_color_map(mut self, image: Handle<Image>) -> Self {
        self.base_color_map = Some(image);
        self
    }

    pub fn with_shader(mut self, shader: gpu::ShaderRef) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn with_alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }

    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }

    pub fn transparent(mut self) -> Self {
        self.alpha_mode = AlphaMode::Blend;
        self
    }

    /// The pipeline-affecting state derived from this descriptor. Only the
    /// fields that actually change how the mesh pass is recorded feed into
    /// it — the rest (metallic, roughness, the extra maps, ...) ride along
    /// on [`Material::desc`] for a future shader to pick up.
    fn pass_state(&self) -> PassState {
        let shader = self.shader.unwrap_or(crate::render::MESH_SHADER);
        let mut pass = PassState::opaque_3d(shader);

        if self.double_sided {
            pass = pass.with_cull(None);
        }

        if matches!(self.alpha_mode, AlphaMode::Blend) {
            pass = pass
                .with_blend(gpu::BlendState::ALPHA_BLENDING)
                .with_depth(Some(gpu::DepthState::TRANSPARENT));
        }

        if let Some(depth) = self.depth_override {
            pass = pass.with_depth(Some(depth));
        }

        pass
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub pass_id: u16,
    pub bind_id: u16,
    pub desc: MaterialDesc,
    pub uniforms: Vec<u8>,
}

#[derive(Default)]
pub struct MaterialRegistry {
    passes: Vec<PassState>,
    pass_ids: FastHashMap<PassState, u16>,
    binds: Vec<Vec<usize>>,
    bind_ids: FastHashMap<Vec<usize>, u16>,
    materials: SlotMap<Material>,
    interned: FastHashMap<MaterialDesc, Handle<Material>>,
    epoch: Option<u64>,
}

impl MaterialRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, assets: &Assets, desc: MaterialDesc) -> Handle<Material> {
        if let Some(&handle) = self.interned.get(&desc) {
            return handle;
        }

        let pass_id = self.intern_pass(desc.pass_state());
        let bind_id = self.intern_binds(desc.base_color_map.map(|i| assets.get_image(i).page));
        let uniforms = Self::resolve_uniforms(assets, &desc);

        let handle = self.materials.insert(Material {
            pass_id,
            bind_id,
            desc: desc.clone(),
            uniforms,
        });

        self.interned.insert(desc, handle);
        self.epoch.get_or_insert(assets.image_epoch());

        handle
    }

    /// Mutates a live material in place and recomputes everything derived
    /// from its descriptor (pipeline pass, texture binding, packed
    /// uniforms). Poking `Material::desc` through [`Self::get_mut`] directly
    /// does *not* do this — the GPU-facing `uniforms` blob is only ever
    /// resolved here or in [`Self::intern`], so a raw edit would be mutating
    /// a copy the renderer never sees.
    pub fn update<F>(&mut self, assets: &Assets, handle: Handle<Material>, edit: F)
    where
        F: FnOnce(&mut MaterialDesc),
    {
        let mut desc = self.get(handle).desc;

        // Drop the stale interning entry so a future `intern` call with the
        // pre-edit descriptor doesn't get handed this now-mutated material.
        self.interned.remove(&desc);

        edit(&mut desc);

        let pass_id = self.intern_pass(desc.pass_state());
        let bind_id = self.intern_binds(desc.base_color_map.map(|i| assets.get_image(i).page));
        let uniforms = Self::resolve_uniforms(assets, &desc);

        let material = self.get_mut(handle);

        material.pass_id = pass_id;
        material.bind_id = bind_id;
        material.uniforms = uniforms;
        material.desc = desc;

        self.interned.insert(desc, handle);
    }

    pub(crate) fn refresh(&mut self, assets: &Assets) {
        let epoch = assets.image_epoch();

        if self.epoch == Some(epoch) {
            return;
        }

        self.epoch = Some(epoch);

        let handles: Vec<_> = self
            .materials
            .iter()
            .filter(|(_, m)| m.desc.base_color_map.is_some())
            .map(|(handle, _)| handle)
            .collect();

        for handle in handles {
            let desc = self.materials[handle].desc.clone();

            let bind_id = self.intern_binds(desc.base_color_map.map(|i| assets.get_image(i).page));
            let uniforms = Self::resolve_uniforms(assets, &desc);

            let material = &mut self.materials[handle];

            material.bind_id = bind_id;
            material.uniforms = uniforms;
        }
    }

    /// Packs the subset of [`MaterialDesc`] the current mesh shader actually
    /// samples: a tint, the atlas rect of `base_color_map` (or the full
    /// [0, 1] rect when there isn't one), and `metallic`. The rest of the
    /// descriptor's PBR fields (roughness, the extra maps, emissive, ...)
    /// aren't consumed by a shader yet.
    fn resolve_uniforms(assets: &Assets, desc: &MaterialDesc) -> Vec<u8> {
        let color: [f32; 4] = desc.base_color.into();

        let uv_rect = match desc.base_color_map {
            Some(image) => {
                let image = assets.get_image(image);

                [
                    image.uv_min.x,
                    image.uv_min.y,
                    image.uv_max.x,
                    image.uv_max.y,
                ]
            }

            None => [0.0, 0.0, 1.0, 1.0],
        };

        // vec4-padded so a std140 uniform block can add the rest of the PBR
        // inputs later without reshuffling this one.
        let pbr = [desc.metallic.clamp(0.0, 1.0), 0.0, 0.0, 0.0];

        let mut uniforms = Vec::with_capacity(48);

        uniforms.extend_from_slice(utils::as_u8_slice(&[color]));
        uniforms.extend_from_slice(utils::as_u8_slice(&[uv_rect]));
        uniforms.extend_from_slice(utils::as_u8_slice(&[pbr]));

        uniforms
    }

    fn intern_pass(&mut self, pass: PassState) -> u16 {
        if let Some(&id) = self.pass_ids.get(&pass) {
            return id;
        }

        let id = self.passes.len() as u16;

        self.passes.push(pass);
        self.pass_ids.insert(pass, id);

        id
    }

    fn intern_binds(&mut self, page: Option<usize>) -> u16 {
        let pages: Vec<usize> = page.into_iter().collect();

        if let Some(&id) = self.bind_ids.get(&pages) {
            return id;
        }

        let id = self.binds.len() as u16;

        self.binds.push(pages.clone());
        self.bind_ids.insert(pages, id);

        id
    }

    pub fn get(&self, handle: Handle<Material>) -> &Material {
        self.materials.get(handle).expect("Failed to get material")
    }

    pub fn get_mut(&mut self, handle: Handle<Material>) -> &mut Material {
        self.materials
            .get_mut(handle)
            .expect("Failed to get material")
    }

    pub fn pass(&self, id: u16) -> PassState {
        self.passes[id as usize]
    }

    pub fn pages(&self, id: u16) -> &[usize] {
        &self.binds[id as usize]
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}
