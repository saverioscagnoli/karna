use crate::color::Color;
use karna_log::KarnaError;
use karna_math::size::Size;
use karna_opengl::{
    bind_vertex_buffer,
    buffers::{IndexBuffer, VertexBuffer},
    draw_elements,
    texture::{Filtering, Texture, Wrap},
    DataType, DrawMode, Vertex,
};
use rect_packer::{DensePacker, Rect};
use std::{collections::HashMap, path::Path};

pub struct Atlas {
    /// The actual texture that will contain all the loaded textures
    /// from the user.
    pub texture: Texture,
    /// Packer that will be used to pack all the textures into the atlas.
    /// TODO: implement my own packer.
    packer: DensePacker,
    regions: HashMap<Box<str>, Rect>,

    /// A white 1x1 texture that is used to ensure that the
    /// fragment shader will draw both textures and vertex colors.
    /// This is because the fragment color by default is calculated like this:
    /// `frag_color = texture_color * vertex_color`.
    /// So if texture_color is not white when drawing a vertex color,
    /// it will be all messed up.
    pub white_1x1: Texture,

    vbo: VertexBuffer,
    ebo: IndexBuffer,
    /// The vertices that will be used to draw the textures.
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Atlas {
    const SIZE_STEP: u32 = 1024;

    pub fn new() -> Self {
        let texture = Texture::new(Self::SIZE_STEP, Self::SIZE_STEP);
        let white_1x1 = Texture::new(1, 1);

        texture.bind();
        texture.set_filtering(Filtering::Nearest, Filtering::Nearest);
        texture.set_wrap(Wrap::ClampToEdge);

        // Fill the texture with black pixels.
        texture.set_data(&[0; (Self::SIZE_STEP * Self::SIZE_STEP * 4) as usize]);

        white_1x1.bind();
        white_1x1.set_data(&[255, 255, 255, 255]);

        Texture::unbind();

        Self {
            texture,
            packer: DensePacker::new(Self::SIZE_STEP as i32, Self::SIZE_STEP as i32),
            regions: HashMap::new(),
            white_1x1,
            vbo: VertexBuffer::new(),
            ebo: IndexBuffer::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn cache_region<L: AsRef<str>>(&mut self, label: L, width: u32, height: u32) -> Rect {
        while !self.packer.can_pack(width as i32, height as i32, false) {
            let size: Size<u32> = self.packer.size().into();
            let new_size = size.width.max(size.height) + Self::SIZE_STEP;

            self.packer.resize(new_size as i32, new_size as i32);

            let prev_atlas = self.texture.get_data();

            let texture = Texture::new(new_size, new_size);
            texture.bind();

            texture.set_filtering(Filtering::Nearest, Filtering::Nearest);
            texture.set_wrap(Wrap::ClampToEdge);

            texture.set_data(&prev_atlas);

            self.texture = texture;

            Texture::unbind();
        }

        let rect = self
            .packer
            .pack(width as i32, height as i32, false)
            .expect("Cannot fail here");

        self.regions.insert(label.as_ref().into(), rect);

        rect
    }

    pub fn buffer_region(&mut self, x: f32, y: f32, z: f32, region: &Rect) {
        let color: [f32; 4] = Color::WHITE.into();

        let width = region.width as f32;
        let height = region.height as f32;

        let tl = Vertex {
            position: [x, y, z],
            color,
            tex_coords: [
                region.x as f32 / self.texture.width() as f32,
                region.y as f32 / self.texture.height() as f32,
            ],
        };

        let tr = Vertex {
            position: [x + width, y, z],
            color,
            tex_coords: [
                (region.x + region.width) as f32 / self.texture.width() as f32,
                region.y as f32 / self.texture.height() as f32,
            ],
        };

        let br = Vertex {
            position: [x + width, y + height, z],
            color,
            tex_coords: [
                (region.x + region.width) as f32 / self.texture.width() as f32,
                (region.y + region.height) as f32 / self.texture.height() as f32,
            ],
        };

        let bl = Vertex {
            position: [x, y + height, z],
            color,
            tex_coords: [
                region.x as f32 / self.texture.width() as f32,
                (region.y + region.height) as f32 / self.texture.height() as f32,
            ],
        };

        let index = self.vertices.len() as u32;

        self.vertices.push(tl);
        self.vertices.push(tr);
        self.vertices.push(br);
        self.vertices.push(bl);

        self.indices.push(index);
        self.indices.push(index + 1);
        self.indices.push(index + 2);
        self.indices.push(index + 2);
        self.indices.push(index + 3);
        self.indices.push(index);
    }

    pub fn load_image<L: AsRef<str>, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let image = image::open(path)
            .map_err(|e| KarnaError::LoadResource(e.to_string()))
            .unwrap()
            .to_rgba8();

        let (width, height) = image.dimensions();

        let rect = self.cache_region(label, width, height);

        self.texture.bind();

        self.texture
            .set_sub_data(rect.x, rect.y, width, height, &image);

        Texture::unbind();
    }

    /// Draws the image with the specified label at the specified position.
    ///
    /// # Panics
    /// Panics if the label is not found in the atlas.
    #[inline]
    pub fn draw_image<L: AsRef<str>>(&mut self, label: L, x: f32, y: f32, z: f32) {
        // TODO: Handle error
        let region = *self.regions.get(label.as_ref()).expect("Wrong label");

        self.buffer_region(x, y, z, &region);
    }

    pub fn draw_atlas(&mut self, x: f32, y: f32, z: f32) {
        let color: [f32; 4] = Color::WHITE.into();

        let width = self.texture.width() as f32;
        let height = self.texture.height() as f32;

        let tl = Vertex {
            position: [x, y, z],
            color,
            tex_coords: [0.0, 0.0],
        };

        let tr = Vertex {
            position: [x + width, y, z],
            color,
            tex_coords: [1.0, 0.0],
        };

        let br = Vertex {
            position: [x + width, y + height, z],
            color,
            tex_coords: [1.0, 1.0],
        };

        let bl = Vertex {
            position: [x, y + height, z],
            color,
            tex_coords: [0.0, 1.0],
        };

        let index = self.vertices.len() as u32;

        self.vertices.push(tl);
        self.vertices.push(tr);
        self.vertices.push(br);
        self.vertices.push(bl);

        self.indices.push(index);
        self.indices.push(index + 1);
        self.indices.push(index + 2);
        self.indices.push(index + 2);
        self.indices.push(index + 3);
        self.indices.push(index);
    }

    pub fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        bind_vertex_buffer(0, &self.vbo, std::mem::size_of::<Vertex>());

        self.vbo.bind();
        self.ebo.bind();

        self.vbo.buffer_data(&self.vertices);
        self.ebo.buffer_data(&self.indices);

        draw_elements(
            DrawMode::Triangles,
            self.indices.len(),
            DataType::UnsignedInt,
            std::ptr::null(),
        );

        self.vertices.clear();
        self.indices.clear();
    }
}
