use gpu::Texture;
use image::{DynamicImage, GenericImageView};
use macros::Get;
use math::Size;
use rect_packer::{Packer, Rect};
use std::sync::{Mutex, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use utils::map::{Label, LabelMap};

static TEXTURE_ATLAS: OnceLock<TextureAtlas> = OnceLock::new();

pub fn init() {
    TEXTURE_ATLAS
        .set(TextureAtlas::new((1024, 1024)))
        .map_err(|_| "Failed to initialize texture atlas")
        .unwrap()
}

#[inline]
pub fn get() -> &'static TextureAtlas {
    TEXTURE_ATLAS.get().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<rect_packer::Rect> for AtlasRegion {
    fn from(value: rect_packer::Rect) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}

impl AtlasRegion {
    #[inline]
    /// (min_x, min_y, max_x, max_y)
    pub fn uv_coordinates(&self, atlas_size: Size<u32>) -> (f32, f32, f32, f32) {
        let x = self.x as f32 / atlas_size.width() as f32;
        let y = self.y as f32 / atlas_size.height() as f32;
        let width = self.width as f32 / atlas_size.width() as f32;
        let height = self.height as f32 / atlas_size.height() as f32;

        (x, y, width, height)
    }
}

#[derive(Get)]
pub struct TextureAtlas {
    texture: Texture,
    #[get]
    bind_group_layout: wgpu::BindGroupLayout,
    size: Size<u32>,
    /// Can be a mutex, since it's mainly used for writing (lock, but it's ok, loading images takes time),
    /// when getting a texture we don't need to lock it
    packer: Mutex<rect_packer::Packer>,
    regions: RwLock<LabelMap<AtlasRegion>>,
}

impl TextureAtlas {
    pub fn new<S: Into<Size<u32>>>(size: S) -> Self {
        let size = size.into();
        let device = gpu::device();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture atlas bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let texture = Texture::new_empty("texture atlas", size, &bind_group_layout, device);

        let packer = Packer::new(rect_packer::Config {
            width: size.width() as i32,
            height: size.height() as i32,
            border_padding: 0,
            rectangle_padding: 0,
        });

        Self {
            texture,
            bind_group_layout,
            size,
            packer: Mutex::new(packer),
            regions: RwLock::new(LabelMap::default()),
        }
    }

    fn write_image(&self, image: DynamicImage, size: Size<u32>, region: AtlasRegion) {
        gpu::queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: self.texture.inner(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x as u32,
                    y: region.y as u32,
                    z: 0,
                },
            },
            image.as_rgba8().expect("Failed to get rgba8"),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width()),
                rows_per_image: Some(size.height()),
            },
            wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    #[inline]
    pub fn add_image(&self, label: Label, bytes: Vec<u8>) {
        let image = image::load_from_memory(&bytes).expect("Failed to load image");
        let (width, height) = image.dimensions();
        let size = Size::new(width, height);
        let mut packer = self.packer.lock().expect("Failed to write to packer");

        let region: AtlasRegion = packer
            .pack(size.width() as i32, size.height() as i32, false)
            .expect("Failed to pack image")
            .into();

        self.write_image(image, size, region);

        let mut regions = self
            .regions
            .write()
            .expect("Texture atlas lock is poisoned");

        regions.insert(label, region);
    }
}
