use std::hash::Hash;
use std::hash::Hasher;

use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialDesc {
    pub pass: PassState,
    pub images: Vec<Handle<Image>>,
    pub uniforms: Vec<u8>,
}

impl Default for MaterialDesc {
    fn default() -> Self {
        Self {
            pass: PassState::opaque_3d(crate::render::MESH_SHADER),
            images: Vec::new(),
            uniforms: Vec::new(),
        }
    }
}

impl MaterialDesc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pass(mut self, pass: PassState) -> Self {
        self.pass = pass;
        self
    }

    pub fn with_shader(mut self, shader: gpu::ShaderRef) -> Self {
        self.pass.shader = shader;
        self
    }

    pub fn transparent(mut self) -> Self {
        self.pass.blend = gpu::BlendState::ALPHA_BLENDING;
        self.pass.depth = Some(gpu::DepthState::TRANSPARENT);
        self
    }

    pub fn with_topology(mut self, topology: gpu::PrimitiveTopology) -> Self {
        self.pass.topology = topology;
        self
    }

    pub fn with_cull(mut self, cull: Option<gpu::Cull>) -> Self {
        self.pass.cull = cull;
        self
    }

    pub fn without_depth(mut self) -> Self {
        self.pass.depth = None;
        self
    }
    pub fn with_image(mut self, image: Handle<Image>) -> Self {
        self.images.push(image);
        self
    }

    pub fn with_uniform<T>(mut self, value: T) -> Self
    where
        T: Copy,
    {
        self.uniforms
            .extend_from_slice(utils::as_u8_slice(&[value]));
        self
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

        let pass_id = self.intern_pass(desc.pass);
        let bind_id = self.intern_binds(assets, &desc.images);
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

    pub(crate) fn refresh(&mut self, assets: &Assets) {
        let epoch = assets.image_epoch();

        if self.epoch == Some(epoch) {
            return;
        }

        self.epoch = Some(epoch);

        let handles: Vec<_> = self
            .materials
            .iter()
            .filter(|(_, m)| !m.desc.images.is_empty())
            .map(|(handle, _)| handle)
            .collect();

        for handle in handles {
            let desc = self.materials[handle].desc.clone();

            let bind_id = self.intern_binds(assets, &desc.images);
            let uniforms = Self::resolve_uniforms(assets, &desc);

            let material = &mut self.materials[handle];

            material.bind_id = bind_id;
            material.uniforms = uniforms;
        }
    }

    fn resolve_uniforms(assets: &Assets, desc: &MaterialDesc) -> Vec<u8> {
        let mut uniforms = Vec::with_capacity(desc.uniforms.len() + desc.images.len() * 16);

        uniforms.extend_from_slice(&desc.uniforms);

        for &image in &desc.images {
            let image = assets.get_image(image);
            let rect = [
                image.uv_min.x,
                image.uv_min.y,
                image.uv_max.x,
                image.uv_max.y,
            ];

            uniforms.extend_from_slice(utils::as_u8_slice(&rect));
        }

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

    fn intern_binds(&mut self, assets: &Assets, images: &[Handle<Image>]) -> u16 {
        let mut pages = Vec::with_capacity(images.len());

        for &image in images {
            let page = assets.get_image(image).page;

            if !pages.contains(&page) {
                pages.push(page);
            }
        }

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
