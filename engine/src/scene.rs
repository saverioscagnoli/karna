use crate::Context;

pub enum LoadControlFlow {
    Ignore,
    Throw,
}

pub trait Scene {
    fn load(&mut self, context: &mut Context) -> Result<(), (LoadControlFlow, String)>;
    fn update(&mut self, context: &mut Context);
    fn render(&mut self, context: &mut Context);
}
