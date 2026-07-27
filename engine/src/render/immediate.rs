use crate::render::packet::FramePacket;

pub struct Draw<'a> {
    pub(crate) packet: &'a mut FramePacket,
}
