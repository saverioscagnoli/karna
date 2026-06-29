use renderer::Draw;
use utils::IndexMap;

use crate::context::ContextRef;
use crate::context::ContextRefMut;

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: ContextRefMut);
    fn update(&mut self, ctx: ContextRefMut);
    fn fixed_update(&mut self, ctx: ContextRefMut) {}
    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw);
}

pub type Scenes = IndexMap<Box<dyn Scene>>;

pub struct SceneManager {
    pub(crate) pending_activate: Vec<String>,
    pub(crate) pending_deactivate: Vec<String>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self {
            pending_activate: Vec::new(),
            pending_deactivate: Vec::new(),
        }
    }

    pub fn activate<L: Into<String>>(&mut self, label: L) {
        self.pending_activate.push(label.into());
    }

    pub fn deactivate<L: Into<String>>(&mut self, label: L) {
        self.pending_deactivate.push(label.into());
    }
}
