use math as m;
use std::rc::Rc;

use crate::events::WindowId;
use crate::window::FpsCalcStrategy;

#[derive(Debug)]
pub enum UserEvent {
    Window {
        id: WindowId,
        wevent: UserWindowEvent,
    },
    ChangeTargetTps(u32),
}

#[derive(Debug)]
pub enum UserWindowEvent {
    ChangeTitle(Rc<str>),
    ChangeSize(m::Size<u32>),
    ChangeResizable(bool),
    ChangeTargetFps(u32),
    ChangeFpsCalcStrategy(FpsCalcStrategy),
}
