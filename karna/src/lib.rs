#![no_std]

pub use engine::App;
pub use engine::assets::Audio;
pub use engine::builder::AppBuilder;
pub use engine::builder::WindowBuilder;
pub use engine::context::DrawContext;
pub use engine::context::LoadContext;
pub use engine::context::UpdateContext;
pub use engine::mesh::Geometry;
pub use engine::mesh::Material;
pub use engine::mesh::Mesh;
pub use engine::mesh::Shading;
pub use engine::monitors::Monitor;
pub use engine::monitors::Monitors;
pub use engine::render::Camera;
pub use engine::render::Draw;
pub use engine::render::Layer;
pub use engine::render::Projection;
pub use engine::scene::Scene;
pub use engine::scene::SceneHandle;
pub use engine::scene::SceneId;
pub use engine::time::Clock;
pub use engine::time::FpsCalculationStrategy;
pub use engine::time::PaceMode;
pub use engine::time::Time;
pub use engine::window::Window;

pub use sdl3::gpu::Blend;
pub use sdl3::gpu::CullMode;
pub use sdl3::gpu::PresentMode;
pub use sdl3::monitor::MonitorId;
pub use sdl3::render::Color;
pub use sdl3::window::FullscreenMode;

#[cfg(feature = "js")]
pub use js::JsScene;
#[cfg(feature = "js")]
pub use js::WindowBuilderExt;

pub mod input {
    pub use engine::input::InputHandle;
    pub use engine::input::MAX_PLAYERS;
    pub use engine::input::PadHandle;
    pub use engine::input::PadView;
    pub use engine::input::STICK_DEADZONE;
    pub use engine::input::Stick;
    pub use engine::input::Trigger;
    pub use sdl3::events::Key;
    pub use sdl3::events::MouseButton;
    pub use sdl3::gamepad::GamepadAxis;
    pub use sdl3::gamepad::GamepadButton;
    pub use sdl3::gamepad::GamepadId;
    pub use sdl3::gamepad::GamepadType;
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
