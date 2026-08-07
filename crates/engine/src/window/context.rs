use std::marker::PhantomData;

pub struct UserContext {}

pub struct LoadContext<'a> {
    _a: PhantomData<&'a ()>,
}

pub struct UpdateContext<'a> {
    _a: PhantomData<&'a ()>,
}

pub struct DrawContext<'a> {
    _a: PhantomData<&'a ()>,
}

impl UserContext {
    pub fn for_load<'a>(&'a mut self) -> LoadContext<'a> {
        LoadContext { _a: PhantomData }
    }

    pub fn for_update<'a>(&'a mut self) -> UpdateContext<'a> {
        UpdateContext { _a: PhantomData }
    }

    pub fn for_draw<'a>(&'a mut self) -> DrawContext<'a> {
        DrawContext { _a: PhantomData }
    }
}
