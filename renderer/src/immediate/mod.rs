//use gpu::Vertex;
//use math::Vector2;
//use math::Vector3;
//use math::Vector4;
//
//use crate::immediate::batcher::Batcher;
//
//pub mod batcher;
//
//pub struct ImmediateRenderer {
//    triangle_batcher: Batcher<Vertex>,
//}
//
//impl ImmediateRenderer {
//    pub fn new() -> Self {
//        Self {
//            triangle_batcher: Batcher::new(),
//        }
//    }
//
//    #[inline]
//    fn push_vertices<V: Copy>(batcher: &mut Batcher<V>, vertices: &[V], pattern: &[u32]) {
//        let base = batcher.vertex_count();
//        batcher.vertices.extend_from_slice(vertices);
//        batcher.indices.extend(pattern.iter().map(|i| base + i));
//    }
//
//    pub fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32) {
//        let white = Vector4::new(1.0, 1.0, 1.0, 1.0);
//        let uv = Vector2::new(0.0, 0.0);
//        let z = 0.0;
//
//        let v = [
//            Vertex::new(Vector3::new(x, y, z), white, uv), // top-left
//            Vertex::new(Vector3::new(x + w, y, z), white, uv), // top-right
//            Vertex::new(Vector3::new(x + w, y + h, z), white, uv), // bottom-right
//            Vertex::new(Vector3::new(x, y + h, z), white, uv), // bottom-left
//        ];
//
//        // two triangles: (0,1,2) and (0,2,3)
//        const PATTERN: [u32; 6] = [0, 1, 2, 0, 2, 3];
//
//        Self::push_vertices(&mut self.triangle_batcher, &v, &PATTERN);
//    }
//}

use gpu::Vertex;

use crate::DrawCommand;

pub fn tessellate(commands: &[DrawCommand], verts: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
    let uv = math::Vector2::zero();

    for cmd in commands {
        match *cmd {
            DrawCommand::ImmediateRect { x, y, w, h, color } => {
                push_quad(verts, indices, x, y, w, h, color, uv)
            }

            _ => {}
        }
    }
}

fn push_quad(
    verts: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: math::Vector4<f32>,
    uv: math::Vector2<f32>,
) {
    let base = verts.len() as u32;
    verts.push(Vertex {
        position: math::Vector3::new(x, y, 0.0),
        color: color,
        uv,
    });
    verts.push(Vertex {
        position: math::Vector3::new(x + w, y, 0.0),
        color: color,
        uv,
    });
    verts.push(Vertex {
        position: math::Vector3::new(x + w, y + h, 0.0),
        color: color,
        uv,
    });
    verts.push(Vertex {
        position: math::Vector3::new(x, y + h, 0.0),
        color: color,
        uv,
    });

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
