use std::path::Path;

use sdl2::{pixels::PixelFormatEnum, surface::Surface};

use crate::Context;

pub trait Scene {
    fn load(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context);
    fn fixed_update(&mut self, ctx: &mut Context);
    fn draw(&mut self, ctx: &mut Context);
}

pub(crate) trait LoadSurface {
    fn from_file<P: AsRef<Path>>(path: P) -> Self;
}

impl LoadSurface for Surface<'_> {
    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let img = image::open(path).unwrap().to_rgba8();
        let (width, height) = img.dimensions();

        let mut surface = Surface::new(width, height, PixelFormatEnum::RGBA32).unwrap();

        surface.with_lock_mut(|pixels| pixels.copy_from_slice(&img));

        surface
    }
}
