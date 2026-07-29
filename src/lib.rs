pub use logging;
pub use math;

pub use engine::App;
pub use engine::AppBuilder;
pub use engine::ContextRef;
pub use engine::DrawContext;
pub use engine::Scene;
pub use engine::Time;
pub use engine::WindowBuilder;
pub use engine::WindowHandle;

pub mod input {
    pub use engine::Input;
    pub use engine::Key;
    pub use engine::MouseButton;
}

pub mod render {
    pub use engine::Color;
    pub use engine::Draw;
    pub use engine::SceneRef;
}

pub mod assets {
    pub use engine::Assets;
    pub use engine::Audio;
    pub use engine::AudioLength;
    pub use engine::Image;
    pub use utils::Handle;
}

pub mod gpu {
    pub use gpu::PresentMode;
}
