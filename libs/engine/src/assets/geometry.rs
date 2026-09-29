use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::SlotMap;
use sdl3::SdlError;
use sdl3::gpu::BufferUsage;
use sdl3::gpu::Device;
use sdl3::gpu::GpuBuffer;
use traccia::error;

use crate::mesh::Geometry;
use crate::render::MeshVertex;

pub(crate) struct GpuGeometry {
    vertex_buffer: GpuBuffer<MeshVertex>,
    index_buffer: GpuBuffer<u32>,
    index_count: u32,
}

impl GpuGeometry {
    fn new(device: &Device, geometry: &Geometry) -> Self {
        Self {
            vertex_buffer: GpuBuffer::new(
                device.share(),
                "mesh vertices",
                geometry.vertices.len(),
                BufferUsage::VERTEX,
            ),
            index_buffer: GpuBuffer::new(
                device.share(),
                "mesh indices",
                geometry.indices.len(),
                BufferUsage::INDEX,
            ),
            index_count: 0,
        }
    }

    fn upload(&mut self, device: &Device, geometry: &Geometry) -> Result<(), SdlError> {
        device.upload(&mut self.vertex_buffer, &geometry.vertices)?;
        device.upload(&mut self.index_buffer, &geometry.indices)?;
        self.index_count = geometry.indices.len() as u32;

        Ok(())
    }

    pub(crate) fn vertex_buffer(&self) -> &GpuBuffer<MeshVertex> {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &GpuBuffer<u32> {
        &self.index_buffer
    }

    pub(crate) fn index_count(&self) -> u32 {
        self.index_count
    }
}

struct GeometryEntry {
    geometry: Geometry,
    gpu: Option<GpuGeometry>,
    radius: f32,
}

pub struct GeometryRegistry {
    device: Device,
    entries: SlotMap<GeometryEntry>,
    dirty: Vec<Handle<Geometry>>,
}

impl GeometryRegistry {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            entries: SlotMap::default(),
            dirty: Vec::new(),
        }
    }

    pub fn add(&mut self, geometry: Geometry) -> Handle<Geometry> {
        let handle = self
            .entries
            .insert(GeometryEntry {
                radius: geometry.radius(),
                geometry,
                gpu: None,
            })
            .cast();

        self.dirty.push(handle);
        handle
    }

    pub fn get(&self, handle: Handle<Geometry>) -> Option<&Geometry> {
        self.entries.get(handle.cast()).map(|e| &e.geometry)
    }

    pub fn get_mut(&mut self, handle: Handle<Geometry>) -> Option<&mut Geometry> {
        let entry = self.entries.get_mut(handle.cast())?;

        if !self.dirty.contains(&handle) {
            self.dirty.push(handle);
        }

        Some(&mut entry.geometry)
    }

    pub fn remove(&mut self, handle: Handle<Geometry>) -> Option<Geometry> {
        self.dirty.retain(|h| *h != handle);
        self.entries.remove(handle.cast()).map(|e| e.geometry)
    }

    pub(crate) fn radius(&self, handle: Handle<Geometry>) -> f32 {
        self.entries.get(handle.cast()).map_or(0.0, |e| e.radius)
    }

    pub(crate) fn gpu(&self, handle: Handle<Geometry>) -> Option<&GpuGeometry> {
        self.entries.get(handle.cast())?.gpu.as_ref()
    }

    pub(crate) fn upload_dirty(&mut self) {
        for handle in self.dirty.drain(..) {
            let Some(entry) = self.entries.get_mut(handle.cast()) else {
                continue;
            };

            entry.radius = entry.geometry.radius();

            let gpu = entry
                .gpu
                .get_or_insert_with(|| GpuGeometry::new(&self.device, &entry.geometry));

            if let Err(e) = gpu.upload(&self.device, &entry.geometry) {
                error!("Failed to upload geometry {:?}: {}", handle, e);
            }
        }
    }
}
