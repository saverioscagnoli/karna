#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: math::Vector3<f32>,
    pub rotation: math::Quaternion<f32>,
    pub scale: math::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: math::Vector3::default(),
            rotation: math::Quaternion::default(),
            scale: math::vec3!(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn matrices(&self) -> (math::Matrix4<f32>, math::Matrix3<f32>) {
        let r = self.rotation.to_matrix3();
        let (s, p) = (self.scale, self.position);
        let inv = |v: f32| if v == 0.0 { 0.0 } else { 1.0 / v };
        let n = math::vec3!(inv(s.x), inv(s.y), inv(s.z));

        let model = math::Matrix4::from_cols([
            [r[0][0] * s.x, r[0][1] * s.x, r[0][2] * s.x, 0.0],
            [r[1][0] * s.y, r[1][1] * s.y, r[1][2] * s.y, 0.0],
            [r[2][0] * s.z, r[2][1] * s.z, r[2][2] * s.z, 0.0],
            [p.x, p.y, p.z, 1.0],
        ]);

        let normal = math::Matrix3::from_cols([
            [r[0][0] * n.x, r[0][1] * n.x, r[0][2] * n.x],
            [r[1][0] * n.y, r[1][1] * n.y, r[1][2] * n.y],
            [r[2][0] * n.z, r[2][1] * n.z, r[2][2] * n.z],
        ]);

        (model, normal)
    }

    pub fn matrix(&self) -> math::Matrix4<f32> {
        self.matrices().0
    }

    pub fn normal_matrix(&self) -> math::Matrix4<f32> {
        let n = self.matrices().1;

        math::Matrix4::from_cols([
            [n[0][0], n[0][1], n[0][2], 0.0],
            [n[1][0], n[1][1], n[1][2], 0.0],
            [n[2][0], n[2][1], n[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: math::Matrix4<f32>, b: math::Matrix4<f32>) -> bool {
        (0..4).all(|c| (0..4).all(|r| (a[c][r] - b[c][r]).abs() < 1e-5))
    }

    #[test]
    fn matrices_match_composed_trs() {
        let t = Transform {
            position: math::vec3!(1.0, -2.0, 3.5),
            rotation: math::Quaternion::from_euler(0.3, -1.1, 0.7),
            scale: math::vec3!(2.0, 0.5, 1.5),
        };

        let composed = math::Matrix4::from_translation(t.position)
            .matmul(&t.rotation.to_matrix4())
            .matmul(&math::Matrix4::from_scale(t.scale));

        let inverse_scale = math::vec3!(0.5, 2.0, 1.0 / 1.5);
        let normal = t
            .rotation
            .to_matrix4()
            .matmul(&math::Matrix4::from_scale(inverse_scale));

        assert!(approx(t.matrix(), composed));
        assert!(approx(t.normal_matrix(), normal));
    }
}
