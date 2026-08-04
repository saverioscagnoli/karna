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

pub mod imgui {
    pub use engine::Condition;
    pub use engine::Ui;
    pub use engine::WindowFlags;
    pub use engine::WindowToken;
}

pub mod render {
    pub use engine::AlphaMode;
    pub use engine::Camera;
    pub use engine::Color;
    pub use engine::Draw;
    pub use engine::Fit;
    pub use engine::Geometry;
    pub use engine::Layer;
    pub use engine::MESH_SHADER;
    pub use engine::Material;
    pub use engine::MaterialDesc;
    pub use engine::Mesh;
    pub use engine::MeshVertex;
    pub use engine::PassState;
    pub use engine::Projection;
    pub use engine::Rasterize;
    pub use engine::SceneRef;
    pub use engine::cube;
    pub use utils::Handle;

    /// Pipeline state a material can select.
    pub mod pipeline {
        pub use gpu::BlendState;
        pub use gpu::CompareOp;
        pub use gpu::Cull;
        pub use gpu::DepthState;
    }
}

pub mod assets {
    pub use engine::Assets;
    pub use engine::Audio;
    pub use engine::AudioLength;
    pub use engine::Font;
    pub use engine::Image;
    pub use engine::Rasterize;
    pub use gpu::Filter;
    pub use utils::Handle;
}

pub mod gpu {
    pub use gpu::Filter;
    pub use gpu::PresentMode;
}
