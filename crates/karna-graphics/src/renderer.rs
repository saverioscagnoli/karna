use crate::{
    batchers::{point::PointBatcher, triangle::TriangleBatcher},
    color::Color,
};
use karna_math::{matrix::Mat4, size::Size, vector::Vec2};
use karna_opengl::{
    buffers::VertexArray,
    clear, clear_color, get_gl_error,
    shaders::{Program, Shader, ShaderKind, Uniform},
    vertex_attrib_binding, vertex_attrib_format, DataType, Mask,
};

pub struct Renderer {
    draw_color: Color,
    vao: VertexArray,
    program: Program,

    /// Batchers
    points: PointBatcher,
    triangles: TriangleBatcher,
}

impl Renderer {
    const NEAR: f32 = -1.0;
    const FAR: f32 = 100.0;

    /// Marked with _ because pub(workspace) doesnt exist. :(
    /// DO NOT USE, this is for internal use only.
    pub fn _new(width: u32, height: u32) -> Self {
        let vao = VertexArray::new();

        vao.bind();

        let float = std::mem::size_of::<f32>();

        // First 3 floats are the position
        vertex_attrib_format(0, 3, DataType::Float, false, 0);
        vertex_attrib_binding(0, 0);
        // Next 4 floats are the color
        vertex_attrib_format(1, 4, DataType::Float, false, 3 * float);
        vertex_attrib_binding(1, 0);
        // Next 2 floats are the texture coordinates
        vertex_attrib_format(2, 2, DataType::Float, false, 7 * float);
        vertex_attrib_binding(2, 0);

        let vert_shader = Shader::new(
            ShaderKind::Vertex,
            include_str!("../../../assets/shader.vert"),
        );

        let frag_shader = Shader::new(
            ShaderKind::Fragment,
            include_str!("../../../assets/shader.frag"),
        );

        let mut program = Program::new(vert_shader, frag_shader);
        let projection = Mat4::ortho(0.0, width as f32, height as f32, 0.0, Self::NEAR, Self::FAR);

        program.enable();
        program.set_uniform("uProjection", Uniform::Mat4(projection));

        Self {
            draw_color: Color::WHITE,
            vao,

            program,
            points: PointBatcher::new(),
            triangles: TriangleBatcher::new(),
        }
    }

    /// Returns the color that will be used when drawing subsequent shapes.
    #[inline]
    pub fn color(&self) -> Color {
        self.draw_color
    }

    /// Sets the color to be used when drawing subsequent shapes.
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.draw_color = color;
    }

    /// Sets the background color of the window.
    #[inline]
    pub fn clear_background(&self, color: Color) {
        clear_color(color.r, color.g, color.b, color.a);
    }

    /// Draws a single pixel at the specified coordinates.
    #[inline]
    pub fn draw_pixel(&mut self, x: f32, y: f32) {
        self.points.draw_pixel(x, y, 0.0, self.draw_color.into());
    }

    /// Draws a single pixel at the specified coordinates.
    ///
    /// This differs from `draw_pixel` because it takes a vector for more ergonomic use.
    #[inline]
    pub fn draw_pixel_v<P: Into<Vec2>>(&mut self, pos: P) {
        let pos: Vec2 = pos.into();

        self.points
            .draw_pixel(pos.x, pos.y, 0.0, self.draw_color.into());
    }

    /// Draws a solid-color rectangle with the top-left corner at (x, y) and the specified width and height.
    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.triangles
            .fill_rect(x, y, 0.0, w, h, self.draw_color.into());
    }

    /// Draws a solid-color rectangle with the top-left corner at `pos` and the specified size.
    /// This differs from `fill_rect` because it takes a vector and size for more ergonomic use.
    ///
    /// # Example
    /// ```no_run
    /// // Draw a rectangle at (10, 10) with a size of (50, 50)
    /// ctx.render.fill_rect_v([10, 10], (50.0, 50.0));
    /// ```
    #[inline]
    pub fn fill_rect_v<P: Into<Vec2>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size<f32> = size.into();

        self.triangles.fill_rect(
            pos.x,
            pos.y,
            0.0,
            size.width,
            size.height,
            self.draw_color.into(),
        );
    }

    /// Marked with _ because pub(workspace) doesnt exist. :(
    /// DO NOT USE, this is for internal use only.
    #[inline]
    pub fn _present(&mut self) {
        clear(Mask::ColorBufferBit | Mask::DepthBufferBit);

        // Chceck for OpenGL errors
        if let Some(error) = get_gl_error() {
            panic!("{}", error);
        }

        self.vao.bind();

        self.program.enable();
        self.vao.bind();

        self.points.flush();
        self.triangles.flush();
    }
}
