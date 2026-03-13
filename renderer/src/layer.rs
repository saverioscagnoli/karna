use crate::Camera;
use crate::ImmediateRenderer;

pub struct RenderLayer {
    camera: Camera,
    immediate: ImmediateRenderer,
}
