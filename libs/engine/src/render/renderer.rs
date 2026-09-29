use core::mem::offset_of;
use core::ops::Range;

use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use sdl3::SdlError;
use sdl3::gpu::BufferUsage;
use sdl3::gpu::Device;
use sdl3::gpu::GpuBuffer;
use sdl3::gpu::GraphicsPipeline;
use sdl3::gpu::IndexSize;
use sdl3::gpu::LoadOp;
use sdl3::gpu::PipelineDesc;
use sdl3::gpu::Sampler;
use sdl3::gpu::SamplerDesc;
use sdl3::gpu::Shader;
use sdl3::gpu::Texture;
use sdl3::gpu::TextureDesc;
use sdl3::gpu::TextureFormat;
use sdl3::gpu::VertexAttribute;
use sdl3::gpu::VertexBuffer;
use sdl3::render::Color;
use sdl3::shadercross::CompileOptions;
use sdl3::shadercross::ShaderCross;
use sdl3::shadercross::ShaderSource;
use sdl3::window::Window;

use crate::assets::AssetServer;
use crate::mesh::Geometry;
use crate::mesh::Material;
use crate::mesh::MaterialKey;
use crate::mesh::MaterialUniform;
use crate::mesh::Mesh;
use crate::render::Camera;
use crate::render::Draw;
use crate::render::Frustum;
use crate::render::ImmediateVertex;
use crate::render::Layer;
use crate::render::LayerData;
use crate::render::LayerMap;
use crate::render::MeshVertex;
use crate::scene::SceneData;

const IMMEDIATE_VERT: &[u8] = include_bytes!("../../../../shaders/immediate.vert.spv");
const IMMEDIATE_FRAG: &[u8] = include_bytes!("../../../../shaders/immediate.frag.spv");
const MESH_VERT: &[u8] = include_bytes!("../../../../shaders/mesh.vert.spv");
const MESH_FRAG: &[u8] = include_bytes!("../../../../shaders/mesh.frag.spv");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InstanceData {
    model: math::Matrix4<f32>,
    normal: [math::Vector3<f32>; 3],
    tint: math::Vector4<f32>,
    material: u32,
}

impl InstanceData {
    fn zeroed() -> Self {
        Self {
            model: math::Matrix4::zero(),
            normal: [math::Vector3::zero(); 3],
            tint: math::Vector4::zero(),
            material: 0,
        }
    }

    fn bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub direction: math::Vector4<f32>,
    pub color: math::Vector4<f32>,
    pub ambient: math::Vector4<f32>,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            direction: math::vec4!(-0.4, -1.0, -0.6, 0.0),
            color: math::vec4!(1.0, 1.0, 1.0, 1.0),
            ambient: math::vec4!(0.25, 0.25, 0.25, 1.0),
        }
    }
}

struct Batch {
    camera: Camera,
    first_index: u32,
    indices: u32,
    vertex_offset: i32,
    retained: usize,
    opaque: Range<usize>,
    transparent: Range<usize>,
}

#[derive(Clone, Copy)]
struct PreparedMesh {
    pipeline: usize,
    geometry: Handle<Geometry>,
    instance: InstanceData,
    transparent: bool,
    distance: f32,
    center: math::Vector3<f32>,
    radius: f32,
}

const CELL_SIZE: f32 = 64.0;

type Cell = (i32, i32, i32);

fn cell_of(v: f32) -> i32 {
    let scaled = v / CELL_SIZE;
    let truncated = scaled as i32;

    if (truncated as f32) > scaled {
        truncated - 1
    } else {
        truncated
    }
}

impl PreparedMesh {
    fn bounds(&self) -> Bounds {
        Bounds::sphere(self.center, self.radius)
    }

    fn cell(&self) -> Cell {
        (
            cell_of(self.center.x),
            cell_of(self.center.y),
            cell_of(self.center.z),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bounds {
    min: math::Vector3<f32>,
    max: math::Vector3<f32>,
}

impl Bounds {
    fn sphere(center: math::Vector3<f32>, radius: f32) -> Self {
        let r = math::Vector3::splat(radius);
        Self {
            min: center - r,
            max: center + r,
        }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            min: math::Vector3::min(&self.min, &other.min),
            max: math::Vector3::max(&self.max, &other.max),
        }
    }
}

struct MeshRun {
    pipeline: usize,
    geometry: Handle<Geometry>,
    first_instance: u32,
    count: u32,
    bounds: Bounds,
}

fn push_runs<'a>(
    meshes: impl IntoIterator<Item = &'a PreparedMesh>,
    instances: &mut Vec<InstanceData>,
    runs: &mut Vec<MeshRun>,
) {
    let from = runs.len();

    for mesh in meshes {
        let first_instance = instances.len() as u32;
        instances.push(mesh.instance);

        if runs.len() > from
            && let Some(run) = runs.last_mut()
            && run.pipeline == mesh.pipeline
            && run.geometry == mesh.geometry
        {
            run.count += 1;
            run.bounds = run.bounds.merge(mesh.bounds());
            continue;
        }

        runs.push(MeshRun {
            pipeline: mesh.pipeline,
            geometry: mesh.geometry,
            first_instance,
            count: 1,
            bounds: mesh.bounds(),
        });
    }
}

struct MeshPipelines {
    vertex: Shader,
    fragment: Shader,
    pipelines: Vec<(MaterialKey, GraphicsPipeline)>,
}

impl MeshPipelines {
    fn get_or_create(
        &mut self,
        device: &Device,
        key: MaterialKey,
        color: TextureFormat,
        depth: TextureFormat,
    ) -> Result<usize, SdlError> {
        if let Some(index) = self.pipelines.iter().position(|(k, _)| *k == key) {
            return Ok(index);
        }

        let model = offset_of!(InstanceData, model) as u32;
        let normal = offset_of!(InstanceData, normal) as u32;

        let buffers = [
            VertexBuffer::of::<MeshVertex>(0),
            VertexBuffer::of_instance::<InstanceData>(1),
        ];
        let attributes = [
            VertexAttribute::float3(0, offset_of!(MeshVertex, position) as u32),
            VertexAttribute::float3(1, offset_of!(MeshVertex, normal) as u32),
            VertexAttribute::float2(2, offset_of!(MeshVertex, uv) as u32),
            VertexAttribute::float4(3, offset_of!(MeshVertex, color) as u32),
            VertexAttribute::float4(4, model).at_slot(1),
            VertexAttribute::float4(5, model + 16).at_slot(1),
            VertexAttribute::float4(6, model + 32).at_slot(1),
            VertexAttribute::float4(7, model + 48).at_slot(1),
            VertexAttribute::float3(8, normal).at_slot(1),
            VertexAttribute::float3(9, normal + 12).at_slot(1),
            VertexAttribute::float3(10, normal + 24).at_slot(1),
            VertexAttribute::float4(11, offset_of!(InstanceData, tint) as u32).at_slot(1),
            VertexAttribute::uint(12, offset_of!(InstanceData, material) as u32).at_slot(1),
        ];
        let targets = [color];

        let pipeline = GraphicsPipeline::new(
            device.share(),
            PipelineDesc::new(&self.vertex, &self.fragment)
                .with_vertex_layout(&buffers, &attributes)
                .with_targets(&targets)
                .with_depth_stencil(depth)
                .with_depth_write(!key.transparent)
                .with_blend(key.blend)
                .with_cull_mode(key.cull),
        )?;

        self.pipelines.push((key, pipeline));
        Ok(self.pipelines.len() - 1)
    }

    fn prepare(
        &mut self,
        device: &Device,
        color: TextureFormat,
        depth: TextureFormat,
        assets: &AssetServer,
        table: &MaterialTable,
        mesh: &Mesh,
    ) -> Option<PreparedMesh> {
        let material = assets.materials().resolve(mesh.material());
        let key = material.key();

        let pipeline = match self.get_or_create(device, key, color, depth) {
            Ok(index) => index,
            Err(e) => {
                traccia::error!("Failed to create mesh pipeline: {}", e);
                return None;
            }
        };

        let transform = mesh.transform();
        let (model, n) = transform.matrices();
        let scale = transform.scale.abs();
        let largest = scale.x.max(scale.y).max(scale.z);
        let tint: math::Vector4<f32> = mesh.tint().into();

        Some(PreparedMesh {
            pipeline,
            geometry: mesh.geometry(),
            instance: InstanceData {
                model,
                normal: [
                    math::vec3!(n[0][0], n[0][1], n[0][2]),
                    math::vec3!(n[1][0], n[1][1], n[1][2]),
                    math::vec3!(n[2][0], n[2][1], n[2][2]),
                ],
                tint,
                material: table.index(mesh.material()),
            },
            transparent: key.transparent,
            distance: 0.0,
            center: math::vec3!(model[3][0], model[3][1], model[3][2]),
            radius: assets.geometries().radius(mesh.geometry()) * largest,
        })
    }
}

struct MaterialTable {
    buffer: GpuBuffer<MaterialUniform>,
    entries: Vec<MaterialUniform>,
    slots: Vec<Option<(Handle<Material>, MaterialKey)>>,
    materials: u64,
    images: u64,
    layout: u64,
    upload: bool,
}

impl MaterialTable {
    fn new(device: &Device) -> Self {
        Self {
            buffer: GpuBuffer::new(
                device.share(),
                "materials",
                64,
                BufferUsage::GRAPHICS_STORAGE_READ,
            ),
            entries: Vec::new(),
            slots: Vec::new(),
            materials: u64::MAX,
            images: u64::MAX,
            layout: 0,
            upload: false,
        }
    }

    fn index(&self, handle: Handle<Material>) -> u32 {
        let index = handle.index() as usize + 1;

        match self.slots.get(index) {
            Some(Some((h, _))) if *h == handle => index as u32,
            _ => 0,
        }
    }

    fn sync(&mut self, assets: &AssetServer) {
        let registry = assets.materials();
        let images = assets.images();

        if registry.generation() == self.materials && images.generation == self.images {
            return;
        }

        self.materials = registry.generation();
        self.images = images.generation;

        let len = registry
            .iter()
            .map(|(h, _)| h.index() as usize + 2)
            .max()
            .unwrap_or(1);

        let fallback = registry.resolve(Handle::INVALID).uniform(images);
        let mut entries = nostd::alloc::vec![fallback; len];
        let mut slots = nostd::alloc::vec![None; len];

        for (handle, material) in registry.iter() {
            let index = handle.index() as usize + 1;
            entries[index] = material.uniform(images);
            slots[index] = Some((handle, material.key()));
        }

        let moved = self
            .slots
            .iter()
            .enumerate()
            .any(|(i, old)| old.is_some() && slots.get(i).copied().flatten() != *old);

        if moved {
            self.layout += 1;
        }

        self.entries = entries;
        self.slots = slots;
        self.upload = true;
    }
}

#[derive(Clone, Copy)]
struct Placement {
    group: usize,
    slot: usize,
    pipeline: usize,
    geometry: Handle<Geometry>,
    layer: Layer,
    cell: Cell,
    transparent: bool,
}

struct Group {
    layer: Layer,
    pipeline: usize,
    geometry: Handle<Geometry>,
    cell: Cell,
    bounds: Option<Bounds>,
    start: usize,
    capacity: usize,
    handles: Vec<Handle<Mesh>>,
    dirty: Option<(usize, usize)>,
}

type GroupKey = (Layer, usize, Handle<Geometry>, Cell);

#[derive(Default)]
struct PlacementMap {
    entries: Vec<Option<(Handle<Mesh>, Placement)>>,
    len: usize,
}

impl PlacementMap {
    fn get(&self, handle: &Handle<Mesh>) -> Option<&Placement> {
        match self.entries.get(handle.index() as usize)? {
            Some((h, p)) if h == handle => Some(p),
            _ => None,
        }
    }

    fn get_mut(&mut self, handle: &Handle<Mesh>) -> Option<&mut Placement> {
        match self.entries.get_mut(handle.index() as usize)? {
            Some((h, p)) if h == handle => Some(p),
            _ => None,
        }
    }

    fn insert(&mut self, handle: Handle<Mesh>, placement: Placement) {
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            self.entries.resize(index + 1, None);
        }

        if self.entries[index].replace((handle, placement)).is_none() {
            self.len += 1;
        }
    }

    fn remove(&mut self, handle: &Handle<Mesh>) -> Option<Placement> {
        let entry = self.entries.get_mut(handle.index() as usize)?;

        match entry {
            Some((h, _)) if h == handle => {
                self.len -= 1;
                entry.take().map(|(_, p)| p)
            }
            _ => None,
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.len
    }
}

impl core::ops::Index<&Handle<Mesh>> for PlacementMap {
    type Output = Placement;

    fn index(&self, handle: &Handle<Mesh>) -> &Placement {
        self.get(handle).expect("placement not found")
    }
}

#[derive(Default)]
struct MeshSlots {
    placements: PlacementMap,
    groups: Vec<Group>,
    lookup: HashMap<GroupKey, usize>,
    instances: Vec<InstanceData>,
    transparent: Vec<(Handle<Mesh>, Layer, PreparedMesh)>,
    runs: Vec<MeshRun>,
    layers: Vec<(Layer, Range<usize>)>,
    dirty_groups: Vec<usize>,
    garbage: usize,
    upload_all: bool,
    runs_dirty: bool,
}

impl MeshSlots {
    fn mark(&mut self, group: usize, first: usize, last: usize) {
        let g = &mut self.groups[group];

        match &mut g.dirty {
            Some((lo, hi)) => {
                *lo = (*lo).min(first);
                *hi = (*hi).max(last);
            }
            None => {
                g.dirty = Some((first, last));
                self.dirty_groups.push(group);
            }
        }
    }

    fn grow(&mut self, group: usize, bounds: Bounds) {
        let g = &mut self.groups[group];
        let grown = g.bounds.map_or(bounds, |b| b.merge(bounds));

        if g.bounds != Some(grown) {
            g.bounds = Some(grown);
            self.runs_dirty = true;
        }
    }

    fn clear_dirty(&mut self) {
        for group in self.dirty_groups.drain(..) {
            self.groups[group].dirty = None;
        }
    }

    fn allocate(&mut self, count: usize) -> usize {
        let start = self.instances.len();
        self.instances.resize(start + count, InstanceData::zeroed());
        start
    }

    fn create_group(&mut self, key: GroupKey, capacity: usize) -> usize {
        let start = self.allocate(capacity);

        self.groups.push(Group {
            layer: key.0,
            pipeline: key.1,
            geometry: key.2,
            cell: key.3,
            bounds: None,
            start,
            capacity,
            handles: Vec::new(),
            dirty: None,
        });

        self.lookup.insert(key, self.groups.len() - 1);
        self.groups.len() - 1
    }

    fn relocate(&mut self, group: usize) {
        let (start, capacity, len) = {
            let g = &self.groups[group];
            (g.start, g.capacity, g.handles.len())
        };

        let new_capacity = capacity * 2;
        let new_start = self.allocate(new_capacity);

        self.instances.copy_within(start..start + len, new_start);
        self.garbage += capacity;

        let g = &mut self.groups[group];
        g.start = new_start;
        g.capacity = new_capacity;
        g.dirty = None;
        self.dirty_groups.retain(|d| *d != group);

        if len > 0 {
            self.mark(group, new_start, new_start + len - 1);
        }
    }

    fn insert(&mut self, handle: Handle<Mesh>, layer: Layer, prepared: PreparedMesh) {
        if prepared.transparent {
            self.placements.insert(
                handle,
                Placement {
                    group: usize::MAX,
                    slot: self.transparent.len(),
                    pipeline: prepared.pipeline,
                    geometry: prepared.geometry,
                    layer,
                    cell: prepared.cell(),
                    transparent: true,
                },
            );
            self.transparent.push((handle, layer, prepared));
            return;
        }

        let key = (layer, prepared.pipeline, prepared.geometry, prepared.cell());

        let group = match self.lookup.get(&key) {
            Some(&group) => group,
            None => self.create_group(key, 4),
        };

        if self.groups[group].handles.len() == self.groups[group].capacity {
            self.relocate(group);
        }

        let g = &mut self.groups[group];
        let slot = g.handles.len();
        let index = g.start + slot;
        g.handles.push(handle);

        self.instances[index] = prepared.instance;
        self.mark(group, index, index);
        self.grow(group, prepared.bounds());
        self.runs_dirty = true;

        self.placements.insert(
            handle,
            Placement {
                group,
                slot,
                pipeline: prepared.pipeline,
                geometry: prepared.geometry,
                layer,
                cell: key.3,
                transparent: false,
            },
        );
    }

    fn remove(&mut self, handle: Handle<Mesh>) {
        let Some(placement) = self.placements.remove(&handle) else {
            return;
        };

        if placement.transparent {
            self.transparent.swap_remove(placement.slot);

            if let Some((moved, _, _)) = self.transparent.get(placement.slot)
                && let Some(p) = self.placements.get_mut(moved)
            {
                p.slot = placement.slot;
            }

            return;
        }

        let g = &mut self.groups[placement.group];
        let last = g.handles.len() - 1;
        let start = g.start;
        let moved = g.handles[last];

        g.handles.swap_remove(placement.slot);

        if g.handles.is_empty() {
            g.bounds = None;
        }

        if placement.slot != last {
            let (to, from) = (start + placement.slot, start + last);
            self.instances[to] = self.instances[from];
            self.mark(placement.group, to, to);

            if let Some(p) = self.placements.get_mut(&moved) {
                p.slot = placement.slot;
            }
        }

        self.runs_dirty = true;
    }

    fn update(&mut self, handle: Handle<Mesh>, layer: Layer, prepared: PreparedMesh) -> bool {
        let Some(placement) = self.placements.get(&handle).copied() else {
            return false;
        };

        if placement.pipeline != prepared.pipeline
            || placement.geometry != prepared.geometry
            || placement.layer != layer
            || placement.cell != prepared.cell()
            || placement.transparent != prepared.transparent
        {
            return false;
        }

        if placement.transparent {
            self.transparent[placement.slot].2 = prepared;
        } else {
            let index = self.groups[placement.group].start + placement.slot;

            if self.instances[index].bytes() != prepared.instance.bytes() {
                self.instances[index] = prepared.instance;
                self.mark(placement.group, index, index);
                self.grow(placement.group, prepared.bounds());
            }
        }

        true
    }

    fn rebuild(&mut self, meshes: impl Iterator<Item = (Handle<Mesh>, Layer, PreparedMesh)>) {
        *self = Self::default();

        let mut opaque = Vec::new();

        for (handle, layer, prepared) in meshes {
            if prepared.transparent {
                self.insert(handle, layer, prepared);
            } else {
                opaque.push((handle, layer, prepared));
            }
        }

        let key = |m: &(Handle<Mesh>, Layer, PreparedMesh)| -> GroupKey {
            (m.1, m.2.pipeline, m.2.geometry, m.2.cell())
        };

        opaque.sort_by_key(key);

        for chunk in opaque.chunk_by(|a, b| key(a) == key(b)) {
            let capacity = chunk.len().next_power_of_two().max(4);
            self.create_group(key(&chunk[0]), capacity);

            for (handle, layer, prepared) in chunk {
                self.insert(*handle, *layer, *prepared);
            }
        }

        self.clear_dirty();
        self.upload_all = true;
        self.runs_dirty = true;
    }

    fn should_compact(&self) -> bool {
        self.garbage > 256 && self.garbage * 2 > self.instances.len()
    }

    fn refresh_runs(&mut self) {
        if !self.runs_dirty {
            return;
        }

        self.runs.clear();
        self.layers.clear();

        let mut order: Vec<usize> = (0..self.groups.len())
            .filter(|g| !self.groups[*g].handles.is_empty())
            .collect();

        order.sort_by_key(|g| {
            let g = &self.groups[*g];
            (g.layer, g.pipeline, g.geometry, g.cell)
        });

        for chunk in order.chunk_by(|a, b| self.groups[*a].layer == self.groups[*b].layer) {
            let first_run = self.runs.len();

            for g in chunk {
                let g = &self.groups[*g];

                self.runs.push(MeshRun {
                    pipeline: g.pipeline,
                    geometry: g.geometry,
                    first_instance: g.start as u32,
                    count: g.handles.len() as u32,
                    bounds: g
                        .bounds
                        .unwrap_or(Bounds::sphere(math::Vector3::zero(), 0.0)),
                });
            }

            let layer = self.groups[chunk[0]].layer;
            self.layers.push((layer, first_run..self.runs.len()));
        }

        self.runs_dirty = false;
    }

    fn opaque_runs(&self, layer: Layer) -> Range<usize> {
        self.layers
            .iter()
            .find(|(l, _)| *l == layer)
            .map_or(0..0, |(_, range)| range.clone())
    }

    fn has_layer(&self, layer: Layer) -> bool {
        self.layers.iter().any(|(l, _)| *l == layer)
            || self.transparent.iter().any(|(_, l, _)| *l == layer)
    }
}

struct RetainedMeshes {
    store: u64,
    layout: u64,
    slots: MeshSlots,
    buffer: GpuBuffer<InstanceData>,
    used: bool,
}

impl RetainedMeshes {
    fn new(device: &Device, store: u64) -> Self {
        Self {
            store,
            layout: u64::MAX,
            slots: MeshSlots::default(),
            buffer: GpuBuffer::new(device.share(), "mesh instances", 256, BufferUsage::VERTEX),
            used: true,
        }
    }
}

pub struct Renderer {
    device: Device,
    data: LayerMap<LayerData>,
    pipeline: GraphicsPipeline,
    mesh_pipelines: MeshPipelines,
    sampler: Sampler,
    color_format: TextureFormat,
    depth_format: TextureFormat,
    depth: Option<Texture>,
    light: Light,
    vertex_buffer: GpuBuffer<ImmediateVertex>,
    index_buffer: GpuBuffer<u32>,
    instance_buffer: GpuBuffer<InstanceData>,
    vertices: Vec<ImmediateVertex>,
    indices: Vec<u32>,
    retained: Vec<RetainedMeshes>,
    material_table: MaterialTable,
    prepared: Vec<PreparedMesh>,
    instances: Vec<InstanceData>,
    runs: Vec<MeshRun>,
    batches: Vec<Batch>,
}

impl Renderer {
    pub fn new(device: Device, shadercross: &ShaderCross, window: &Window) -> Self {
        let data = LayerMap::new(
            LayerData::default(),
            LayerData::default(),
            LayerData::default(),
        );

        let color_format = device.swapchain_format(window);
        let depth_format = device.depth_format();

        let pipeline = Self::immediate_pipeline(&device, shadercross, color_format, depth_format)
            .expect("Failed to create immediate pipeline");
        let mesh_pipelines =
            Self::mesh_shaders(&device, shadercross).expect("Failed to create mesh shaders");
        let sampler = Sampler::new(device.share(), SamplerDesc::nearest());

        let vertex_buffer = GpuBuffer::new(
            device.share(),
            "immediate vertices",
            4096,
            BufferUsage::VERTEX,
        );
        let index_buffer = GpuBuffer::new(
            device.share(),
            "immediate indices",
            6144,
            BufferUsage::INDEX,
        );
        let instance_buffer =
            GpuBuffer::new(device.share(), "mesh instances", 256, BufferUsage::VERTEX);

        let material_table = MaterialTable::new(&device);

        Self {
            device,
            data,
            pipeline,
            mesh_pipelines,
            sampler,
            color_format,
            depth_format,
            depth: None,
            light: Light::default(),
            vertex_buffer,
            index_buffer,
            instance_buffer,
            vertices: Vec::new(),
            indices: Vec::new(),
            retained: Vec::new(),
            material_table,
            prepared: Vec::new(),
            instances: Vec::new(),
            runs: Vec::new(),
            batches: Vec::new(),
        }
    }

    fn immediate_pipeline(
        device: &Device,
        shadercross: &ShaderCross,
        color_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Result<GraphicsPipeline, SdlError> {
        let vertex = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(IMMEDIATE_VERT),
            &CompileOptions::vertex().with_name("immediate.vert"),
        )?;
        let fragment = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(IMMEDIATE_FRAG),
            &CompileOptions::fragment().with_name("immediate.frag"),
        )?;

        let buffers = [VertexBuffer::of::<ImmediateVertex>(0)];
        let attributes = [
            VertexAttribute::float3(0, offset_of!(ImmediateVertex, position) as u32),
            VertexAttribute::float4(1, offset_of!(ImmediateVertex, color) as u32),
            VertexAttribute::float2(2, offset_of!(ImmediateVertex, uv) as u32),
            VertexAttribute::float(3, offset_of!(ImmediateVertex, page) as u32),
        ];
        let targets = [color_format];

        GraphicsPipeline::new(
            device.share(),
            PipelineDesc::new(&vertex, &fragment)
                .with_vertex_layout(&buffers, &attributes)
                .with_targets(&targets)
                .with_depth_stencil(depth_format)
                .with_depth_test(false)
                .with_depth_write(false),
        )
    }

    fn mesh_shaders(device: &Device, shadercross: &ShaderCross) -> Result<MeshPipelines, SdlError> {
        let vertex = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(MESH_VERT),
            &CompileOptions::vertex().with_name("mesh.vert"),
        )?;
        let fragment = shadercross.create_shader(
            device.share(),
            ShaderSource::Spirv(MESH_FRAG),
            &CompileOptions::fragment().with_name("mesh.frag"),
        )?;

        Ok(MeshPipelines {
            vertex,
            fragment,
            pipelines: Vec::new(),
        })
    }

    pub fn light(&self) -> &Light {
        &self.light
    }

    pub fn light_mut(&mut self) -> &mut Light {
        &mut self.light
    }

    pub fn draw_handle<'a>(&'a mut self, assets: &'a AssetServer) -> Draw<'a> {
        for layer in self.data.values_mut() {
            layer.vertices.clear();
            layer.indices.clear();
        }

        Draw::new(&mut self.data, assets)
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.prepared.clear();
        self.instances.clear();
        self.runs.clear();
        self.batches.clear();

        self.retained.retain(|r| r.used);

        for retained in &mut self.retained {
            retained.used = false;
        }
    }

    fn sync_retained(&mut self, scene: &mut SceneData, assets: &AssetServer) -> usize {
        let store_id = scene.meshes().id();

        let index = match self.retained.iter().position(|r| r.store == store_id) {
            Some(index) => index,
            None => {
                self.retained
                    .push(RetainedMeshes::new(&self.device, store_id));
                self.retained.len() - 1
            }
        };

        #[rustfmt::skip]
        let Self { device, mesh_pipelines, color_format, depth_format, retained, material_table, .. } = self;

        let retained = &mut retained[index];
        retained.used = true;

        let changes = scene.meshes_mut().take_changes();
        let store = scene.meshes();
        let table = &*material_table;

        let mut prepare = |mesh: &Mesh| {
            mesh_pipelines.prepare(device, *color_format, *depth_format, assets, table, mesh)
        };

        let slots = &mut retained.slots;

        if changes.reset || retained.layout != table.layout {
            retained.layout = table.layout;
            slots.rebuild(
                store
                    .iter()
                    .filter_map(|(h, m)| prepare(m).map(|p| (h, m.layer(), p))),
            );
        } else {
            for handle in changes.despawned {
                slots.remove(handle);
            }

            for handle in changes.spawned {
                if let Some(mesh) = store.get(handle)
                    && let Some(prepared) = prepare(mesh)
                {
                    slots.insert(handle, mesh.layer(), prepared);
                }
            }

            for handle in changes.dirty {
                let Some(mesh) = store.get(handle) else {
                    continue;
                };

                match prepare(mesh) {
                    Some(prepared) => {
                        if !slots.update(handle, mesh.layer(), prepared) {
                            slots.remove(handle);
                            slots.insert(handle, mesh.layer(), prepared);
                        }
                    }
                    None => slots.remove(handle),
                }
            }

            if slots.should_compact() {
                slots.rebuild(
                    store
                        .iter()
                        .filter_map(|(h, m)| prepare(m).map(|p| (h, m.layer(), p))),
                );
            }
        }

        slots.refresh_runs();

        index
    }

    pub fn commit(&mut self, scene: &mut SceneData, assets: &AssetServer) {
        self.material_table.sync(assets);
        let retained = self.sync_retained(scene, assets);

        let mut order = Vec::with_capacity(self.data.len() + 3);
        order.push(Layer::WORLD);
        order.extend(self.data.order().iter().copied());

        for (layer, _) in &self.retained[retained].slots.layers {
            if !order.contains(layer) && *layer != Layer::UI && *layer != Layer::DEBUG {
                order.push(*layer);
            }
        }

        for (_, layer, _) in &self.retained[retained].slots.transparent {
            if !order.contains(layer) && *layer != Layer::UI && *layer != Layer::DEBUG {
                order.push(*layer);
            }
        }

        order.extend([Layer::UI, Layer::DEBUG]);

        for layer in order {
            let immediate = self.data.contains(layer) && !self.data[layer].is_empty();
            let meshes = &self.retained[retained].slots;

            if !immediate && !meshes.has_layer(layer) {
                continue;
            }

            let camera = *scene.camera(layer);
            let eye = camera.position();
            let start = self.prepared.len();

            for (_, l, prepared) in &meshes.transparent {
                if *l != layer {
                    continue;
                }

                let model = prepared.instance.model;
                let position = math::vec3!(model[3][0], model[3][1], model[3][2]);
                let mut prepared = *prepared;
                prepared.distance = position.distance_sq(&eye);
                self.prepared.push(prepared);
            }

            self.prepared[start..].sort_by(|a, b| b.distance.total_cmp(&a.distance));

            let first_run = self.runs.len();
            push_runs(&self.prepared[start..], &mut self.instances, &mut self.runs);

            let (indices, vertices) = if immediate {
                let data = &self.data[layer];
                (data.indices.as_slice(), data.vertices.as_slice())
            } else {
                (&[][..], &[][..])
            };

            self.batches.push(Batch {
                camera,
                first_index: self.indices.len() as u32,
                indices: indices.len() as u32,
                vertex_offset: self.vertices.len() as i32,
                retained,
                opaque: meshes.opaque_runs(layer),
                transparent: first_run..self.runs.len(),
            });

            self.vertices.extend_from_slice(vertices);
            self.indices.extend_from_slice(indices);
        }
    }

    fn ensure_depth(&mut self, size: math::Size<u32>) {
        let stale = self
            .depth
            .as_ref()
            .is_none_or(|d| d.width() != size.width || d.height() != size.height);

        if stale && size.width > 0 && size.height > 0 {
            self.depth = Some(Texture::new(
                self.device.share(),
                "depth",
                TextureDesc::depth(size.width, size.height, self.depth_format),
            ));
        }
    }

    fn upload_retained(&mut self) -> Result<(), SdlError> {
        for retained in &mut self.retained {
            if !retained.used {
                continue;
            }

            let slots = &mut retained.slots;
            let resized = retained.buffer.len() != slots.instances.len();

            if slots.upload_all || resized {
                if !slots.instances.is_empty() {
                    self.device.upload(&mut retained.buffer, &slots.instances)?;
                }

                slots.upload_all = false;
                slots.clear_dirty();
                continue;
            }

            if slots.dirty_groups.is_empty() {
                continue;
            }

            let regions: Vec<(usize, &[InstanceData])> = slots
                .dirty_groups
                .iter()
                .filter_map(|g| slots.groups[*g].dirty)
                .map(|(first, last)| (first, &slots.instances[first..=last]))
                .collect();

            self.device.upload_regions(&retained.buffer, &regions)?;
            slots.clear_dirty();
        }

        Ok(())
    }

    pub fn flush(
        &mut self,
        window: &Window,
        assets: &AssetServer,
        clear_color: Color,
    ) -> Result<(), SdlError> {
        let Some(mut frame) = self.device.begin_frame(window)? else {
            return Ok(());
        };

        let size = frame.size();

        self.ensure_depth(size);
        self.upload_retained()?;

        if self.material_table.upload {
            self.device.upload(
                &mut self.material_table.buffer,
                &self.material_table.entries,
            )?;
            self.material_table.upload = false;
        }

        if !self.indices.is_empty() {
            self.device
                .upload(&mut self.vertex_buffer, &self.vertices)?;
            self.device.upload(&mut self.index_buffer, &self.indices)?;
        }

        if !self.instances.is_empty() {
            self.device
                .upload(&mut self.instance_buffer, &self.instances)?;
        }

        let Some(depth) = self.depth.as_ref() else {
            frame.render_pass(LoadOp::Clear(clear_color))?;
            return frame.submit();
        };

        let atlas = assets.atlas();
        let geometries = assets.geometries();
        let stride = core::mem::size_of::<InstanceData>() as u32;

        if self.batches.is_empty() {
            frame.render_pass_with_depth(LoadOp::Clear(clear_color), depth, 1.0)?;
        }

        for (i, batch) in self.batches.iter().enumerate() {
            let load = if i == 0 {
                LoadOp::Clear(clear_color)
            } else {
                LoadOp::Load
            };

            let mut pass = frame.render_pass_with_depth(load, depth, 1.0)?;
            let mut camera = batch.camera;
            camera.update(size);
            let view_projection = camera.mvp();
            let frustum = Frustum::from_matrix(&view_projection);

            let draw_runs = |pass: &mut sdl3::gpu::RenderPass<'_>,
                             runs: &[MeshRun],
                             instances: &GpuBuffer<InstanceData>| {
                let mut bound = usize::MAX;

                for run in runs {
                    if !frustum.intersects_aabb(run.bounds.min, run.bounds.max) {
                        continue;
                    }

                    let Some((_, pipeline)) = self.mesh_pipelines.pipelines.get(run.pipeline)
                    else {
                        continue;
                    };

                    let Some(gpu) = geometries.gpu(run.geometry) else {
                        continue;
                    };

                    if gpu.index_count() == 0 {
                        continue;
                    }

                    if bound != run.pipeline {
                        pass.bind_pipeline(pipeline);
                        pass.bind_vertex_storage_buffer(0, &self.material_table.buffer);
                        pass.bind_fragment_sampler(0, atlas.texture(), &self.sampler);
                        pass.push_vertex_uniform(0, &view_projection);
                        pass.push_fragment_uniform(0, &self.light);
                        bound = run.pipeline;
                    }

                    pass.bind_vertex_buffer(0, gpu.vertex_buffer(), 0);
                    pass.bind_vertex_buffer(1, instances, run.first_instance * stride);
                    pass.bind_index_buffer(gpu.index_buffer(), IndexSize::U32, 0);
                    pass.draw_indexed_instanced(gpu.index_count(), run.count, 0, 0, 0);
                }
            };

            let retained = &self.retained[batch.retained];
            draw_runs(
                &mut pass,
                &retained.slots.runs[batch.opaque.clone()],
                &retained.buffer,
            );

            if batch.indices > 0 {
                pass.bind_pipeline(&self.pipeline);
                pass.bind_vertex_buffer(0, &self.vertex_buffer, 0);
                pass.bind_index_buffer(&self.index_buffer, IndexSize::U32, 0);
                pass.bind_fragment_sampler(0, atlas.texture(), &self.sampler);
                pass.push_vertex_uniform(0, &view_projection);
                pass.draw_indexed_instanced(
                    batch.indices,
                    1,
                    batch.first_index,
                    batch.vertex_offset,
                    0,
                );
            }

            draw_runs(
                &mut pass,
                &self.runs[batch.transparent.clone()],
                &self.instance_buffer,
            );
        }

        drop(atlas);
        frame.submit()
    }
}

#[cfg(test)]
mod tests {
    use nostd::collections::SlotMap;

    use super::*;

    fn instance() -> InstanceData {
        InstanceData {
            model: math::Matrix4::identity(),
            normal: [math::Vector3::zero(); 3],
            tint: math::Vector4::one(),
            material: 0,
        }
    }

    fn mesh(pipeline: usize, geometry: Handle<Geometry>) -> PreparedMesh {
        PreparedMesh {
            pipeline,
            geometry,
            instance: instance(),
            transparent: false,
            distance: 0.0,
            center: math::Vector3::zero(),
            radius: 1.0,
        }
    }

    fn handles() -> (Handle<Geometry>, Handle<Geometry>) {
        let mut slots: SlotMap<()> = SlotMap::new();
        (slots.insert(()).cast(), slots.insert(()).cast())
    }

    #[test]
    fn merges_matching_neighbours() {
        let (cube, sphere) = handles();
        let meshes = [
            mesh(0, cube),
            mesh(0, cube),
            mesh(0, sphere),
            mesh(1, sphere),
            mesh(1, sphere),
        ];
        let (mut instances, mut runs) = (Vec::new(), Vec::new());

        push_runs(&meshes, &mut instances, &mut runs);

        assert_eq!(instances.len(), 5);
        let shape: Vec<_> = runs
            .iter()
            .map(|r| (r.pipeline, r.first_instance, r.count))
            .collect();
        assert_eq!(shape, [(0, 0, 2), (0, 2, 1), (1, 3, 2)]);
    }

    #[test]
    fn never_merges_across_calls() {
        let (cube, _) = handles();
        let (mut instances, mut runs) = (Vec::new(), Vec::new());

        push_runs(&[mesh(0, cube)], &mut instances, &mut runs);
        push_runs(&[mesh(0, cube)], &mut instances, &mut runs);

        assert_eq!(runs.len(), 2);
        assert_eq!(runs[1].first_instance, 1);
    }

    fn mesh_handles(count: usize) -> Vec<Handle<Mesh>> {
        let mut slots: SlotMap<()> = SlotMap::new();
        (0..count).map(|_| slots.insert(()).cast()).collect()
    }

    fn tagged(pipeline: usize, geometry: Handle<Geometry>, id: f32) -> PreparedMesh {
        let mut m = mesh(pipeline, geometry);
        m.instance.tint.x = id;
        m
    }

    fn dirty_span(slots: &MeshSlots) -> usize {
        slots
            .dirty_groups
            .iter()
            .filter_map(|g| slots.groups[*g].dirty)
            .map(|(first, last)| last - first + 1)
            .sum()
    }

    fn check(slots: &MeshSlots, live: &[(Handle<Mesh>, f32)]) {
        assert_eq!(slots.placements.len(), live.len());

        for (handle, id) in live {
            let p = slots.placements[handle];
            let g = &slots.groups[p.group];
            assert_eq!(g.handles[p.slot], *handle);
            assert!(p.slot < g.capacity);
            assert_eq!(slots.instances[g.start + p.slot].tint.x, *id);
        }

        let drawn: u32 = slots.runs.iter().map(|r| r.count).sum();
        assert_eq!(drawn as usize, live.len());

        for run in &slots.runs {
            let ids: Vec<f32> = (0..run.count)
                .map(|i| slots.instances[(run.first_instance + i) as usize].tint.x)
                .collect();

            for id in ids {
                assert!(live.iter().any(|(_, l)| *l == id));
            }
        }
    }

    #[test]
    fn slots_grow_and_keep_data() {
        let (cube, _) = handles();
        let meshes = mesh_handles(20);
        let mut slots = MeshSlots::default();
        let mut live = Vec::new();

        for (i, h) in meshes.iter().enumerate() {
            slots.insert(*h, Layer::WORLD, tagged(0, cube, i as f32));
            live.push((*h, i as f32));
        }

        slots.refresh_runs();
        check(&slots, &live);
        assert_eq!(slots.runs.len(), 1);
        assert!(slots.garbage > 0);
    }

    #[test]
    fn slots_remove_swaps_last_into_hole() {
        let (cube, _) = handles();
        let meshes = mesh_handles(6);
        let mut slots = MeshSlots::default();
        let mut live = Vec::new();

        for (i, h) in meshes.iter().enumerate() {
            slots.insert(*h, Layer::WORLD, tagged(0, cube, i as f32));
            live.push((*h, i as f32));
        }

        slots.clear_dirty();
        slots.remove(meshes[1]);
        live.retain(|(h, _)| *h != meshes[1]);

        slots.refresh_runs();
        check(&slots, &live);
        assert_eq!(dirty_span(&slots), 1);

        slots.remove(meshes[5]);
        slots.remove(meshes[5]);
        live.retain(|(h, _)| *h != meshes[5]);

        slots.refresh_runs();
        check(&slots, &live);
    }

    #[test]
    fn slots_update_in_place_or_move_groups() {
        let (cube, sphere) = handles();
        let meshes = mesh_handles(3);
        let mut slots = MeshSlots::default();

        for (i, h) in meshes.iter().enumerate() {
            slots.insert(*h, Layer::WORLD, tagged(0, cube, i as f32));
        }

        slots.clear_dirty();
        assert!(slots.update(meshes[0], Layer::WORLD, tagged(0, cube, 0.0)));
        assert_eq!(dirty_span(&slots), 0);

        assert!(slots.update(meshes[0], Layer::WORLD, tagged(0, cube, 10.0)));
        assert_eq!(dirty_span(&slots), 1);

        assert!(!slots.update(meshes[1], Layer::WORLD, tagged(0, sphere, 1.0)));
        slots.remove(meshes[1]);
        slots.insert(meshes[1], Layer::WORLD, tagged(0, sphere, 1.0));

        slots.refresh_runs();
        check(
            &slots,
            &[(meshes[0], 10.0), (meshes[1], 1.0), (meshes[2], 2.0)],
        );
        assert_eq!(slots.runs.len(), 2);
    }

    #[test]
    fn slots_track_transparent_separately() {
        let (cube, _) = handles();
        let meshes = mesh_handles(3);
        let mut slots = MeshSlots::default();

        for (i, h) in meshes.iter().enumerate() {
            let mut m = tagged(0, cube, i as f32);
            m.transparent = true;
            slots.insert(*h, Layer::WORLD, m);
        }

        slots.remove(meshes[0]);

        let p = slots.placements[&meshes[2]];
        assert_eq!(slots.transparent[p.slot].0, meshes[2]);
        assert_eq!(slots.transparent.len(), 2);

        slots.refresh_runs();
        assert!(slots.runs.is_empty());
        assert!(slots.has_layer(Layer::WORLD));
    }

    #[test]
    fn slots_rebuild_compacts() {
        let (cube, sphere) = handles();
        let meshes = mesh_handles(40);
        let mut slots = MeshSlots::default();

        for (i, h) in meshes.iter().enumerate() {
            let geometry = if i % 2 == 0 { cube } else { sphere };
            slots.insert(*h, Layer::WORLD, tagged(0, geometry, i as f32));
        }

        let live: Vec<_> = meshes
            .iter()
            .enumerate()
            .map(|(i, h)| (*h, i as f32))
            .collect();

        let before = slots.instances.len();

        slots.rebuild(meshes.iter().enumerate().map(|(i, h)| {
            let geometry = if i % 2 == 0 { cube } else { sphere };
            (*h, Layer::WORLD, tagged(0, geometry, i as f32))
        }));

        slots.refresh_runs();
        check(&slots, &live);
        assert!(slots.instances.len() < before);
        assert_eq!(slots.garbage, 0);
        assert!(slots.upload_all);
    }

    fn placed(id: f32, center: math::Vector3<f32>, geometry: Handle<Geometry>) -> PreparedMesh {
        let mut m = tagged(0, geometry, id);
        m.center = center;
        m
    }

    #[test]
    fn cells_split_groups_and_carry_bounds() {
        let (cube, _) = handles();
        let meshes = mesh_handles(3);
        let mut slots = MeshSlots::default();

        slots.insert(
            meshes[0],
            Layer::WORLD,
            placed(0.0, math::vec3!(1.0, 0.0, 1.0), cube),
        );
        slots.insert(
            meshes[1],
            Layer::WORLD,
            placed(1.0, math::vec3!(3.0, 0.0, 2.0), cube),
        );
        slots.insert(
            meshes[2],
            Layer::WORLD,
            placed(2.0, math::vec3!(500.0, 0.0, 1.0), cube),
        );
        slots.refresh_runs();

        check(
            &slots,
            &[(meshes[0], 0.0), (meshes[1], 1.0), (meshes[2], 2.0)],
        );
        assert_eq!(slots.runs.len(), 2);

        let near = slots.runs.iter().find(|r| r.count == 2).unwrap();
        assert_eq!(near.bounds.min, math::vec3!(0.0, -1.0, 0.0));
        assert_eq!(near.bounds.max, math::vec3!(4.0, 1.0, 3.0));

        let far = slots.runs.iter().find(|r| r.count == 1).unwrap();
        assert_eq!(far.bounds.min.x, 499.0);
    }

    #[test]
    fn crossing_a_cell_regroups() {
        let (cube, _) = handles();
        let meshes = mesh_handles(1);
        let mut slots = MeshSlots::default();

        slots.insert(
            meshes[0],
            Layer::WORLD,
            placed(0.0, math::vec3!(1.0, 0.0, 0.0), cube),
        );

        assert!(slots.update(
            meshes[0],
            Layer::WORLD,
            placed(0.0, math::vec3!(2.0, 0.0, 0.0), cube)
        ));
        assert!(!slots.update(
            meshes[0],
            Layer::WORLD,
            placed(0.0, math::vec3!(-2.0, 0.0, 0.0), cube)
        ));
    }

    #[test]
    fn moving_within_a_cell_grows_bounds() {
        let (cube, _) = handles();
        let meshes = mesh_handles(1);
        let mut slots = MeshSlots::default();

        slots.insert(
            meshes[0],
            Layer::WORLD,
            placed(0.0, math::vec3!(1.0, 0.0, 0.0), cube),
        );
        slots.refresh_runs();
        assert_eq!(slots.runs[0].bounds.max.x, 2.0);

        slots.update(
            meshes[0],
            Layer::WORLD,
            placed(1.0, math::vec3!(10.0, 0.0, 0.0), cube),
        );
        slots.refresh_runs();
        assert_eq!(slots.runs[0].bounds.max.x, 11.0);
    }

    #[test]
    fn negative_coordinates_floor_into_cells() {
        assert_eq!(cell_of(0.0), 0);
        assert_eq!(cell_of(63.9), 0);
        assert_eq!(cell_of(64.0), 1);
        assert_eq!(cell_of(-0.1), -1);
        assert_eq!(cell_of(-64.0), -1);
        assert_eq!(cell_of(-64.1), -2);
    }

    #[test]
    fn instance_bytes_detect_changes() {
        let a = instance();
        let mut b = instance();
        assert_eq!(a.bytes(), b.bytes());

        b.model[3][0] = 1.0;
        assert_ne!(a.bytes(), b.bytes());
    }

    #[test]
    fn instance_layout_matches_shader() {
        assert_eq!(offset_of!(InstanceData, normal), 64);
        assert_eq!(offset_of!(InstanceData, tint), 100);
        assert_eq!(offset_of!(InstanceData, material), 116);
        assert_eq!(core::mem::size_of::<InstanceData>(), 120);
    }
}
