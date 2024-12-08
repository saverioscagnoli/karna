use crate::Context;

pub trait Load {
    fn load(&mut self, ctx: &mut Context);
}

pub trait Update {
    fn update(&mut self, ctx: &mut Context);
    fn fixed_update(&mut self, ctx: &mut Context);
}

pub trait Draw {
    fn draw(&mut self, ctx: &mut Context);
}
