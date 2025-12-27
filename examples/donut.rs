use std::time::Duration;

use karna::{AppBuilder, Scene, WindowBuilder, input::KeyCode, label, render::Text};
use renderer::{Color, Layer, TextHandle};
use utils::Timer;

struct Donut {
    text: TextHandle,
    debug_text: TextHandle,
    debug_timer: Timer,
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
        ctx.time.set_target_fps(120);

        ctx.assets.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            18,
        );

        let size = ctx.window.size();

        let mut text = Text::new(label!("jetbrains mono"), "");

        text.set_position_x(size.width as f32 / 2.0 - 400.0);
        text.set_position_y(size.height as f32 / 2.0 - 300.0);

        let mut debug_text = Text::new(label!("debug"), "");

        debug_text.set_position([10.0, 10.0, 0.0]);

        self.text = ctx.render.add_text(Layer::World, text);
        self.debug_text = ctx.render.add_text(Layer::World, debug_text);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let dt = ctx.time.delta();

        self.angle_a += 1.0 * dt;
        self.angle_b += 0.5 * dt;
        self.color_timer += 2.0 * dt;

        let frame_content = self.generate_frame();
        let rainbow_color = self.get_rainbow_color();

        {
            let text = ctx.render.get_text_mut(self.text);

            *text.content_mut() = frame_content.into();
            *text.color_mut() = rainbow_color.into();

            if ctx.input.key_pressed(&KeyCode::Space) {
                if text.font() == label!("debug") {
                    text.set_font(label!("jetbrains mono"));
                } else {
                    text.set_font(label!("debug"));
                }
            }
        }

        self.debug_timer.tick(dt);

        if self.debug_timer.is_finished() {
            let debug_text = ctx.render.get_text_mut(self.debug_text);
            *debug_text.content_mut() = format!("fps: {} dt: {}", ctx.time.fps(), ctx.time.delta());
            self.debug_timer.reset();
        }
    }

    fn fixed_update(&mut self, _ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {}
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("spinning donut")
                .with_size((1280, 720))
                .with_resizable(false)
                .with_initial_scene(Donut {
                    text: TextHandle::dummy(),
                    debug_text: TextHandle::dummy(),
                    debug_timer: Timer::new(Duration::from_millis(100)),
                    angle_a: 0.0,
                    angle_b: 0.0,
                    color_timer: 0.0,
                }),
        )
        .build()
        .run();
}
