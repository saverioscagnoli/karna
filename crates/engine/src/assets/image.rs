use std::ffi::CString;
use std::path::Path;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice;

use crossbeam_channel::Sender;
use logging::error;
use logging::fatal;
use logging::warn;
use sdl3::SDL_ConvertSurface;
use sdl3::SDL_DestroySurface;
use sdl3::SDL_IOFromConstMem;
use sdl3::SDL_LockSurface;
use sdl3::SDL_PixelFormat;
use sdl3::SDL_Surface;
use sdl3::SDL_UnlockSurface;
use sdl3_image::IMG_Load;
use sdl3_image::IMG_Load_IO;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::Key::H;
use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetServer;
use crate::assets::AssetSlot;
use crate::assets::AssetSource;
use crate::assets::atlas::TextureAtlas;
use crate::err::SDL_LastError;
use crate::gpu::Gpu;
use crate::gpu::texture::Filter;
use crate::gpu::texture::Texture;

const TARGET_FORMAT: SDL_PixelFormat = SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888;

#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub page: usize,
    pub uv_min: math::Vector2<f32>,
    pub uv_max: math::Vector2<f32>,
    pub size: math::Size<u32>,
}

impl Image {
    pub fn uv_center(&self) -> math::Vector2<f32> {
        let u = (self.uv_min.x + self.uv_max.x) * 0.5;
        let v = (self.uv_min.y + self.uv_max.y) * 0.5;
        math::Vector2::new(u, v)
    }
}

pub struct DecodedImage {
    pub size: math::Size<u32>,
    pub pixels: Vec<u8>,
}

struct OwnedSurface(NonNull<SDL_Surface>);

impl OwnedSurface {
    unsafe fn take(ptr: *mut SDL_Surface) -> Result<Self, String> {
        NonNull::new(ptr)
            .map(Self)
            .ok_or_else(|| SDL_LastError().to_string())
    }

    fn as_ptr(&self) -> *mut SDL_Surface {
        self.0.as_ptr()
    }

    fn format(&self) -> SDL_PixelFormat {
        unsafe { (*self.as_ptr()).format }
    }

    fn into_format(self, format: SDL_PixelFormat) -> Result<Self, String> {
        if self.format() == format {
            return Ok(self);
        }

        unsafe { Self::take(SDL_ConvertSurface(self.as_ptr(), format)) }
    }
}

impl Drop for OwnedSurface {
    fn drop(&mut self) {
        unsafe { SDL_DestroySurface(self.as_ptr()) }
    }
}

fn to_decoded(surface: OwnedSurface) -> Result<DecodedImage, String> {
    let surface = surface.into_format(TARGET_FORMAT)?;
    let raw = surface.as_ptr();

    let (width, height, pitch) = unsafe { ((*raw).w, (*raw).h, (*raw).pitch) };

    let width = u32::try_from(width).map_err(|_| String::from("surface has negative width"))?;
    let height = u32::try_from(height).map_err(|_| String::from("surface has negative height"))?;
    let pitch = usize::try_from(pitch).map_err(|_| String::from("surface has negative pitch"))?;

    let row_bytes = width as usize * 4;

    if pitch < row_bytes {
        return Err(format!(
            "surface pitch {pitch} is shorter than a {row_bytes}-byte row"
        ));
    }

    let mut pixels = Vec::with_capacity(row_bytes * height as usize);

    unsafe {
        if !SDL_LockSurface(raw) {
            return Err(SDL_LastError().to_string());
        }

        let base = (*raw).pixels as *const u8;

        if base.is_null() {
            SDL_UnlockSurface(raw);
            return Err(String::from("surface has no pixel data"));
        }

        if pitch == row_bytes {
            pixels.extend_from_slice(slice::from_raw_parts(base, row_bytes * height as usize));
        } else {
            for y in 0..height as usize {
                let row = slice::from_raw_parts(base.add(y * pitch), row_bytes);
                pixels.extend_from_slice(row);
            }
        }

        SDL_UnlockSurface(raw);
    }

    Ok(DecodedImage {
        size: math::Size::new(width, height),
        pixels,
    })
}

pub fn decode_image(path: &Path) -> Result<DecodedImage, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display()))?;

    let c_path = CString::new(path_str).map_err(|_| String::from("path contains a NUL byte"))?;

    let surface = unsafe { OwnedSurface::take(IMG_Load(c_path.as_ptr()))? };

    to_decoded(surface)
}

pub fn decode_image_bytes(bytes: &[u8]) -> Result<DecodedImage, String> {
    if bytes.is_empty() {
        return Err(String::from("empty image buffer"));
    }

    let surface = unsafe {
        let io = SDL_IOFromConstMem(bytes.as_ptr().cast(), bytes.len());

        if io.is_null() {
            return Err(SDL_LastError().to_string());
        }

        OwnedSurface::take(IMG_Load_IO(io, true))?
    };

    to_decoded(surface)
}

pub struct ImageRegistry {
    pub atlas: TextureAtlas,
    pub slots: SlotMap<AssetSlot<Image>>,
    pub paths: FastHashMap<PathBuf, Handle<Image>>,
    pub white_texel: Handle<Image>,
    pub placeholder: Handle<Image>,
}

impl ImageRegistry {
    const WHITE_TEXEL_BYTES: &'static [u8] = include_bytes!("../../../../assets/white-texel.png");
    const PLACEHOLDER_IMAGE_BYTES: &'static [u8] =
        include_bytes!("../../../../assets/placeholder.png");

    pub fn new() -> Self {
        let mut this = Self {
            atlas: TextureAtlas::new(),
            slots: SlotMap::new(),
            paths: FastHashMap::default(),
            white_texel: Handle::INVALID,
            placeholder: Handle::INVALID,
        };

        this.white_texel = this.bake(&Self::WHITE_TEXEL_BYTES);
        this.placeholder = this.bake(Self::PLACEHOLDER_IMAGE_BYTES);

        this
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
            source: AssetSource::Path(path.clone()),
            kind: AssetKind::Image(Filter::default()),
        };

        if sender.send(request).is_err() {
            warn!(
                "Asset workers are gone, cannot load image '{}'",
                path.display()
            );

            self.slots[handle.cast()] = AssetSlot::Failed;
        }

        handle
    }

    pub fn load_bytes(&mut self, bytes: &[u8], sender: &Sender<AssetRequest>) -> Handle<Image> {
        let handle: Handle<Image> = self.slots.insert(AssetSlot::Pending).cast();
        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Bytes(bytes.to_vec()),
            kind: AssetKind::Image(Filter::default()),
        };

        if sender.send(request).is_err() {
            warn!("Asset workers are gone, cannot load image bytes.");
            self.slots[handle.cast()] = AssetSlot::Failed;
        }

        handle
    }

    pub fn bake(&mut self, bytes: &[u8]) -> Handle<Image> {
        let handle: Handle<Image> = self.slots.insert(AssetSlot::Pending).cast();

        match decode_image_bytes(bytes) {
            Ok(decoded) => {
                let image = self.atlas.insert(&decoded, handle, Filter::default());
                self.slots[handle.cast()] = AssetSlot::Ready(image);
            }

            Err(e) => {
                error!("Failed to load image bytes: {}", e);
                self.slots[handle.cast()] = AssetSlot::Failed;
            }
        }

        handle
    }

    pub fn get(&self, handle: Handle<Image>) -> &Image {
        if let Some(AssetSlot::Ready(image)) = self.slots.get(handle.cast()) {
            return image;
        }

        match self.slots.get(self.placeholder.cast()) {
            Some(AssetSlot::Ready(p)) => p,
            _ => fatal!("Image fallback is missing"),
        }
    }
}

impl AssetServer {
    pub fn load_image<P>(&mut self, path: P) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        self.images.load_path(path, &self.requests)
    }

    pub fn load_image_bytes(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.load_bytes(bytes, &self.requests)
    }

    pub fn bake_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.bake(bytes)
    }

    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.images.get(handle)
    }

    pub fn white_texel(&self) -> Handle<Image> {
        self.images.white_texel
    }

    pub fn placeholder_image(&self) -> Handle<Image> {
        self.images.placeholder
    }

    pub fn set_placeholder_image(&mut self, handle: Handle<Image>) {
        self.images.placeholder = handle;
    }

    pub(crate) fn upload_dirty_images(&mut self, gpu: &Rc<Gpu>) {
        self.images.atlas.upload_dirty(gpu);
    }

    pub(crate) fn atlas_page_texture(&self, page: usize) -> Option<&Texture> {
        self.images.atlas.page_texture(page)
    }
}
