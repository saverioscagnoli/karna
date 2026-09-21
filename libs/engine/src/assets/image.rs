use core::hash::Hash;
use core::hash::Hasher;

use nostd::alloc::vec::Vec;
use nostd::collections::FxHasher;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use nostd::path::Path;
use nostd::path::PathBuf;
use nostd::sync::Sender;
use sdl3::gpu::Device;
use sdl3::image::DecodedImage;
use traccia::error;
use traccia::info;

use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetSlot;
use crate::assets::AssetSource;
use crate::assets::TextureAtlas;

#[derive(Debug, Clone, Copy)]
pub struct Image {
    size: math::Size<u32>,
    page: u32,
    uv_min: math::Vector2<f32>,
    uv_max: math::Vector2<f32>,
}

impl Image {
    pub(crate) fn new(
        size: math::Size<u32>,
        page: u32,
        uv_min: math::Vector2<f32>,
        uv_max: math::Vector2<f32>,
    ) -> Self {
        Self {
            size,
            page,
            uv_min,
            uv_max,
        }
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn page(&self) -> u32 {
        self.page
    }

    pub fn uv_min(&self) -> math::Vector2<f32> {
        self.uv_min
    }

    pub fn uv_max(&self) -> math::Vector2<f32> {
        self.uv_max
    }
}

pub struct ImageRegistry {
    pub atlas: TextureAtlas,
    pub slots: SlotMap<AssetSlot<Image>>,
    pub paths: HashMap<PathBuf, Handle<Image>>,
    pub bytes: HashMap<u64, Handle<Image>>,
    pub white_texel: Handle<Image>,
    pub placeholder: Handle<Image>,
}

impl ImageRegistry {
    pub const WHITE_TEXEL_BYTES: &'static [u8] =
        include_bytes!("../../../../assets/white-texel.png");

    pub const PLACEHOLDER_IMAGE_BYTES: &'static [u8] =
        include_bytes!("../../../../assets/placeholder.png");

    pub fn new(device: &Device) -> Self {
        Self {
            atlas: TextureAtlas::new(device.share(), 2048, 32),
            slots: SlotMap::default(),
            paths: HashMap::default(),
            bytes: HashMap::default(),
            white_texel: Handle::INVALID,
            placeholder: Handle::INVALID,
        }
    }

    /// Returns the image if it has finished loading.
    pub fn resolve(&self, handle: Handle<Image>) -> Option<Image> {
        match self.slots.get(handle.cast())? {
            AssetSlot::Ready(image) => Some(*image),
            _ => None,
        }
    }

    pub fn load_path<P>(&mut self, path: P, sender: &Sender<AssetRequest>) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        if let Some(handle) = self.paths.get(&path) {
            return *handle;
        }

        let handle: Handle<Image> = self.slots.insert(AssetSlot::Pending).cast();
        self.paths.insert(path.clone(), handle);

        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Path(path),
            kind: AssetKind::Image,
        };

        if sender.send(request).is_err() {
            error!("Asset workers are gone, cannot load more assets.");
            self.slots[handle.cast()] = AssetSlot::Failed("Asset worker stopped.".into());
        }

        handle
    }

    pub fn load_bytes(&mut self, bytes: Vec<u8>, sender: &Sender<AssetRequest>) -> Handle<Image> {
        let mut hasher = FxHasher::default();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(handle) = self.bytes.get(&hash) {
            return *handle;
        }

        let handle: Handle<Image> = self.slots.insert(AssetSlot::Pending).cast();
        self.bytes.insert(hash, handle);

        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Bytes(bytes),
            kind: AssetKind::Image,
        };

        if sender.send(request).is_err() {
            error!("Asset workers are gone, cannot load more assets.");
            self.slots[handle.cast()] = AssetSlot::Failed("Asset worker stopped.".into());
        }

        handle
    }

    pub fn bake(&mut self, bytes: Vec<u8>) -> Handle<Image> {
        let handle: Handle<Image> = self.slots.insert(AssetSlot::Pending).cast();

        match DecodedImage::from_bytes(&bytes) {
            Ok(dec) => {
                let Some(image) = self.atlas.insert(&dec) else {
                    error!("Failed to bake image, no space in atlas.");
                    return Handle::INVALID;
                };

                info!("Baked image {:?} (page {})", image.size(), image.page());
                self.slots[handle.cast()] = AssetSlot::Ready(image)
            }

            Err(e) => {
                error!("Failed to decode image bytes: {}", e);
                self.slots[handle.cast()] = AssetSlot::Failed("Decode failed".into());
            }
        }

        handle
    }
}
