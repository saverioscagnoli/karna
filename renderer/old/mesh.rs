use crate::{
    Color, Renderer,
    fundamentals::{Descriptor, Vertex},
};
use math::{Vec3, Vec4};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshInstance {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub color: Vec4,
    pub dirty: bool,
}

impl Descriptor for MeshInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec3>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec3>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl MeshInstance {
    pub fn new(translation: Vec3, rotation: Vec3, scale: Vec3, color: Vec4) -> Self {
        Self {
            translation,
            rotation,
            scale,
            color,
            dirty: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshGeometry {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MeshDrawData {
    pub geometry: MeshGeometry,
    pub instances: Vec<MeshInstance>,
    pub base_instance: u32,
    pub buffer_instance_offset: usize, // Where this mesh's instances start in the global buffer
}

impl MeshDrawData {
    pub fn new(geometry: MeshGeometry) -> Self {
        Self {
            geometry,
            instances: Vec::new(),
            base_instance: 0,
            buffer_instance_offset: 0,
        }
    }
}

pub trait Mesh: Sized {
    /// This function returns a unique identifier for the mesh type.
    /// This is used to differentiate between different mesh types in the renderer.
    /// It is recommended to use a hash of the mesh type name or a similar method to
    /// ensure uniqueness.
    ///
    /// The id will be checked against other meshes the renderer is rendering, so if they are in cache,
    /// instance it.
    fn id() -> u64 {
        static ID: OnceLock<u64> = OnceLock::new();

        *ID.get_or_init(|| {
            let type_name = std::any::type_name::<Self>();
            let mut hasher = DefaultHasher::new();
            type_name.hash(&mut hasher);
            hasher.finish()
        })
    }

    /// This function specifies the vertices that make up the mesh.
    /// The vertices must be defined in model space, centered around the origin (0, 0, 0).
    /// So, as an example, a square mesh would have vertices at (-0.5, -0.5, 0.0), (0.5, -0.5, 0.0), (0.5, 0.5, 0.0), and (-0.5, 0.5, 0.0).
    /// The actual size and position of the mesh will be determined by the instance data.
    fn vertices() -> Vec<Vertex>;

    /// This function specifies the indices that make up the mesh.
    /// The indices define how the vertices are connected to form triangles.
    /// Each group of three indices represents a triangle.
    /// For example, to create two triangles that form a square using four vertices, the indices
    /// would be [0, 1, 2, 2, 3, 0].
    fn indices() -> Vec<u16>;

    fn instance(&self) -> &MeshInstance;

    fn insance_mut(&mut self) -> &mut MeshInstance;

    fn dirty(&self) -> bool {
        self.instance().dirty
    }

    fn set_dirty(&mut self, v: bool) {
        self.insance_mut().dirty = v;
    }

    fn render(&mut self, renderer: &mut Renderer) {
        _ = renderer.upsert_mesh_instance::<Self>(self.insance_mut());
    }

    fn position(&self) -> Vec3 {
        self.instance().translation
    }

    fn set_position<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().translation = delta.into();
        self.set_dirty(true);
    }

    fn add_position<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().translation += delta.into();
        self.set_dirty(true);
    }

    fn scale(&self) -> Vec3 {
        self.instance().scale
    }

    fn set_scale<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().scale = delta.into();
        self.set_dirty(true);
    }

    fn add_scale<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().scale += delta.into();
        self.set_dirty(true);
    }

    fn rotation(&self) -> Vec3 {
        self.instance().rotation
    }

    fn set_rotation<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().rotation = delta.into();
        self.set_dirty(true);
    }

    fn add_rotation<T: Into<Vec3>>(&mut self, delta: T) {
        self.insance_mut().rotation += delta.into();
        self.set_dirty(true);
    }

    fn color(&self) -> Color {
        self.instance().color.into()
    }

    fn set_color(&mut self, color: Color) {
        self.insance_mut().color = color.into();
        self.set_dirty(true);
    }
}
