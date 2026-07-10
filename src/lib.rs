pub use assets;
pub use engine::App;
pub use engine::AppBuilder;
pub use engine::ContextMut;
pub use engine::ContextRef;
pub use engine::Scene;
pub use engine::SceneManager;
pub use engine::WindowBuilder;
// Re-export imgui
pub use imgui;
pub use logging;
pub use math;
pub use utils::Handle;

pub mod render {
    pub use engine::Draw;
    pub use gpu::CircleVertex;
    pub use gpu::Vertex;
    pub use renderer::*;
}
