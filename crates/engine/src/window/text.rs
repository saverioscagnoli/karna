use crate::TextSystem;
use crate::assets::TextureAtlas;
use crate::text::Text;
use crate::text::TextSpan;
use crate::text::TextStyle;

pub struct TextHandle<'a> {
    pub(crate) text: &'a mut TextSystem,
    pub(crate) atlas: &'a mut TextureAtlas,
}

impl TextHandle<'_> {
    pub fn layout<T>(&mut self, text: T, style: &TextStyle) -> Text
    where
        T: AsRef<str>,
    {
        self.text.layout(text, style, self.atlas)
    }

    pub fn layout_rich(&mut self, spans: &[TextSpan], style: &TextStyle) -> Text {
        self.text.layout_rich(spans, style, self.atlas)
    }
}
