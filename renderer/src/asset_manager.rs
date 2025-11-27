use std::{ops::Index, path::PathBuf};

use math::{Vector3, Vector4};
use wgpu::wgc::pipeline::VertexStep;
use winit::dpi::Position;

use crate::{Vertex, color::Color, mesh::MeshInstanceData};

pub struct MeshManager;

impl MeshManager {
    pub fn load_mesh(path: PathBuf) -> Option<(Vec<Vertex>, Vec<u32>)> {
        let (document, buffers, _images) = gltf::import(path).unwrap();

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut vertex_offset = 0u32;

        // Iterate through all meshes in the file
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read positions (required)
                let positions = reader
                    .read_positions()
                    .ok_or("Mesh missing positions")
                    .unwrap()
                    .collect::<Vec<[f32; 3]>>();

                // Read colors (optional, default to white)
                let colors: Vec<[f32; 4]> = reader
                    .read_colors(0)
                    .map(|colors| colors.into_rgba_f32().collect())
                    .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

                // Create vertices
                let vertices: Vec<Vertex> = positions
                    .iter()
                    .zip(colors.iter())
                    .map(|(pos, color)| Vertex {
                        position: Vector3::new(pos[0], pos[1], pos[2]),
                        color: Vector4::new(color[0], color[1], color[2], color[3]),
                    })
                    .collect();

                // Read indices
                if let Some(indices_reader) = reader.read_indices() {
                    let indices: Vec<u32> = indices_reader
                        .into_u32()
                        .map(|idx| idx + vertex_offset)
                        .collect();
                    all_indices.extend(indices);
                }

                vertex_offset += vertices.len() as u32;
                all_vertices.extend(vertices);
            }
        }

        Some((all_vertices, all_indices))
    }
}

// First, the helper function (can be private)
pub fn load_gltf_data(path: &str) -> (Vec<Vertex>, Vec<u32>) {
    let (document, buffers, _) =
        gltf::import(path).expect(&format!("Failed to load GLTF file: {}", path));

    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut vertex_offset = 0u32;

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Read positions (required)
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .expect("Mesh missing positions")
                .collect();

            // Read colors (optional, default to white)
            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|colors| colors.into_rgba_f32().collect())
                .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

            // Create vertices
            let vertices: Vec<Vertex> = positions
                .iter()
                .zip(colors.iter())
                .map(|(pos, color)| Vertex {
                    position: Vector3::new(pos[0], pos[1], pos[2]),
                    color: Vector4::new(color[0], color[1], color[2], color[3]),
                })
                .collect();

            // Read indices
            if let Some(indices_reader) = reader.read_indices() {
                let indices: Vec<u32> = indices_reader
                    .into_u32()
                    .map(|idx| idx + vertex_offset)
                    .collect();
                all_indices.extend(indices);
            }

            vertex_offset += vertices.len() as u32;
            all_vertices.extend(vertices);
        }
    }
    (all_vertices, all_indices)
}

#[macro_export]
macro_rules! define_mesh_from_gltf {
    ($name:ident, $path:literal) => {
        use std::sync::LazyLock;
        use $crate::{Vertex, asset_manager::load_gltf_data, mesh::MeshInstanceData};

        #[derive(Debug, Default)]
        pub struct $name {
            pub instance_data: MeshInstanceData,
        }

        impl std::ops::Deref for $name {
            type Target = $crate::mesh::MeshInstanceData;

            fn deref(&self) -> &Self::Target {
                &self.instance_data
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.instance_data
            }
        }

        impl Mesh for $name {
            fn vertices() -> Vec<Vertex> {
                static VERTICES: LazyLock<Vec<Vertex>> = LazyLock::new(|| {
                    let (vertices, _) = load_gltf_data($path);
                    vertices
                });

                VERTICES.clone()
            }

            fn indices() -> Vec<u32> {
                static INDICES: LazyLock<Vec<u32>> = LazyLock::new(|| {
                    let (_, indices) = load_gltf_data($path);
                    indices
                });

                INDICES.clone()
            }
        }
    };
}
