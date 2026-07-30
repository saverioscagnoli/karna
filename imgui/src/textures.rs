use std::slice;

use dear_imgui_sys::*;
use gpu::Gpu;
use logging::debug;
use utils::FastHashMap;

/// All the textures that imgui asked to create.Z
/// Separate from the engine texture atlas
pub struct ImguiTextureRegistry {
    textures: FastHashMap<ImTextureID, gpu::Texture>,
    next_id: ImTextureID,
}

impl ImguiTextureRegistry {
    pub fn new() -> Self {
        Self {
            textures: FastHashMap::default(),
            next_id: 1,
        }
    }

    pub fn get(&self, id: ImTextureID) -> Option<&gpu::Texture> {
        self.textures.get(&id)
    }

    unsafe fn create(&mut self, gpu: &Gpu, tex: *mut ImTextureData) {
        // Why tf do i have to put unsafe inside an unsafe function
        // This language sucks ass
        unsafe {
            let width = (*tex).Width as u32;
            let height = (*tex).Height as u32;

            debug_assert_eq!((*tex).BytesPerPixel, 4);

            let pixels =
                slice::from_raw_parts((*tex).Pixels as *const u8, (width * height * 4) as usize);

            let id = self.next_id;
            self.next_id += 1;

            // Imgui's font atlas is already antialiased
            let texture =
                gpu::Texture::from_rgba(gpu, format!("imgui-{id}"), width, height, pixels)
                    .with_filter(gpu::Filter::Linear);

            self.textures.insert(id, texture);

            ImTextureData_SetTexID(tex, id);
            ImTextureData_SetStatus(tex, ImTextureStatus_OK);
        }
    }

    unsafe fn stage_updates<'a>(
        &'a self,
        batch: &mut gpu::TextureBatch<'a>,
        tex: *mut ImTextureData,
    ) {
        // See the very important comment in Self::create
        unsafe {
            let id = ImTextureData_GetTexID(tex);

            let Some(texture) = self.textures.get(&id) else {
                return;
            };

            let pitch = ImTextureData_GetPitch(tex) as usize;
            let rect = (*tex).UpdateRect;
            let (x, y) = (rect.x as u32, rect.y as u32);
            let (w, h) = (rect.w as u32, rect.h as u32);

            let mut region = Vec::with_capacity((w * h * 4) as usize);

            for row in 0..h {
                let start = (y + row) as usize * pitch + x as usize * 4;
                let src = slice::from_raw_parts((*tex).Pixels.add(start), (w * 4) as usize);

                region.extend_from_slice(src);
            }

            batch.stage(texture, x, y, w, h, &region);

            ImTextureData_SetStatus(tex, ImTextureStatus_OK);
        }
    }

    unsafe fn destroy(&mut self, tex: *mut ImTextureData) {
        // See the very important comment in Self::create
        unsafe {
            if (*tex).UnusedFrames < 2 {
                return;
            }

            let id = ImTextureData_GetTexID(tex);
            self.textures.remove(&id);

            ImTextureData_SetTexID(tex, 0);
            ImTextureData_SetStatus(tex, ImTextureStatus_Destroyed);

            debug!("Destroyed imgui texture {}", id);
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn update(&mut self, gpu: &Gpu, draw_data: *mut ImDrawData) {
        unsafe {
            let list = (*draw_data).Textures;

            if list.is_null() {
                return;
            }

            let mut batch = gpu::TextureBatch::new();
            let mut staged = Vec::new();

            for i in 0..(*list).Size {
                let tex = *(*list).Data.offset(i as isize);

                if tex.is_null() {
                    continue;
                }

                match (*tex).Status {
                    ImTextureStatus_WantCreate => self.create(gpu, tex),
                    ImTextureStatus_WantUpdates => staged.push(tex),
                    ImTextureStatus_WantDestroy => self.destroy(tex),
                    _ => {}
                };
            }

            for tex in staged {
                self.stage_updates(&mut batch, tex);
            }

            batch.submit(gpu);
        }
    }
}
