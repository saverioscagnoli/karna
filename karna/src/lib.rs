#![no_std]

pub use engine::App;
pub use engine::builder::AppBuilder;
pub use engine::builder::WindowBuilder;
pub use engine::context::DrawContext;
pub use engine::context::LoadContext;
pub use engine::context::UpdateContext;
pub use engine::render::Draw;
pub use engine::render::Layer;
pub use engine::scene::Scene;
pub use engine::scene::SceneId;
pub use engine::time::Clock;
pub use engine::time::FpsCalculationStrategy;
pub use engine::time::PaceMode;
pub use engine::time::Time;
pub use engine::window::Window;

pub use sdl3::gpu::PresentMode;
pub use sdl3::render::Color;
pub use sdl3::window::FullscreenMode;

#[cfg(feature = "js")]
pub use js::JsScene;
#[cfg(feature = "js")]
pub use js::WindowBuilderExt;

pub mod input {
    pub use sdl3::events::Key;
    pub use sdl3::events::MouseButton;
}

pub use math;
pub use utils::Label;

pub use nostd::log;

pub mod prelude {
    pub use crate::*;
    pub use engine::assets::AssetServer;
    pub use engine::assets::Image;
    pub use engine::text::Font;
    pub use engine::text::Text;
    pub use engine::text::TextSpan;
    pub use engine::text::TextStyle;
    pub use input::*;
    pub use math::*;
    pub use nostd::collections::Handle;
    pub use nostd::collections::SlotMap;
    pub use nostd::log::*;
}
