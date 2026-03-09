use macros::Get;
use macros::Set;
use renderer::Color;
use renderer::Renderer;

/// Immediate-mode drawing handle.
///
/// `Draw` is handed out once per frame and lets scenes push 2D geometry
/// (rectangles, triangles, lines, …) that will be rendered in a single
/// batched draw call when the frame is presented.
///
/// All coordinates are in **screen-space pixels** with `(0, 0)` at the
/// top-left corner of the window.
#[derive(Get, Set)]
pub struct Draw<'a> {
    pub(crate) renderer: &'a mut Renderer,

    #[get]
    #[set]
    color: Color,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            color: Color::White,
        }
    }

    // ------------------------------------------------------------------
    // Primitives
    // ------------------------------------------------------------------

    /// Draws a filled axis-aligned rectangle using the current color.
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color: [f32; 4] = self.color.into();
        self.renderer.immediate_mut().push_quad(x, y, w, h, color);
    }

    /// Draws a filled rectangle with an explicit color (does **not** change
    /// the current drawing color).
    pub fn fill_rect_colored(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.renderer
            .immediate_mut()
            .push_quad(x, y, w, h, color.into());
    }

    /// Draws a filled triangle using the current color.
    pub fn fill_triangle(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let color: [f32; 4] = self.color.into();
        self.renderer
            .immediate_mut()
            .push_triangle([x0, y0], [x1, y1], [x2, y2], color);
    }

    /// Draws a filled triangle with an explicit color.
    pub fn fill_triangle_colored(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) {
        self.renderer
            .immediate_mut()
            .push_triangle([x0, y0], [x1, y1], [x2, y2], color.into());
    }

    /// Draws a line between two points with the given thickness using the
    /// current color.
    ///
    /// The line is rendered as a rotated quad.
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, thickness: f32) {
        self.line_colored(x0, y0, x1, y1, thickness, self.color);
    }

    /// Draws a line between two points with the given thickness and explicit
    /// color.
    pub fn line_colored(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        thickness: f32,
        color: Color,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();

        if len < f32::EPSILON {
            return;
        }

        // Unit normal perpendicular to the line direction
        let nx = -dy / len * (thickness * 0.5);
        let ny = dx / len * (thickness * 0.5);

        let color: [f32; 4] = color.into();
        let immediate = self.renderer.immediate_mut();

        // The four corners of the line quad
        let p0 = [x0 + nx, y0 + ny];
        let p1 = [x0 - nx, y0 - ny];
        let p2 = [x1 - nx, y1 - ny];
        let p3 = [x1 + nx, y1 + ny];

        immediate.push_triangle(p0, p1, p2, color);
        immediate.push_triangle(p0, p2, p3, color);
    }
}
