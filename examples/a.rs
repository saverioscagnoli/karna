use karna::render::{Mesh, Vertex};
use math::Vec3;

struct Square;

impl Mesh for Square {
    const VERTICES: &'static [Vertex] = &[
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            color: karna::math::Vec4::new(1.0, 0.0, 0.0, 1.0),
        },
        Vertex {
            position: karna::math::Vec3::new(0.5, -0.5, 0.0),
            color: karna::math::Vec4::new(0.0, 1.0, 0.0, 1.0),
        },
        Vertex {
            position: karna::math::Vec3::new(0.5, 0.5, 0.0),
            color: karna::math::Vec4::new(0.0, 0.0, 1.0, 1.0),
        },
        Vertex {
            position: karna::math::Vec3::new(-0.5, 0.5, 0.0),
            color: karna::math::Vec4::new(1.0, 1.0, 0.0, 1.0),
        },
    ];

    const INDICES: &'static [u16] = &[0, 1, 2, 2, 3, 0];
}

struct Triangle;

impl Mesh for Triangle {
    const VERTICES: &'static [Vertex] = &[
        Vertex {
            position: karna::math::Vec3::new(0.0, 1.0, 0.0),
            color: karna::math::Vec4::new(0.0, 1.0, 0.0, 1.0),
        },
        Vertex {
            position: karna::math::Vec3::new(-1.0, -1.0, 0.0),
            color: karna::math::Vec4::new(0.0, 0.0, 1.0, 1.0),
        },
        Vertex {
            position: karna::math::Vec3::new(1.0, -1.0, 0.0),
            color: karna::math::Vec4::new(1.0, 1.0, 0.0, 1.0),
        },
    ];

    const INDICES: &'static [u16] = &[0, 1, 2];
}

fn main() {
    let square = Square;
    let triangle = Triangle;

    println!("Square vertices: {:?}", Square::vertices());
    println!("Square indices: {:?}", Square::indices());

    println!("Triangle vertices: {:?}", Triangle::vertices());
    println!("Triangle indices: {:?}", Triangle::indices());
}
