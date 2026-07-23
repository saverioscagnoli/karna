use utils::WindowId;

#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    Window(WindowId),
}
