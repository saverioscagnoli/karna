use sokol::gfx as sg;
use utils::FastHashMap;

use crate::immediate::immediate_2d as shd;
use crate::vertex::Vertex;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PipelineDesc {
    pub shader: &'static str,
    pub topology: sg::PrimitiveType, // your own enum mirroring sgA
    pub blend: bool,                 // simplify: alpha blend on/off
}

pub struct PipelineCache {
    pipelines: FastHashMap<PipelineDesc, sg::Pipeline>,
    shader: sg::Shader,
}

impl PipelineCache {
    pub fn new() -> Self {
        let shader = sg::make_shader(&&shd::shader_shader_desc(sg::query_backend()));
        Self {
            pipelines: FastHashMap::default(),
            shader,
        }
    }

    pub fn create_pipeline(&mut self, desc: PipelineDesc) {
        let mut pip = sg::PipelineDesc {
            shader: self.shader,
            layout: Vertex::layout(),
            index_type: sg::IndexType::Uint32,
            primitive_type: desc.topology,
            ..Default::default()
        };

        if desc.blend {
            pip.colors[0].blend = sg::BlendState {
                enabled: true,
                src_factor_rgb: sg::BlendFactor::SrcAlpha,
                dst_factor_rgb: sg::BlendFactor::OneMinusSrcAlpha,
                op_rgb: sg::BlendOp::Add,
                src_factor_alpha: sg::BlendFactor::One,
                dst_factor_alpha: sg::BlendFactor::OneMinusSrcAlpha,
                op_alpha: sg::BlendOp::Add,
            };
        }

        self.pipelines.insert(desc.clone(), sg::make_pipeline(&pip));
    }

    pub fn get_pipeline(&self, desc: &PipelineDesc) -> sg::Pipeline {
        self.pipelines[desc]
    }
}
