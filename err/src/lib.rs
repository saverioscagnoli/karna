#[derive(Debug, Clone, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to initialize SDL: {0}")]
    SdlInit(String),

    #[error("Failed to initialize video subsystem: {0}")]
    VideoInit(String),

    #[error("Failed to create SDL window: {0}")]
    WindowCreation(#[from] sdl2::video::WindowBuildError),

    #[error("Failed to create SDL event pump: {0}")]
    EventPumpCreation(String),

    #[error("Failed to create OpenGL context: {0}")]
    OpenGLContextCreation(String),

    #[error("Failed to load OpenGL function pointers: {0}")]
    OpenGLFunctionLoadError(String),

    #[error("There is no current scene set. Before running the app, you must set a scene.")]
    NoScene,

    #[error("Scene '{0}' not found")]
    SceneNotFound(String),

    #[error("Fatal error during scene loading: {0}")]
    SceneLoad(String),
}
