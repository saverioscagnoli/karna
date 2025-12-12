use karna::{AppBuilder, Scene, WindowBuilder, label, render::Text};
use math::Vector2;
use renderer::Color;

struct Donut {
    text: Text,
    angle_a: f32,
    angle_b: f32,
    color_timer: f32,
}

impl Donut {
    fn get_rainbow_color(&self) -> Color {
        let r = self.color_timer.sin() * 127.0 + 128.0;
        let g = (self.color_timer + 2.0).sin() * 127.0 + 128.0;
        let b = (self.color_timer + 4.0).sin() * 127.0 + 128.0;

        Color::rgb(r / 255.0, g / 255.0, b / 255.0)
    }

    fn generate_frame(&self) -> String {
        let width = 80;
        let height = 22;
        let mut buffer = vec![' '; width * height];
        let mut z_buffer = vec![0.0f32; width * height];

        let theta_spacing = 0.07;
        let phi_spacing = 0.02;

        let r1 = 1.0; // Tube radius
        let r2 = 2.0; // Donut radius
        let k2 = 5.0; // Distance from camera
        let k1 = 15.0;
        let x_scale = 2.0;
        let y_scale = 1.0;

        let cos_a = self.angle_a.cos();
        let sin_a = self.angle_a.sin();
        let cos_b = self.angle_b.cos();
        let sin_b = self.angle_b.sin();

        let mut theta = 0.0f32;

        while theta < 2.0 * std::f32::consts::PI {
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            let mut phi = 0.0f32;
            while phi < 2.0 * std::f32::consts::PI {
                let cos_phi = phi.cos();
                let sin_phi = phi.sin();

                let circle_x = r2 + r1 * cos_theta;
                let circle_y = r1 * sin_theta;

                let x = circle_x * (cos_b * cos_phi + sin_a * sin_b * sin_phi)
                    - circle_y * cos_a * sin_b;
                let y = circle_x * (sin_b * cos_phi - sin_a * cos_b * sin_phi)
                    + circle_y * cos_a * cos_b;
                let z = k2 + cos_a * circle_x * sin_phi + circle_y * sin_a;
                let ooz = 1.0 / z;

                let xp = (width as f32 / 2.0 + (k1 * x_scale) * ooz * x) as i32;
                let yp = (height as f32 / 2.0 - (k1 * y_scale) * ooz * y) as i32;

                let l =
                    cos_phi * cos_theta * sin_b - cos_a * cos_theta * sin_phi - sin_a * sin_theta
                        + cos_b * (cos_a * sin_theta - cos_theta * sin_a * sin_phi);

                if l > 0.0 {
                    if xp >= 0 && xp < width as i32 && yp >= 0 && yp < height as i32 {
                        let idx = (xp + yp * width as i32) as usize;
                        if ooz > z_buffer[idx] {
                            z_buffer[idx] = ooz;
                            let luminance_chars = ".,-~:;=!*#$@";
                            let luminance_index = (l * 8.0) as usize;
                            let char_idx = luminance_index.clamp(0, 11);
                            buffer[idx] = luminance_chars.chars().nth(char_idx).unwrap();
                        }
                    }
                }
                phi += phi_spacing;
            }
            theta += theta_spacing;
        }

        let mut output = String::new();

        for y in 0..height {
            for x in 0..width {
                output.push(buffer[x + y * width]);
            }
            output.push('\n');
        }
        output
    }
}

impl Scene for Donut {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
        ctx.time.uncap_fps();

        ctx.render.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf"),
            18,
        );

        let size = ctx.window.size();

        self.text.set_position(Vector2::new(
            size.width as f32 / 2.0 - 400.0,
            size.height as f32 / 2.0 - 300.0,
        ));
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let dt = ctx.time.delta();

        self.angle_a += 1.0 * dt;
        self.angle_b += 0.5 * dt;

        self.color_timer += 2.0 * dt;

        self.text.content = self.generate_frame().into();
        self.text.color = self.get_rainbow_color().into();
    }

    fn fixed_update(&mut self, _ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_text(&self.text);

        ctx.render.draw_debug_text(
            format!("fps: {} dt: {}", ctx.time.fps(), ctx.time.delta()),
            [10.0, 10.0],
        );
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Spinning Donut")
                .with_size((1280, 720))
                .with_initial_scene(Box::new(Donut {
                    text: Text::new(label!("jetbrains mono"), ""),
                    angle_a: 0.0,
                    angle_b: 0.0,
                    color_timer: 0.0,
                })),
        )
        .build()
        .run();
}
