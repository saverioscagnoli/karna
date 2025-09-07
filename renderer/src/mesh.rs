use crate::fundamentals::{Descriptor, Vertex};
use math::{Mat4, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshInstance {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub color: Vec4,
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
    pub fn to_transform_matrix(&self) -> Mat4 {
        // Create scale matrix
        let scale_matrix = Mat4::scale(self.scale);

        // Create rotation matrix (assuming rotation is in radians)
        let rotation_matrix = Mat4::rotation_x(self.rotation.x)
            * Mat4::rotation_y(self.rotation.y)
            * Mat4::rotation_z(self.rotation.z);

        // Create translation matrix
        let translation_matrix = Mat4::translation(self.translation);

        // Combine: T * R * S
        translation_matrix * rotation_matrix * scale_matrix
    }
}

pub trait Mesh {
    /// This function returns a unique identifier for the mesh type.
    /// This is used to differentiate between different mesh types in the renderer.
    /// It is recommended to use a hash of the mesh type name or a similar method to
    /// ensure uniqueness.
    ///
    /// The id will be checked against other meshes the renderer is rendering, so if they are in cache,
    /// instance it.
    fn id() -> u64;

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
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub instances: Vec<MeshInstance>,
    pub instance_buffer: Option<wgpu::Buffer>,
}
