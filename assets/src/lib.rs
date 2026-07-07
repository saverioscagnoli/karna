mod font;
mod image;
mod mesh;

use std::sync::Arc;

use gpu::Vertex;
use logging::info;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use utils::ByteSize;
use utils::Handle;

pub use crate::font::Font;
use crate::font::FontAtlas;
pub use crate::font::GlyphInfo;
use crate::font::GlyphKey;
pub use crate::image::Image;
use crate::image::TextureAtlas;
pub use crate::mesh::Geometry;
pub use crate::mesh::Material;
pub use crate::mesh::MaterialDesc;
use crate::mesh::Meshes;

#[derive(Clone)]
pub struct AssetServer {
    atlas: Arc<RwLock<TextureAtlas>>,
    fonts_atlas: Arc<RwLock<FontAtlas>>,
    meshes: Arc<RwLock<Meshes>>,
}

pub struct AssetServerGuard<'a> {
    atlas: RwLockReadGuard<'a, TextureAtlas>,
    font_atlas: RwLockReadGuard<'a, FontAtlas>,
    meshes: RwLockReadGuard<'a, Meshes>,
}

impl AssetServer {
    #[doc(hidden)]
    pub fn _new() -> Self {
        let mut atlas = TextureAtlas::new((1024, 1024));
        let font_atlas = FontAtlas::new(&mut atlas);

        Self {
            atlas: Arc::new(RwLock::new(atlas)),
            fonts_atlas: Arc::new(RwLock::new(font_atlas)),
            meshes: Arc::new(RwLock::new(Meshes::new())),
        }
    }

    #[doc(hidden)]
    pub fn _guard<'a>(&'a self) -> AssetServerGuard<'a> {
        AssetServerGuard {
            atlas: self.atlas.read(),
            font_atlas: self.fonts_atlas.read(),
            meshes: self.meshes.read(),
        }
    }

    /// Only png accepted for now
    pub fn load_image(&self, bytes: &[u8]) -> Handle<Image> {
        self.atlas.write().load_image(bytes)
    }

    pub fn load_font(&self, bytes: &[u8], size: u16) -> Handle<Font> {
        let mut atlas = self.atlas.write();
        let mut font_atlas = self.fonts_atlas.write();
        let size_bytes = bytes.len() as u64;

        let font_vec =
            ab_glyph::FontVec::try_from_vec(bytes.to_vec()).expect("Failed to read font");

        info!("Loaded font with size={}", ByteSize::from_bytes(size_bytes));

        font_atlas.register_font(&mut atlas, font_vec, size)
    }

    pub fn create_geometry(&self, vertices: &[Vertex], indices: &[u32]) -> Handle<Geometry> {
        self.meshes.write().create_geometry(vertices, indices)
    }

    pub fn create_material(&self, desc: MaterialDesc) -> Handle<Material> {
        self.meshes
            .write()
            .create_material(desc, &self.atlas.read())
    }
}

impl<'a> AssetServerGuard<'a> {
    pub fn white_handle(&self) -> Handle<Image> {
        self.atlas.white
    }

    pub fn atlas_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.atlas.bgl
    }

    pub fn atlas_bg(&self) -> &wgpu::BindGroup {
        &self.atlas.texture.bind_group
    }

    pub fn atlas_handle(&self) -> Handle<Image> {
        self.atlas.handle
    }

    pub fn atlas_size(&self) -> &math::Size<u32> {
        &self.atlas.size
    }

    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.atlas.images.get(handle).expect("Failed to get image")
    }

    pub fn get_font(&self, font: Handle<Font>) -> &Font {
        self.font_atlas.fonts.get(font).expect("Failed to get font")
    }

    pub fn get_glyph(&self, font: Handle<Font>, ch: char, size: u16) -> &GlyphInfo {
        let key = GlyphKey {
            font,
            ch,
            size_px: size,
        };

        self.font_atlas
            .glyphs
            .get(&key)
            .expect("Failed to get glyph")
    }

    pub fn debug_font_handle(&self) -> Handle<Font> {
        self.font_atlas.debug_font
    }

    pub fn get_geometry(&self, geometry: Handle<Geometry>) -> &Geometry {
        self.meshes.get_geometry(geometry)
    }

    pub fn get_material(&self, material: Handle<Material>) -> &Material {
        self.meshes.get_material(material)
    }

    pub fn material_bgl(&self) -> &wgpu::BindGroupLayout {
        self.meshes.material_bgl()
    }
}
