use core::f32::consts::PI;
use core::f32::consts::TAU;

use math::SdlFloat;
use math::Vector2;
use math::Vector3;
use math::vec2;
use math::vec3;
use nostd::alloc::vec::Vec;

use crate::render::MeshVertex;

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Geometry {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn new(vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn radius(&self) -> f32 {
        self.vertices
            .iter()
            .map(|v| v.position.length_sq())
            .fold(0.0, f32::max)
            .sdl_sqrt()
    }

    pub fn quad(width: f32, height: f32) -> Self {
        let mut data = Self::empty();

        data.push_face(
            Vector3::zero(),
            vec3!(width * 0.5, 0.0, 0.0),
            vec3!(0.0, height * 0.5, 0.0),
            vec3!(0.0, 0.0, 1.0),
        );

        data
    }

    pub fn plane(width: f32, depth: f32) -> Self {
        let mut data = Self::empty();

        data.push_face(
            Vector3::zero(),
            vec3!(width * 0.5, 0.0, 0.0),
            vec3!(0.0, 0.0, -depth * 0.5),
            vec3!(0.0, 1.0, 0.0),
        );

        data
    }

    pub fn cube(size: f32) -> Self {
        Self::cuboid(Vector3::splat(size))
    }

    pub fn cuboid(size: Vector3<f32>) -> Self {
        let h = size * 0.5;
        let mut data = Self::empty();

        let faces = [
            (
                vec3!(1.0, 0.0, 0.0),
                vec3!(0.0, 0.0, -h.z),
                vec3!(0.0, h.y, 0.0),
                h.x,
            ),
            (
                vec3!(-1.0, 0.0, 0.0),
                vec3!(0.0, 0.0, h.z),
                vec3!(0.0, h.y, 0.0),
                h.x,
            ),
            (
                vec3!(0.0, 1.0, 0.0),
                vec3!(h.x, 0.0, 0.0),
                vec3!(0.0, 0.0, -h.z),
                h.y,
            ),
            (
                vec3!(0.0, -1.0, 0.0),
                vec3!(h.x, 0.0, 0.0),
                vec3!(0.0, 0.0, h.z),
                h.y,
            ),
            (
                vec3!(0.0, 0.0, 1.0),
                vec3!(h.x, 0.0, 0.0),
                vec3!(0.0, h.y, 0.0),
                h.z,
            ),
            (
                vec3!(0.0, 0.0, -1.0),
                vec3!(-h.x, 0.0, 0.0),
                vec3!(0.0, h.y, 0.0),
                h.z,
            ),
        ];

        for (normal, u, v, distance) in faces {
            data.push_face(normal * distance, u, v, normal);
        }

        data
    }

    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let segments = segments.max(3);
        let rings = rings.max(2);
        let mut data = Self::empty();

        for i in 0..=rings {
            let v = i as f32 / rings as f32;
            let phi = v * PI;

            for j in 0..=segments {
                let u = j as f32 / segments as f32;
                let theta = u * TAU;
                let normal = vec3!(
                    phi.sdl_sin() * theta.sdl_cos(),
                    phi.sdl_cos(),
                    -phi.sdl_sin() * theta.sdl_sin(),
                );

                data.vertices
                    .push(MeshVertex::new(normal * radius, normal, vec2!(u, v)));
            }
        }

        let stride = segments + 1;

        for i in 0..rings {
            for j in 0..segments {
                let a = i * stride + j;
                let b = a + stride;

                if i != 0 {
                    data.indices.extend_from_slice(&[a, b, a + 1]);
                }

                if i != rings - 1 {
                    data.indices.extend_from_slice(&[a + 1, b, b + 1]);
                }
            }
        }

        data
    }

    pub fn cylinder(radius: f32, height: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let half = height * 0.5;
        let mut data = Self::empty();

        for j in 0..=segments {
            let u = j as f32 / segments as f32;
            let (sin, cos) = ((u * TAU).sdl_sin(), (u * TAU).sdl_cos());
            let normal = vec3!(cos, 0.0, -sin);
            let rim = normal * radius;

            data.vertices.push(MeshVertex::new(
                rim + vec3!(0.0, half, 0.0),
                normal,
                vec2!(u, 0.0),
            ));
            data.vertices.push(MeshVertex::new(
                rim - vec3!(0.0, half, 0.0),
                normal,
                vec2!(u, 1.0),
            ));
        }

        for j in 0..segments {
            let a = j * 2;
            let b = a + 1;

            data.indices
                .extend_from_slice(&[a, b, a + 2, a + 2, b, b + 2]);
        }

        data.push_cap(radius, half, segments, 1.0);
        data.push_cap(radius, -half, segments, -1.0);

        data
    }

    fn push_face(
        &mut self,
        center: Vector3<f32>,
        u: Vector3<f32>,
        v: Vector3<f32>,
        normal: Vector3<f32>,
    ) {
        let base = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            MeshVertex::new(center - u - v, normal, vec2!(0.0, 1.0)),
            MeshVertex::new(center + u - v, normal, vec2!(1.0, 1.0)),
            MeshVertex::new(center + u + v, normal, vec2!(1.0, 0.0)),
            MeshVertex::new(center - u + v, normal, vec2!(0.0, 0.0)),
        ]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn push_cap(&mut self, radius: f32, y: f32, segments: u32, side: f32) {
        let normal = vec3!(0.0, side, 0.0);
        let center = self.vertices.len() as u32;

        self.vertices
            .push(MeshVertex::new(vec3!(0.0, y, 0.0), normal, vec2!(0.5, 0.5)));

        for j in 0..=segments {
            let theta = j as f32 / segments as f32 * TAU;
            let (sin, cos) = (theta.sdl_sin(), theta.sdl_cos());
            let uv: Vector2<f32> = vec2!(0.5 + cos * 0.5, 0.5 - side * sin * 0.5);

            self.vertices.push(MeshVertex::new(
                vec3!(cos * radius, y, -sin * radius),
                normal,
                uv,
            ));
        }

        for j in 0..segments {
            let (p, q) = (center + 1 + j, center + 2 + j);

            if side > 0.0 {
                self.indices.extend_from_slice(&[center, p, q]);
            } else {
                self.indices.extend_from_slice(&[center, q, p]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_outward(data: &Geometry) {
        assert_eq!(data.indices.len() % 3, 0);

        for tri in data.indices.chunks(3) {
            let [a, b, c] = [0, 1, 2].map(|i| data.vertices[tri[i] as usize]);
            let face = (b.position - a.position).cross(&(c.position - a.position));
            let normal = a.normal + b.normal + c.normal;

            assert!(face.length_sq() > 0.0, "degenerate triangle {:?}", tri);
            assert!(face.dot(&normal) > 0.0, "inward winding {:?}", tri);
        }
    }

    fn assert_indices_in_range(data: &Geometry) {
        let len = data.vertices.len() as u32;
        assert!(data.indices.iter().all(|i| *i < len));
    }

    #[test]
    fn quad() {
        let data = Geometry::quad(2.0, 1.0);
        assert_eq!(data.vertices.len(), 4);
        assert_eq!(data.indices.len(), 6);
        assert_outward(&data);
    }

    #[test]
    fn plane() {
        let data = Geometry::plane(4.0, 2.0);
        assert_eq!(data.vertices.len(), 4);
        assert!(data.vertices.iter().all(|v| v.position.y == 0.0));
        assert_outward(&data);
    }

    #[test]
    fn cube() {
        let data = Geometry::cube(2.0);
        assert_eq!(data.vertices.len(), 24);
        assert_eq!(data.indices.len(), 36);
        assert_indices_in_range(&data);
        assert_outward(&data);

        for v in &data.vertices {
            assert!(v.position.iter().all(|c| c.abs() == 1.0));
        }
    }

    #[test]
    fn cuboid_extents() {
        let data = Geometry::cuboid(vec3!(2.0, 4.0, 6.0));
        let max = data
            .vertices
            .iter()
            .fold(Vector3::zero(), |m, v| Vector3::max(&m, &v.position));
        assert_eq!(max, vec3!(1.0, 2.0, 3.0));
    }

    #[test]
    fn sphere() {
        let data = Geometry::sphere(2.0, 16, 8);
        assert_eq!(data.vertices.len(), 17 * 9);
        assert_eq!(data.indices.len(), (16 * 8 * 2 - 16 * 2) * 3);
        assert_indices_in_range(&data);
        assert_outward(&data);

        for v in &data.vertices {
            assert!((v.position.length() - 2.0).abs() < 1e-4);
        }
    }

    #[test]
    fn cylinder() {
        let data = Geometry::cylinder(1.0, 2.0, 12);
        assert_indices_in_range(&data);
        assert_outward(&data);
        assert!(data.vertices.iter().all(|v| v.position.y.abs() <= 1.0));
    }

    #[test]
    fn radius_covers_every_vertex() {
        assert!((Geometry::cube(2.0).radius() - 3f32.sdl_sqrt()).abs() < 1e-5);
        assert!((Geometry::sphere(1.5, 12, 8).radius() - 1.5).abs() < 1e-5);
        assert_eq!(Geometry::empty().radius(), 0.0);
    }

    #[test]
    fn minimum_segments() {
        assert_outward(&Geometry::sphere(1.0, 0, 0));
        assert_outward(&Geometry::cylinder(1.0, 1.0, 0));
    }
}
