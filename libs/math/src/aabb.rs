use core::ops::Neg;

use num_traits::Num;

use crate::Size;
use crate::Vector;
use crate::Vector2;
use crate::Vector3;
use crate::size;
use crate::vec2;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb<const N: usize, T: Num + Copy> {
    pub min: Vector<N, T>,
    pub max: Vector<N, T>,
}

impl<const N: usize, T: Num + Copy + PartialOrd> Aabb<N, T> {
    pub fn from_min_max(a: Vector<N, T>, b: Vector<N, T>) -> Self {
        Self {
            min: a.min(&b),
            max: a.max(&b),
        }
    }

    #[inline]
    pub fn from_center(center: Vector<N, T>, half: Vector<N, T>) -> Self {
        Self::from_min_max(center - half, center + half)
    }

    pub fn from_points<I>(points: I) -> Option<Self>
    where
        I: IntoIterator<Item = Vector<N, T>>,
    {
        let mut iter = points.into_iter();
        let first = iter.next()?;

        Some(iter.fold(
            Self {
                min: first,
                max: first,
            },
            |acc, p| Self {
                min: acc.min.min(&p),
                max: acc.max.max(&p),
            },
        ))
    }

    #[inline]
    pub fn extent(&self) -> Vector<N, T> {
        self.max - self.min
    }

    #[inline]
    pub fn center(&self) -> Vector<N, T> {
        let two = T::one() + T::one();
        self.min + self.extent() / two
    }

    #[inline]
    pub fn translate(&mut self, by: Vector<N, T>) {
        self.min += by;
        self.max += by;
    }

    #[inline]
    pub fn contains(&self, p: Vector<N, T>) -> bool {
        (0..N).all(|i| p[i] >= self.min[i] && p[i] < self.max[i])
    }

    #[inline]
    pub fn contains_aabb(&self, other: &Self) -> bool {
        (0..N).all(|i| other.min[i] >= self.min[i] && other.max[i] <= self.max[i])
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        (0..N).all(|i| self.min[i] < other.max[i] && self.max[i] > other.min[i])
    }

    #[inline]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min = self.min.max(&other.min);
        let max = self.max.min(&other.max);

        (0..N).all(|i| min[i] < max[i]).then_some(Self { min, max })
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(&other.min),
            max: self.max.max(&other.max),
        }
    }

    #[inline]
    pub fn expanded(&self, by: T) -> Self {
        Self {
            min: self.min - by,
            max: self.max + by,
        }
    }
}

impl<const N: usize, T: Num + Copy + PartialOrd + Neg<Output = T>> Aabb<N, T> {
    pub fn penetration(&self, other: &Self) -> Option<Vector<N, T>> {
        let overlap = self.intersection(other)?.extent();
        let axis = (1..N).fold(
            0,
            |best, i| if overlap[i] < overlap[best] { i } else { best },
        );
        let (a, b) = (self.center(), other.center());

        let mut push = Vector::zero();
        push[axis] = if a[axis] < b[axis] {
            -overlap[axis]
        } else {
            overlap[axis]
        };

        Some(push)
    }
}

pub type Aabb2<T> = Aabb<2, T>;
pub type Aabb3<T> = Aabb<3, T>;

impl<T: Num + Copy + PartialOrd> Aabb<2, T> {
    #[inline]
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self::from_min_max(vec2!(x, y), vec2!(x + w, y + h))
    }

    #[inline]
    pub fn from_pos_size(pos: Vector2<T>, size: Size<T>) -> Self {
        Self::new(pos.x, pos.y, size.w(), size.h())
    }

    #[inline]
    pub fn position(&self) -> Vector2<T> {
        self.min
    }

    #[inline]
    pub fn size(&self) -> Size<T> {
        let e = self.extent();
        size!(e.x, e.y)
    }

    #[inline]
    pub fn set_size(&mut self, size: Size<T>) {
        self.max = self.min + vec2!(size.w(), size.h())
    }
}

impl<T: Num + Copy + PartialOrd> Aabb<3, T> {
    #[inline]
    pub fn new(x: T, y: T, z: T, w: T, h: T, d: T) -> Self {
        Self::from_min_max(Vector3::new(x, y, z), Vector3::new(x + w, y + h, z + d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_is_half_open() {
        let r = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Vector2::new(0.0, 0.0)));
        assert!(!r.contains(Vector2::new(10.0, 5.0)));
    }

    #[test]
    fn intersects_nested_and_cross() {
        let big = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let small = Aabb2::new(4.0, 4.0, 2.0, 2.0);
        assert!(big.intersects(&small) && small.intersects(&big));

        let wide = Aabb2::new(0.0, 4.0, 10.0, 2.0);
        let tall = Aabb2::new(4.0, 0.0, 2.0, 10.0);
        assert!(wide.intersects(&tall));
    }

    #[test]
    fn touching_does_not_intersect() {
        let a = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let b = Aabb2::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn from_min_max_normalizes() {
        let a = Aabb2::from_min_max(Vector2::new(5.0, 0.0), Vector2::new(0.0, 5.0));
        assert_eq!(a.min, Vector2::new(0.0, 0.0));
        assert_eq!(a.max, Vector2::new(5.0, 5.0));
    }

    #[test]
    fn penetration_2d_lands_on_ground() {
        let player = Aabb2::new(0.0, 8.0, 4.0, 4.0);
        let ground = Aabb2::new(-10.0, 10.0, 30.0, 10.0);
        assert_eq!(player.penetration(&ground), Some(Vector2::new(0.0, -2.0)));
    }

    #[test]
    fn separated_on_z_only_does_not_intersect() {
        let a = Aabb3::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = Aabb3::new(0.0, 0.0, 5.0, 1.0, 1.0, 1.0);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn penetration_3d_picks_shallow_axis() {
        let a = Aabb3::new(0.0, 0.0, 0.0, 2.0, 2.0, 2.0);
        let b = Aabb3::new(0.5, 0.5, 1.5, 2.0, 2.0, 2.0);
        assert_eq!(a.penetration(&b), Some(Vector3::new(0.0, 0.0, -0.5)));
    }

    #[test]
    fn from_points_bounds_everything() {
        let pts = [Vector3::new(1.0, -2.0, 0.0), Vector3::new(-1.0, 3.0, 4.0)];
        let b = Aabb3::from_points(pts).unwrap();
        assert_eq!(b.min, Vector3::new(-1.0, -2.0, 0.0));
        assert_eq!(b.max, Vector3::new(1.0, 3.0, 4.0));
    }

    fn eq<const N: usize>(a: Aabb<N, f32>, min: Vector<N, f32>, max: Vector<N, f32>) -> bool {
        a.min == min && a.max == max
    }

    #[test]
    fn default_is_zero() {
        let a = Aabb2::<f32>::default();
        assert!(eq(a, Vector2::zero(), Vector2::zero()));
    }

    #[test]
    fn new_2d() {
        let a = Aabb2::new(1.0, 2.0, 3.0, 4.0);
        assert!(eq(a, Vector2::new(1.0, 2.0), Vector2::new(4.0, 6.0)));
    }

    #[test]
    fn new_2d_negative_size_normalizes() {
        let a = Aabb2::new(10.0, 0.0, -5.0, 5.0);
        assert!(eq(a, Vector2::new(5.0, 0.0), Vector2::new(10.0, 5.0)));
    }

    #[test]
    fn new_3d() {
        let a = Aabb3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert!(eq(a, Vector3::new(1.0, 2.0, 3.0), Vector3::new(5.0, 7.0, 9.0)));
    }

    #[test]
    fn from_center() {
        let a = Aabb2::from_center(Vector2::new(5.0, 5.0), Vector2::new(2.0, 1.0));
        assert!(eq(a, Vector2::new(3.0, 4.0), Vector2::new(7.0, 6.0)));
    }

    #[test]
    fn from_points_empty_is_none() {
        let pts: [Vector2<f32>; 0] = [];
        assert!(Aabb2::from_points(pts).is_none());
    }

    #[test]
    fn from_points_single_is_degenerate() {
        let p = Vector2::new(3.0, 4.0);
        let a = Aabb2::from_points([p]).unwrap();
        assert!(eq(a, p, p));
        assert_eq!(a.extent(), Vector2::zero());
    }

    #[test]
    fn extent_and_center() {
        let a = Aabb3::new(0.0, 2.0, -4.0, 4.0, 6.0, 8.0);
        assert_eq!(a.extent(), Vector3::new(4.0, 6.0, 8.0));
        assert_eq!(a.center(), Vector3::new(2.0, 5.0, 0.0));
    }

    #[test]
    fn center_integer_rounds_down() {
        let a = Aabb2::new(0, 0, 3, 5);
        assert_eq!(a.center(), Vector2::new(1, 2));
    }

    #[test]
    fn translate() {
        let mut a = Aabb2::new(0.0, 0.0, 2.0, 2.0);
        a.translate(Vector2::new(3.0, -1.0));
        assert!(eq(a, Vector2::new(3.0, -1.0), Vector2::new(5.0, 1.0)));
    }

    #[test]
    fn contains_edges() {
        let r = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Vector2::new(9.999, 9.999)));
        assert!(!r.contains(Vector2::new(5.0, 10.0)));
        assert!(!r.contains(Vector2::new(-0.001, 5.0)));
    }

    #[test]
    fn contains_3d() {
        let r = Aabb3::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(r.contains(Vector3::new(0.5, 0.5, 0.5)));
        assert!(!r.contains(Vector3::new(0.5, 0.5, 1.5)));
    }

    #[test]
    fn contains_aabb() {
        let big = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let small = Aabb2::new(2.0, 2.0, 3.0, 3.0);
        let poking = Aabb2::new(8.0, 8.0, 3.0, 3.0);
        assert!(big.contains_aabb(&small));
        assert!(big.contains_aabb(&big));
        assert!(!small.contains_aabb(&big));
        assert!(!big.contains_aabb(&poking));
    }

    #[test]
    fn intersects_is_symmetric() {
        let a = Aabb2::new(0.0, 0.0, 4.0, 4.0);
        let b = Aabb2::new(3.0, -1.0, 4.0, 2.0);
        assert_eq!(a.intersects(&b), b.intersects(&a));
        assert!(a.intersects(&b));
    }

    #[test]
    fn separated_does_not_intersect() {
        let a = Aabb2::new(0.0, 0.0, 1.0, 1.0);
        let b = Aabb2::new(5.0, 5.0, 1.0, 1.0);
        assert!(!a.intersects(&b));
        assert!(a.intersection(&b).is_none());
        assert!(a.penetration(&b).is_none());
    }

    #[test]
    fn intersection_region() {
        let a = Aabb2::new(0.0, 0.0, 4.0, 4.0);
        let b = Aabb2::new(2.0, 1.0, 4.0, 2.0);
        let i = a.intersection(&b).unwrap();
        assert!(eq(i, Vector2::new(2.0, 1.0), Vector2::new(4.0, 3.0)));
    }

    #[test]
    fn intersection_of_nested_is_inner() {
        let big = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let small = Aabb2::new(2.0, 3.0, 1.0, 1.0);
        let i = big.intersection(&small).unwrap();
        assert!(eq(i, small.min, small.max));
    }

    #[test]
    fn union() {
        let a = Aabb2::new(0.0, 0.0, 1.0, 1.0);
        let b = Aabb2::new(5.0, -2.0, 1.0, 1.0);
        let u = a.union(&b);
        assert!(eq(u, Vector2::new(0.0, -2.0), Vector2::new(6.0, 1.0)));
        assert!(u.contains_aabb(&a) && u.contains_aabb(&b));
    }

    #[test]
    fn expanded() {
        let a = Aabb2::new(0.0, 0.0, 2.0, 2.0).expanded(1.0);
        assert!(eq(a, Vector2::new(-1.0, -1.0), Vector2::new(3.0, 3.0)));
    }

    #[test]
    fn penetration_pushes_away_on_each_side() {
        let wall = Aabb2::new(0.0, 0.0, 10.0, 10.0);
        let left = Aabb2::new(-3.0, 4.0, 4.0, 2.0);
        let right = Aabb2::new(9.0, 4.0, 4.0, 2.0);
        let below = Aabb2::new(4.0, 9.5, 2.0, 4.0);
        assert_eq!(left.penetration(&wall), Some(Vector2::new(-1.0, 0.0)));
        assert_eq!(right.penetration(&wall), Some(Vector2::new(1.0, 0.0)));
        assert_eq!(below.penetration(&wall), Some(Vector2::new(0.0, 0.5)));
    }

    #[test]
    fn penetration_resolves_overlap() {
        let mut player = Aabb2::new(1.0, 8.5, 2.0, 2.0);
        let ground = Aabb2::new(0.0, 10.0, 20.0, 5.0);
        let push = player.penetration(&ground).unwrap();
        player.translate(push);
        assert!(!player.intersects(&ground));
    }

    #[test]
    fn penetration_touching_is_none() {
        let a = Aabb2::new(0.0, 0.0, 1.0, 1.0);
        let b = Aabb2::new(1.0, 0.0, 1.0, 1.0);
        assert!(a.penetration(&b).is_none());
    }

    #[test]
    fn penetration_integers() {
        let a = Aabb2::new(0, 0, 4, 4);
        let b = Aabb2::new(3, 0, 4, 4);
        assert_eq!(a.penetration(&b), Some(Vector2::new(-1, 0)));
    }

    #[test]
    fn pos_size_round_trip() {
        let a = Aabb2::from_pos_size(Vector2::new(1.0, 2.0), Size::new(3.0, 4.0));
        assert_eq!(a.position(), Vector2::new(1.0, 2.0));
        assert_eq!(a.size(), Size::new(3.0, 4.0));
    }

    #[test]
    fn set_size_keeps_position() {
        let mut a = Aabb2::new(1.0, 2.0, 3.0, 4.0);
        a.set_size(Size::new(10.0, 1.0));
        assert_eq!(a.position(), Vector2::new(1.0, 2.0));
        assert_eq!(a.size(), Size::new(10.0, 1.0));
    }
}
