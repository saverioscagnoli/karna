use crate::retained::{GlyphGpu, Text};
use globals::consts;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use utils::Handle;

pub struct TextBatch {
    pub handles: Vec<Handle<Text>>,
    pub instance_buffer: GpuBuffer<GlyphGpu>,
    pub needs_rebuild: bool,
    pub total_glyphs: usize,
}

impl TextBatch {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            instance_buffer: GpuBufferBuilder::new()
                .label("Text Instance Buffer")
                .vertex()
                .copy_dst()
                .capacity(consts::TEXT_INSTANCE_BASE_CAPACITY)
                .build(),
            needs_rebuild: false,
            total_glyphs: 0,
        }
    }
}
