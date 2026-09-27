use core::slice;

use alloc::vec::Vec;
use sdl3_sys::*;

use crate::window::Window;

pub type MonitorId = SDL_DisplayID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monitor {
    pub id: MonitorId,
    pub position: math::Vector2<i32>,
    pub size: math::Size<u32>,
    pub pixel_density: f32,
    pub refresh_rate: f32,
}

impl Monitor {
    pub fn all() -> Vec<Monitor> {
        let mut count = 0;
        let ptr = unsafe { SDL_GetDisplays(&mut count) };

        if ptr.is_null() {
            return Vec::new();
        }

        let monitors = unsafe { slice::from_raw_parts(ptr, count.max(0) as usize) }
            .iter()
            .filter_map(|&id| Self::query(id))
            .collect();

        unsafe { SDL_free(ptr.cast()) };
        monitors
    }

    pub fn primary() -> Option<Monitor> {
        Self::query(unsafe { SDL_GetPrimaryDisplay() })
    }

    pub fn for_window(window: &Window) -> Option<Monitor> {
        Self::query(unsafe { SDL_GetDisplayForWindow(window.as_ptr()) })
    }

    fn query(id: SDL_DisplayID) -> Option<Monitor> {
        if id == 0 {
            return None;
        }

        let mut r = SDL_Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };

        if !unsafe { SDL_GetDisplayBounds(id, &mut r) } {
            return None;
        }

        let mode = unsafe { SDL_GetCurrentDisplayMode(id).as_ref() }?;

        Some(Self {
            id,
            position: math::Vector2::new(r.x, r.y),
            size: math::size!(r.w.max(0) as u32, r.h.max(0) as u32),
            pixel_density: mode.pixel_density,
            refresh_rate: mode.refresh_rate,
        })
    }
}
