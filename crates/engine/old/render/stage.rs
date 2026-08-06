use crate::render::color::Color;

pub struct Stage {
    pub clear_color: Color,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            clear_color: Color::Black,
        }
    }

    pub fn view<'a>(&'a mut self) -> SceneView<'a> {
        SceneView { stage: self }
    }
}

pub struct SceneView<'a> {
    pub(crate) stage: &'a mut Stage,
}

impl<'a> SceneView<'a> {
    pub fn clear_color(&self) -> Color {
        self.stage.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.stage.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.stage.clear_color = color.into();
    }
}
