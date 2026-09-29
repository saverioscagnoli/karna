use math::Matrix4;
use math::Vector3;
use math::Vector4;

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    planes: [Vector4<f32>; 6],
}

impl Frustum {
    pub fn from_matrix(m: &Matrix4<f32>) -> Self {
        let row = |i: usize| math::vec4!(m[0][i], m[1][i], m[2][i], m[3][i]);
        let (r0, r1, r2, r3) = (row(0), row(1), row(2), row(3));

        let planes = [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|p| {
            let length = p.xyz().length();

            if length == 0.0 { p } else { p / length }
        });

        Self { planes }
    }

    pub fn intersects_aabb(&self, min: Vector3<f32>, max: Vector3<f32>) -> bool {
        self.planes.iter().all(|p| {
            let x = if p.x >= 0.0 { max.x } else { min.x };
            let y = if p.y >= 0.0 { max.y } else { min.y };
            let z = if p.z >= 0.0 { max.z } else { min.z };

            p.x * x + p.y * y + p.z * z + p.w >= 0.0
        })
    }

    pub fn intersects_sphere(&self, center: Vector3<f32>, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|p| p.x * center.x + p.y * center.y + p.z * center.z + p.w >= -radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Frustum {
        let projection = Matrix4::perspective(60f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let view = Matrix4::look_at(
            &math::vec3!(0.0, 0.0, -10.0),
            &math::vec3!(0.0, 0.0, 0.0),
            &math::vec3!(0.0, 1.0, 0.0),
        );

        Frustum::from_matrix(&projection.matmul(&view))
    }

    fn cube(center: Vector3<f32>, half: f32) -> (Vector3<f32>, Vector3<f32>) {
        (center - Vector3::splat(half), center + Vector3::splat(half))
    }

    #[test]
    fn sees_what_is_in_front() {
        let f = camera();
        let (min, max) = cube(math::vec3!(0.0, 0.0, 0.0), 1.0);
        assert!(f.intersects_aabb(min, max));
        assert!(f.intersects_sphere(math::vec3!(0.0, 0.0, 0.0), 1.0));
    }

    #[test]
    fn rejects_behind_and_beyond() {
        let f = camera();
        let (min, max) = cube(math::vec3!(0.0, 0.0, -20.0), 1.0);
        assert!(!f.intersects_aabb(min, max));

        let (min, max) = cube(math::vec3!(0.0, 0.0, 200.0), 1.0);
        assert!(!f.intersects_aabb(min, max));
    }

    #[test]
    fn rejects_off_to_the_sides() {
        let f = camera();

        for c in [
            math::vec3!(100.0, 0.0, 0.0),
            math::vec3!(-100.0, 0.0, 0.0),
            math::vec3!(0.0, 100.0, 0.0),
            math::vec3!(0.0, -100.0, 0.0),
        ] {
            let (min, max) = cube(c, 1.0);
            assert!(!f.intersects_aabb(min, max));
            assert!(!f.intersects_sphere(c, 1.0));
        }
    }

    #[test]
    fn keeps_boxes_straddling_an_edge() {
        let f = camera();
        let (min, max) = cube(math::vec3!(9.0, 0.0, 0.0), 4.0);
        assert!(f.intersects_aabb(min, max));
    }

    #[test]
    fn works_with_orthographic() {
        let m = Matrix4::orthographic(0.0, 1280.0, 720.0, 0.0, -1.0, 1.0);
        let f = Frustum::from_matrix(&m);

        assert!(f.intersects_sphere(math::vec3!(640.0, 360.0, 0.0), 10.0));
        assert!(!f.intersects_sphere(math::vec3!(-100.0, 360.0, 0.0), 10.0));
        assert!(!f.intersects_sphere(math::vec3!(640.0, 900.0, 0.0), 10.0));
    }
}
