use crate::window::Window;

pub struct ContextMut<'a> {
    window: &'a Window, // Cannot be mutated, no sense in making it &mut
}

pub struct ContextRef<'a> {
    window: &'a Window,
}
