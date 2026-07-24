use crate::render::target::RenderTarget;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub position: math::Vector2<u32>,
    pub size: math::Size<u32>,
}

impl Viewport {
    pub fn new<P, S>(position: P, size: S) -> Self
    where
        P: Into<math::Vector2<u32>>,
        S: Into<math::Size<u32>>,
    {
        let position: math::Vector2<u32> = position.into();
        let size: math::Size<u32> = size.into();

        Self { position, size }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct View {
    pub target: RenderTarget,
    pub viewport: Viewport,
}
