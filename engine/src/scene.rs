use renderer::Draw;
use utils::FastHashMap;

use crate::context::ContextRef;
use crate::context::ContextRefMut;

pub trait Scene {
    fn load(&mut self, ctx: ContextRefMut);

    fn update(&mut self, ctx: ContextRefMut);

    fn fixed_update(&mut self, ctx: ContextRefMut) {
        let _ = ctx;
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw);

    fn cleanup(&mut self, ctx: ContextRefMut) {
        let _ = ctx;
    }
}

pub type Scenes = FastHashMap<String, Box<dyn Scene>>;
