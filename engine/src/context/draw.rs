use renderer::Renderer;

pub struct Draw<'a> {
    pub(crate) renderer: &'a mut Renderer,
}
