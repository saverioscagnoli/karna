mod audio;
mod font;
mod image;
mod material;
mod mesh;

use std::ops::Deref;
use std::sync::Arc;

use gpu::Vertex;
use logging::info;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use utils::ByteSize;
use utils::Handle;

pub use crate::audio::Audio;
use crate::audio::AudioRegistry;
pub use crate::font::Font;
use crate::font::FontAtlas;
pub use crate::font::GlyphInfo;
use crate::font::GlyphKey;
pub use crate::image::Image;
use crate::image::TextureAtlas;
pub use crate::material::Material;
pub use crate::material::MaterialDesc;
pub use crate::mesh::Geometry;
use crate::mesh::MeshAssets;

#[derive(Clone)]
pub struct AssetServer {
    atlas: Arc<RwLock<TextureAtlas>>,
    fonts_atlas: Arc<RwLock<FontAtlas>>,
    meshes: MeshAssets,
    audios: AudioRegistry,
}

mod sealed {
    pub trait Sealed {}
}

pub struct ReadOnly;
pub struct ReadWrite;

pub trait AssetAccess: sealed::Sealed {
    type AtlasGuard<'a>: Deref<Target = TextureAtlas>;
    type FontAtlasGuard<'a>: Deref<Target = FontAtlas>;
    type MeshAssetsGuard<'a>: Deref<Target = MeshAssets>;
    type AudioRegistryGuard<'a>: Deref<Target = AudioRegistry>;
}

impl sealed::Sealed for ReadOnly {}
impl sealed::Sealed for ReadWrite {}

impl AssetAccess for ReadOnly {
    type AtlasGuard<'a> = RwLockReadGuard<'a, TextureAtlas>;
    type FontAtlasGuard<'a> = RwLockReadGuard<'a, FontAtlas>;
    type MeshAssetsGuard<'a> = &'a MeshAssets;
    type AudioRegistryGuard<'a> = &'a AudioRegistry;
}

impl AssetAccess for ReadWrite {
    type AtlasGuard<'a> = RwLockWriteGuard<'a, TextureAtlas>;
    type FontAtlasGuard<'a> = RwLockWriteGuard<'a, FontAtlas>;
    type MeshAssetsGuard<'a> = &'a mut MeshAssets;
    type AudioRegistryGuard<'a> = &'a mut AudioRegistry;
}

pub struct AssetServerView<'a, A: AssetAccess> {
    atlas: A::AtlasGuard<'a>,
    font_atlas: A::FontAtlasGuard<'a>,
    meshes: A::MeshAssetsGuard<'a>,
    audios: A::AudioRegistryGuard<'a>,
}

impl AssetServer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let mut atlas = TextureAtlas::new((1024, 1024));
        let font_atlas = FontAtlas::new(&mut atlas);

        Self {
            atlas: Arc::new(RwLock::new(atlas)),
            fonts_atlas: Arc::new(RwLock::new(font_atlas)),
            meshes: MeshAssets::new(),
            audios: AudioRegistry::new(),
        }
    }

    #[doc(hidden)]
    pub fn rguard<'a>(&'a self) -> AssetServerView<'a, ReadOnly> {
        AssetServerView {
            atlas: self.atlas.read(),
            font_atlas: self.fonts_atlas.read(),
            meshes: &self.meshes,
            audios: &self.audios,
        }
    }

    #[doc(hidden)]
    pub fn wguard<'a>(&'a mut self) -> AssetServerView<'a, ReadWrite> {
        AssetServerView {
            atlas: self.atlas.write(),
            font_atlas: self.fonts_atlas.write(),
            meshes: &mut self.meshes,
            audios: &mut self.audios,
        }
    }
}

/// Read only method are defined here, because they will still need to be
/// accessible in the writable view
impl<'a, A: AssetAccess> AssetServerView<'a, A> {
    #[doc(hidden)]
    pub fn atlas_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.atlas.bgl
    }

    #[doc(hidden)]
    pub fn atlas_bg(&self) -> &wgpu::BindGroup {
        &self.atlas.texture.bind_group
    }

    #[doc(hidden)]
    pub fn material_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.meshes.material_bgl
    }

    pub fn white_handle(&self) -> Handle<Image> {
        self.atlas.white
    }

    pub fn atlas_handle(&self) -> Handle<Image> {
        self.atlas.handle
    }

    pub fn atlas_size(&self) -> math::Size<u32> {
        self.atlas.size
    }

    pub fn debug_font_handle(&self) -> Handle<Font> {
        self.font_atlas.debug_font
    }

    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.atlas.images.get(handle).expect("Failed to get image")
    }

    pub fn get_font(&self, handle: Handle<Font>) -> &Font {
        self.font_atlas
            .fonts
            .get(handle)
            .expect("Failed to get font")
    }

    pub fn get_glyph(&self, font_handle: Handle<Font>, ch: char, size: u16) -> &GlyphInfo {
        let key = GlyphKey {
            font: font_handle,
            ch,
            size_px: size,
        };

        self.font_atlas
            .glyphs
            .get(&key)
            .expect("Failed to get glyph")
    }

    pub fn get_geometry(&self, handle: Handle<Geometry>) -> &Geometry {
        self.meshes
            .geometries
            .get(handle)
            .expect("Failed to get geometry")
    }

    pub fn get_material(&self, handle: Handle<Material>) -> &Material {
        self.meshes
            .materials
            .get(handle)
            .expect("Failed to get material")
    }

    pub fn get_audio(&self, handle: Handle<Audio>) -> &Audio {
        self.audios
            .registry
            .get(handle)
            .expect("Failed to load audio")
    }
}

/// Write only methods
impl<'a> AssetServerView<'a, ReadWrite> {
    pub fn load_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.atlas.load_image(bytes)
    }

    pub fn load_font(&mut self, bytes: &[u8], size: u16) -> Handle<Font> {
        let size_bytes = bytes.len() as u64;
        let font_vec =
            ab_glyph::FontVec::try_from_vec(bytes.to_vec()).expect("Failed to read font");

        info!("Loaded font with size={}", ByteSize::from_bytes(size_bytes));

        self.font_atlas
            .register_font(&mut self.atlas, font_vec, size)
    }

    pub fn load_geometry(&mut self, vertices: &[Vertex], indices: &[u32]) -> Handle<Geometry> {
        self.meshes
            .geometries
            .insert(Geometry::new(vertices, indices))
    }

    pub fn load_material(&mut self, desc: MaterialDesc) -> Handle<Material> {
        self.meshes
            .materials
            .insert(Material::new(desc, &self.atlas, self.material_bgl()))
    }

    pub fn load_audio(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.audios.load_audio(bytes)
    }

    pub fn get_geometry_mut(&mut self, handle: Handle<Geometry>) -> &mut Geometry {
        self.meshes
            .geometries
            .get_mut(handle)
            .expect("Failed to get geometry")
    }

    pub fn get_material_mut(&mut self, handle: Handle<Material>) -> &mut Material {
        self.meshes
            .materials
            .get_mut(handle)
            .expect("Failed to get material")
    }
}
