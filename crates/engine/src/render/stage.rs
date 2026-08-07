use std::marker::PhantomData;

pub struct Stage {}

impl Stage {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct SceneView<'a> {
    _a: PhantomData<&'a ()>,
}

impl Stage {
    pub fn view<'a>(&mut self) -> SceneView<'a> {
        SceneView { _a: PhantomData }
    }
}
