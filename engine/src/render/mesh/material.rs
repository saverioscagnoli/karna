use utils::Handle;

use crate::MESH_SHADER;
use crate::assets::Image;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialUniforms {
    pub base_color: math::Vector4<f32>,
}

impl Default for MaterialUniforms {
    fn default() -> Self {
        Self {
            base_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub shader: gpu::ShaderRef,
    pub blend: gpu::BlendState,
    pub cull: Option<gpu::Cull>,
    pub texture: Option<Handle<Image>>,
    pub uniforms: Vec<u8>,
}

impl Default for Material {
    fn default() -> Self {
        Self::standard(math::Vector4::new(1.0, 1.0, 1.0, 1.0))
    }
}

impl Material {
    pub fn standard<C>(base_color: C) -> Self
    where
        C: Into<math::Vector4<f32>>,
    {
        let uniforms = MaterialUniforms {
            base_color: base_color.into(),
        };

        Self {
            shader: gpu::ShaderRef::Builtin(MESH_SHADER),
            blend: gpu::BlendState::None,
            cull: Some(gpu::Cull::Back),
            texture: None,
            uniforms: utils::as_u8_slice(std::slice::from_ref(&uniforms)).to_vec(),
        }
    }

    pub fn with_texture(mut self, texture: Handle<Image>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn set_base_color<C>(&mut self, base_color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        let uniforms = MaterialUniforms {
            base_color: base_color.into(),
        };

        self.uniforms = utils::as_u8_slice(std::slice::from_ref(&uniforms)).to_vec();
    }

    pub(crate) fn sort_key(&self) -> (usize, u8, u8, Option<Handle<Image>>) {
        let gpu::ShaderRef::Builtin(shader) = self.shader;

        let cull = match self.cull {
            None => 0,
            Some(gpu::Cull::Front) => 1,
            Some(gpu::Cull::Back) => 2,
        };

        (shader, self.blend as u8, cull, self.texture)
    }
}
