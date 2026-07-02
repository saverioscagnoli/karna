use math::Size;
use sokol::gfx as sg;
use sokol::glue as sglue;

use crate::camera::Camera;
use crate::camera::Projection;
use crate::layer::LayerId;
use crate::layer::RenderLayer;
use crate::pipeline::PipelineCache;
use crate::pipeline::PipelineDesc;

mod camera;
mod color;
mod immediate;
mod layer;
mod pipeline;
mod vertex;

pub use crate::color::Color;
pub use crate::immediate::handle::Draw;

pub struct Renderer {
    pipeline_cache: PipelineCache,
    clear_color: Color,
    layers: Vec<RenderLayer>,
    active_layer: LayerId,
    pub world: LayerId,
    pub ui: LayerId,
    pub debug: LayerId,
}

impl Renderer {
    pub fn new(view: Size<u32>) -> Self {
        let mut pipeline_cache = PipelineCache::new();

        for topo in [
            sg::PrimitiveType::Points,
            sg::PrimitiveType::Lines,
            sg::PrimitiveType::Triangles,
        ] {
            pipeline_cache.create_pipeline(PipelineDesc {
                shader: "immediate-2d",
                topology: topo,
                blend: true,
            });
        }

        let mut layers = Vec::new();

        let world = LayerId(layers.len());
        let world_camera = Camera::new(Projection::standard_2d(view));

        layers.push(RenderLayer::new(world_camera));

        let ui = LayerId(layers.len());
        let ui_camera = Camera::new(Projection::standard_2d(view));
        layers.push(RenderLayer::new(ui_camera));

        let debug = LayerId(layers.len());
        let debug_camera = Camera::new(Projection::standard_2d(view));
        layers.push(RenderLayer::new(debug_camera));

        Self {
            pipeline_cache,
            clear_color: Color::Black,
            layers,
            active_layer: world,
            world,
            ui,
            debug,
        }
    }

    pub fn resize(&mut self, view: Size<u32>) {
        for l in self.layers.iter_mut() {
            l.camera.update(view);
        }
    }

    pub fn layer(&self, id: &LayerId) -> &RenderLayer {
        &self.layers[id.0]
    }

    pub fn layer_mut(&mut self, id: &LayerId) -> &mut RenderLayer {
        &mut self.layers[id.0]
    }

    pub fn active_layer(&self) -> &RenderLayer {
        &self.layers[self.active_layer.0]
    }

    pub fn active_layer_mut(&mut self) -> &mut RenderLayer {
        &mut self.layers[self.active_layer.0]
    }

    pub fn present(&mut self, view: Size<u32>) {
        let mut action = sg::PassAction::new();
        action.colors[0] = sg::ColorAttachmentAction {
            load_action: sg::LoadAction::Clear,
            clear_value: self.clear_color.into(),
            ..Default::default()
        };

        sg::begin_pass(&sg::Pass {
            action,
            swapchain: sglue::swapchain(),
            ..Default::default()
        });

        for layer in &mut self.layers {
            layer.present(view, &self.pipeline_cache);
        }

        sg::end_pass();
        sg::commit();
    }
}
