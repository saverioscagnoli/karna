use crate::render::packet::FramePacket;

pub struct SceneRef<'a> {
    pub(crate) packet: &'a mut FramePacket,
}
