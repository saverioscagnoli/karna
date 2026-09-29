use core::ops::Index;
use core::ops::IndexMut;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Neg;

use num_traits::Num;
use utils::impl_deref_to_generic;

use crate::Matrix3;
use crate::Matrix4;
use crate::SdlFloat;
use crate::Vector3;
use crate::point::Point4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion<T: Num + Copy>([T; 4]);

impl_deref_to_generic!(Quaternion<T> => Point4<T> where T: Num + Copy);

impl<T: Num + Copy> Quaternion<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self([x, y, z, w])
    }

    pub fn identity() -> Self {
        Self::new(T::zero(), T::zero(), T::zero(), T::one())
    }

    pub fn from_vector(v: Vector3<T>, w: T) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }
}

impl<T: Num + Copy> Default for Quaternion<T> {
    fn default() -> Self {
        Self::identity()
    }
}

impl<T: Num + Copy> Quaternion<T> {
    pub fn as_array(&self) -> [T; 4] {
        self.0
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn xyz(&self) -> Vector3<T> {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl<T: Num + Copy> Index<usize> for Quaternion<T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl<T: Num + Copy> IndexMut<usize> for Quaternion<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl<T: Num + Copy> Mul for Quaternion<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let (ax, ay, az, aw) = self.tuple();
        let (bx, by, bz, bw) = rhs.tuple();

        Self::new(
            aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz,
        )
    }
}

impl<T: Num + Copy> MulAssign for Quaternion<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: Num + Copy + Neg<Output = T>> Neg for Quaternion<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self(self.0.map(|x| -x))
    }
}

impl<T: Num + SdlFloat> Mul<Vector3<T>> for Quaternion<T> {
    type Output = Vector3<T>;

    fn mul(self, rhs: Vector3<T>) -> Vector3<T> {
        self.rotate(&rhs)
    }
}

impl<T: Num + Copy> Quaternion<T> {
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn length_sq(&self) -> T {
        self.dot(self)
    }
}

impl<T: Num + Copy + Neg<Output = T>> Quaternion<T> {
    pub fn conjugate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }
}

impl<T: Num + SdlFloat> Quaternion<T> {
    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        let half = angle / (T::ONE + T::ONE);
        let axis = axis.normalize();

        Self::from_vector(axis * half.sdl_sin(), half.sdl_cos())
    }

    pub fn from_rotation_x(radians: T) -> Self {
        Self::from_axis_angle(&Vector3::new(T::ONE, T::ZERO, T::ZERO), radians)
    }

    pub fn from_rotation_y(radians: T) -> Self {
        Self::from_axis_angle(&Vector3::new(T::ZERO, T::ONE, T::ZERO), radians)
    }

    pub fn from_rotation_z(radians: T) -> Self {
        Self::from_axis_angle(&Vector3::new(T::ZERO, T::ZERO, T::ONE), radians)
    }

    pub fn from_euler(pitch: T, yaw: T, roll: T) -> Self {
        Self::from_rotation_y(yaw) * Self::from_rotation_x(pitch) * Self::from_rotation_z(roll)
    }

    pub fn from_rotation_arc(from: &Vector3<T>, to: &Vector3<T>) -> Self {
        let a = from.normalize();
        let b = to.normalize();
        let d = a.dot(&b);

        if d < -T::ONE + T::sdl_from_f32(1e-6) {
            let mut axis = Vector3::new(T::ONE, T::ZERO, T::ZERO).cross(&a);

            if axis.length_sq() < T::sdl_from_f32(1e-6) {
                axis = Vector3::new(T::ZERO, T::ONE, T::ZERO).cross(&a);
            }

            return Self::from_axis_angle(&axis, T::sdl_from_f32(core::f32::consts::PI));
        }

        Self::from_vector(a.cross(&b), T::ONE + d).normalize()
    }

    pub fn to_axis_angle(&self) -> (Vector3<T>, T) {
        let q = self.normalize();
        let w = q.w.sdl_clamp(-T::ONE, T::ONE);
        let angle = (T::ONE + T::ONE) * w.sdl_acos();
        let s = (T::ONE - w * w).sdl_sqrt();

        if s < T::sdl_from_f32(1e-6) {
            return (Vector3::new(T::ONE, T::ZERO, T::ZERO), angle);
        }

        (q.xyz() / s, angle)
    }

    pub fn length(&self) -> T {
        self.length_sq().sdl_sqrt()
    }

    pub fn normalize(&self) -> Self {
        let l = self.length();

        if l == T::ZERO {
            return Self::identity();
        }

        Self(self.0.map(|x| x / l))
    }

    pub fn normalize_mut(&mut self) {
        *self = self.normalize();
    }

    pub fn inverse(&self) -> Self {
        let l = self.length_sq();

        if l == T::ZERO {
            return Self::identity();
        }

        Self(self.conjugate().0.map(|x| x / l))
    }

    pub fn rotate(&self, v: &Vector3<T>) -> Vector3<T> {
        let u = self.xyz();
        let two = T::ONE + T::ONE;
        let t = u.cross(v) * two;

        *v + t * self.w + u.cross(&t)
    }

    pub fn nlerp(&self, other: &Self, t: T) -> Self {
        let b = if self.dot(other) < T::ZERO {
            -*other
        } else {
            *other
        };

        Self(core::array::from_fn(|i| self[i] + t * (b[i] - self[i]))).normalize()
    }

    pub fn slerp(&self, other: &Self, t: T) -> Self {
        let mut b = *other;
        let mut d = self.dot(other);

        if d < T::ZERO {
            b = -b;
            d = -d;
        }

        if d > T::sdl_from_f32(0.9995) {
            return self.nlerp(&b, t);
        }

        let theta = d.sdl_acos();
        let sin_theta = theta.sdl_sin();
        let wa = ((T::ONE - t) * theta).sdl_sin() / sin_theta;
        let wb = (t * theta).sdl_sin() / sin_theta;

        Self(core::array::from_fn(|i| wa * self[i] + wb * b[i]))
    }

    pub fn to_matrix3(&self) -> Matrix3<T> {
        let (x, y, z, w) = self.tuple();
        let two = T::ONE + T::ONE;

        Matrix3::new(
            T::ONE - two * (y * y + z * z),
            two * (x * y + w * z),
            two * (x * z - w * y),
            two * (x * y - w * z),
            T::ONE - two * (x * x + z * z),
            two * (y * z + w * x),
            two * (x * z + w * y),
            two * (y * z - w * x),
            T::ONE - two * (x * x + y * y),
        )
    }

    pub fn to_matrix4(&self) -> Matrix4<T> {
        let r = self.to_matrix3();
        let mut m = Matrix4::identity();

        for col in 0..3 {
            for row in 0..3 {
                m[col][row] = r[col][row];
            }
        }

        m
    }
}

impl<T: Num + SdlFloat> From<Quaternion<T>> for Matrix3<T> {
    fn from(q: Quaternion<T>) -> Self {
        q.to_matrix3()
    }
}

impl<T: Num + SdlFloat> From<Quaternion<T>> for Matrix4<T> {
    fn from(q: Quaternion<T>) -> Self {
        q.to_matrix4()
    }
}

impl<T: Num + Copy> From<[T; 4]> for Quaternion<T> {
    fn from(arr: [T; 4]) -> Self {
        Self(arr)
    }
}

impl<T: Num + Copy> From<Quaternion<T>> for [T; 4] {
    fn from(q: Quaternion<T>) -> Self {
        q.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::f32::consts::FRAC_PI_2;
    use core::f32::consts::PI;

    const EPS: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        let d = if a > b { a - b } else { b - a };
        d < EPS
    }

    fn approx_vec(a: Vector3<f32>, b: Vector3<f32>) -> bool {
        (0..3).all(|i| approx(a[i], b[i]))
    }

    fn approx_quat(a: Quaternion<f32>, b: Quaternion<f32>) -> bool {
        (0..4).all(|i| approx(a[i], b[i]))
    }

    fn approx_mat3(a: Matrix3<f32>, b: Matrix3<f32>) -> bool {
        (0..3).all(|c| (0..3).all(|r| approx(a[c][r], b[c][r])))
    }

    #[test]
    fn default_is_identity() {
        assert_eq!(
            Quaternion::<f32>::default().as_array(),
            [0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn field_access() {
        let q = Quaternion::new(1, 2, 3, 4);
        assert_eq!((q.x, q.y, q.z, q.w), (1, 2, 3, 4));
        assert_eq!(q.xyz().as_array(), [1, 2, 3]);
    }

    #[test]
    fn identity_rotates_nothing() {
        let v = Vector3::new(1.0f32, 2.0, 3.0);
        assert!(approx_vec(Quaternion::identity().rotate(&v), v));
    }

    #[test]
    fn rotate_z_quarter_turn() {
        let q = Quaternion::from_rotation_z(FRAC_PI_2);
        let v = q * Vector3::new(1.0f32, 0.0, 0.0);
        assert!(approx_vec(v, Vector3::new(0.0, 1.0, 0.0)));
    }

    #[test]
    fn rotate_x_quarter_turn() {
        let q = Quaternion::from_rotation_x(FRAC_PI_2);
        let v = q * Vector3::new(0.0f32, 1.0, 0.0);
        assert!(approx_vec(v, Vector3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn rotate_y_quarter_turn() {
        let q = Quaternion::from_rotation_y(FRAC_PI_2);
        let v = q * Vector3::new(0.0f32, 0.0, 1.0);
        assert!(approx_vec(v, Vector3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn from_axis_angle_normalizes_axis() {
        let a = Quaternion::from_axis_angle(&Vector3::new(0.0f32, 0.0, 5.0), 1.0);
        let b = Quaternion::from_rotation_z(1.0);
        assert!(approx_quat(a, b));
    }

    #[test]
    fn mul_composes_rotations() {
        let a = Quaternion::from_rotation_z(0.4f32);
        let b = Quaternion::from_rotation_x(1.1f32);
        let v = Vector3::new(0.3f32, -1.2, 2.0);
        assert!(approx_vec((a * b) * v, a * (b * v)));
    }

    #[test]
    fn mul_assign() {
        let a = Quaternion::from_rotation_y(0.7f32);
        let b = Quaternion::from_rotation_z(-0.2f32);
        let mut c = a;
        c *= b;
        assert!(approx_quat(c, a * b));
    }

    #[test]
    fn inverse_undoes_rotation() {
        let q = Quaternion::from_axis_angle(&Vector3::new(1.0f32, 2.0, 3.0), 0.9);
        let v = Vector3::new(4.0f32, -5.0, 6.0);
        assert!(approx_vec(q.inverse() * (q * v), v));
        assert!(approx_quat(q * q.inverse(), Quaternion::identity()));
    }

    #[test]
    fn conjugate_of_unit_is_inverse() {
        let q = Quaternion::from_rotation_x(1.3f32);
        assert!(approx_quat(q.conjugate(), q.inverse()));
    }

    #[test]
    fn normalize() {
        let q = Quaternion::new(1.0f32, 2.0, 3.0, 4.0).normalize();
        assert!(approx(q.length(), 1.0));
    }

    #[test]
    fn normalize_zero_is_identity() {
        let q = Quaternion::new(0.0f32, 0.0, 0.0, 0.0).normalize();
        assert_eq!(q, Quaternion::identity());
    }

    #[test]
    fn to_matrix3_matches_axis_angle() {
        let axis = Vector3::new(1.0f32, -2.0, 0.5).normalize();
        let q = Quaternion::from_axis_angle(&axis, 0.8);
        assert!(approx_mat3(
            q.to_matrix3(),
            Matrix3::from_axis_angle(&axis, 0.8)
        ));
    }

    #[test]
    fn to_matrix4_matches_rotate() {
        let q = Quaternion::from_euler(0.3f32, -1.1, 0.6);
        let v = Vector3::new(1.0f32, 2.0, 3.0);
        let m = q.to_matrix4();
        let r = m.mul_vec(&crate::Vector4::new(v.x, v.y, v.z, 1.0));
        assert!(approx_vec(r.xyz(), q * v));
        assert!(approx(r.w, 1.0));
    }

    #[test]
    fn matrix_from_quaternion() {
        let q = Quaternion::from_rotation_z(0.5f32);
        let m: Matrix4<f32> = q.into();
        let r = Matrix4::from_rotation_z(0.5f32);
        assert!((0..4).all(|c| (0..4).all(|i| approx(m[c][i], r[c][i]))));
    }

    #[test]
    fn from_euler_order() {
        let q = Quaternion::from_euler(0.2f32, 0.5, -0.3);
        let e = Quaternion::from_rotation_y(0.5)
            * Quaternion::from_rotation_x(0.2)
            * Quaternion::from_rotation_z(-0.3);
        assert!(approx_quat(q, e));
    }

    #[test]
    fn to_axis_angle_round_trip() {
        let axis = Vector3::new(0.0f32, 3.0, 4.0).normalize();
        let (a, angle) = Quaternion::from_axis_angle(&axis, 1.2).to_axis_angle();
        assert!(approx_vec(a, axis));
        assert!(approx(angle, 1.2));
    }

    #[test]
    fn to_axis_angle_identity() {
        let (_, angle) = Quaternion::<f32>::identity().to_axis_angle();
        assert!(approx(angle, 0.0));
    }

    #[test]
    fn from_rotation_arc() {
        let from = Vector3::new(1.0f32, 0.0, 0.0);
        let to = Vector3::new(0.0f32, 3.0, 4.0);
        let q = Quaternion::from_rotation_arc(&from, &to);
        assert!(approx_vec(q * from, to.normalize()));
    }

    #[test]
    fn from_rotation_arc_opposite() {
        let from = Vector3::new(0.0f32, 1.0, 0.0);
        let to = Vector3::new(0.0f32, -1.0, 0.0);
        let q = Quaternion::from_rotation_arc(&from, &to);
        assert!(approx_vec(q * from, to));
    }

    #[test]
    fn slerp_endpoints() {
        let a = Quaternion::from_rotation_y(0.1f32);
        let b = Quaternion::from_rotation_y(2.0f32);
        assert!(approx_quat(a.slerp(&b, 0.0), a));
        assert!(approx_quat(a.slerp(&b, 1.0), b));
    }

    #[test]
    fn slerp_midpoint() {
        let a = Quaternion::identity();
        let b = Quaternion::from_rotation_z(PI / 2.0);
        assert!(approx_quat(
            a.slerp(&b, 0.5),
            Quaternion::from_rotation_z(PI / 4.0)
        ));
    }

    #[test]
    fn slerp_takes_short_path() {
        let a = Quaternion::from_rotation_z(0.2f32);
        let b = -Quaternion::from_rotation_z(0.6f32);
        let v = Vector3::new(1.0f32, 0.0, 0.0);
        let r = a.slerp(&b, 0.5) * v;
        assert!(approx_vec(r, Quaternion::from_rotation_z(0.4f32) * v));
    }

    #[test]
    fn nlerp_is_unit() {
        let a = Quaternion::from_rotation_x(0.3f32);
        let b = Quaternion::from_rotation_y(1.4f32);
        assert!(approx(a.nlerp(&b, 0.37).length(), 1.0));
    }

    #[test]
    fn array_round_trip() {
        let q = Quaternion::new(1, 2, 3, 4);
        let a: [i32; 4] = q.into();
        assert_eq!(Quaternion::from(a), q);
    }

    #[test]
    fn repr_matches_array() {
        assert_eq!(
            core::mem::size_of::<Quaternion<f32>>(),
            core::mem::size_of::<[f32; 4]>()
        );
    }
}
