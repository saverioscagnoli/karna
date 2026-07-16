pub use assets;
pub use engine::App;
pub use engine::AppBuilder;
pub use engine::ContextMut;
pub use engine::Scene;
pub use engine::SceneHandle;
pub use engine::SceneManager;
pub use engine::WindowBuilder;
pub use engine::input;
#[cfg(feature = "net")]
pub use engine::net;
// Re-export imgui
pub use imgui;
pub use logging;
pub use math;
pub use utils::Handle;

pub mod render {
    pub use engine::Draw;
    pub use renderer::*;
}

pub mod prelude {
    pub use assets::*;
    #[cfg(feature = "net")]
    pub use engine::net::*;
    pub use engine::*;
    pub use input::*;
    pub use math::*;
    pub use render::*;

    pub use crate::*;
}
