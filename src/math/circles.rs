use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{BlendMode, Texture, TextureCreator},
    surface::SurfaceContext,
};

pub(crate) trait CircleOutline<'a> {
    fn arc(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a>;

    fn aa_arc(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a>;

    fn circle_outline(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a>;

    fn aa_circle_outline(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a>;
}

impl<'a> CircleOutline<'a> for Texture<'a> {
    fn arc(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer, pitch| {
                let mut t1 = (radius / 16) as i32;
                let mut t2;
                let mut x = radius as i32;
                let mut y = 0 as i32;

                let center = radius as i32;

                let start_angle = start_angle.to_radians();
                let end_angle = end_angle.to_radians();

                while x >= y {
                    let points = [
                        (x, y),
                        (y, x),
                        (-y, x),
                        (-x, y),
                        (-x, -y),
                        (-y, -x),
                        (y, -x),
                        (x, -y),
                    ];

                    for &(px, py) in points.iter() {
                        let dx = center + px;
                        let dy = center + py;

                        let angle = (py as f32).atan2(px as f32);
                        let normalized_angle = if angle < 0.0 {
                            angle + 2.0 * std::f32::consts::PI
                        } else {
                            angle
                        };

                        if normalized_angle >= start_angle && normalized_angle <= end_angle {
                            set_pixel(buffer, pitch, dx, dy, Color::WHITE);
                        }
                    }

                    y += 1;
                    t1 += y;
                    t2 = t1 - x;

                    if t2 >= 0 {
                        t1 = t2;
                        x -= 1;
                    }
                }
            })
            .unwrap();

        texture
    }

    fn aa_arc(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer, pitch| {
                let center = radius as f32;
                let start_angle = start_angle.to_radians();
                let end_angle = end_angle.to_radians();

                for y in 0..=d {
                    for x in 0..=d {
                        let dx = x as f32 - center;
                        let dy = y as f32 - center;
                        let angle = dy.atan2(dx);
                        let normalized_angle = if angle < 0.0 {
                            angle + 2.0 * std::f32::consts::PI
                        } else {
                            angle
                        };

                        let distance = (dx * dx + dy * dy).sqrt();
                        let coverage = 1.0 - (distance - radius as f32).abs().min(1.0);

                        if normalized_angle >= start_angle
                            && normalized_angle <= end_angle
                            && coverage > 0.0
                        {
                            let alpha = (255.0 * coverage) as u8;
                            let color = Color::RGBA(255, 255, 255, alpha);
                            set_pixel(buffer, pitch, x as i32, y as i32, color);
                        }
                    }
                }
            })
            .unwrap();

        texture
    }

    fn circle_outline(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let color = Color::WHITE;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let (cx, cy) = (radius as i32, radius as i32);
                let mut x = radius as i32;
                let mut y = 0;
                let mut err = 1 - x;

                while x >= y {
                    set_pixel(buffer, pitch, cx + x, cy + y, color);
                    set_pixel(buffer, pitch, cx + y, cy + x, color);
                    set_pixel(buffer, pitch, cx - y, cy + x, color);
                    set_pixel(buffer, pitch, cx - x, cy + y, color);
                    set_pixel(buffer, pitch, cx - x, cy - y, color);
                    set_pixel(buffer, pitch, cx - y, cy - x, color);
                    set_pixel(buffer, pitch, cx + y, cy - x, color);
                    set_pixel(buffer, pitch, cx + x, cy - y, color);

                    y += 1;
                    if err < 0 {
                        err += 2 * y + 1;
                    } else {
                        x -= 1;
                        err += 2 * (y - x + 1);
                    }
                }
            })
            .unwrap();

        texture
    }

    fn aa_circle_outline(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let color = Color::WHITE;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let radius = radius as f32;
                let center = radius;

                for y in 0..=d {
                    for x in 0..=d {
                        let dx = x as f32 - center;
                        let dy = y as f32 - center;
                        let distance = (dx * dx + dy * dy).sqrt();
                        let coverage = 1.0 - (distance - radius).abs().min(1.0);

                        if coverage > 0.0 {
                            let alpha = (color.a as f32 * coverage) as u8;
                            let blended_color = Color::RGBA(color.r, color.g, color.b, alpha);
                            set_pixel(buffer, pitch, x as i32, y as i32, blended_color);
                        }
                    }
                }
            })
            .unwrap();

        texture
    }
}

pub(crate) trait CircleFill<'a> {
    fn arc_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a>;

    fn aa_arc_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a>;

    fn circle_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a>;

    fn aa_circle_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a>;
}

impl<'a> CircleFill<'a> for Texture<'a> {
    fn arc_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer, pitch| {
                let center = radius as f32;
                let start_angle = start_angle.to_radians();
                let end_angle = end_angle.to_radians();

                for y in 0..=d {
                    for x in 0..=d {
                        let dx = x as f32 - center;
                        let dy = y as f32 - center;
                        let angle = dy.atan2(dx);
                        let normalized_angle = if angle < 0.0 {
                            angle + 2.0 * std::f32::consts::PI
                        } else {
                            angle
                        };

                        let distance = (dx * dx + dy * dy).sqrt();

                        if normalized_angle >= start_angle
                            && normalized_angle <= end_angle
                            && distance <= radius as f32
                        {
                            let alpha = 255;
                            let color = Color::RGBA(255, 255, 255, alpha);
                            set_pixel(buffer, pitch, x as i32, y as i32, color);
                        }
                    }
                }
            })
            .unwrap();

        texture
    }

    fn aa_arc_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
        start_angle: f32,
        end_angle: f32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer, pitch| {
                let center = radius as f32;
                let start_angle = start_angle.to_radians();
                let end_angle = end_angle.to_radians();

                for y in 0..=d {
                    for x in 0..=d {
                        let dx = x as f32 - center;
                        let dy = y as f32 - center;
                        let angle = dy.atan2(dx);
                        let normalized_angle = if angle < 0.0 {
                            angle + 2.0 * std::f32::consts::PI
                        } else {
                            angle
                        };

                        let distance = (dx * dx + dy * dy).sqrt();
                        let coverage = 1.0 - (distance - radius as f32).abs().min(1.0);

                        if normalized_angle >= start_angle && normalized_angle <= end_angle {
                            // Fill interior pixels with full alpha
                            if distance <= radius as f32 {
                                let alpha = 255; // Fully opaque
                                let color = Color::RGBA(255, 255, 255, alpha);
                                set_pixel(buffer, pitch, x as i32, y as i32, color);
                            }
                            // Anti-alias edge pixels
                            else if coverage > 0.0 {
                                let alpha = (255.0 * coverage) as u8;
                                let color = Color::RGBA(255, 255, 255, alpha);
                                set_pixel(buffer, pitch, x as i32, y as i32, color);
                            }
                        }
                    }
                }
            })
            .unwrap();

        texture
    }

    fn circle_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let color = Color::WHITE;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let (cx, cy) = (radius as i32, radius as i32);
                let mut x = radius as i32;
                let mut y = 0;
                let mut err = 1 - x;

                while x >= y {
                    set_pixel(buffer, pitch, cx + x, cy + y, color);
                    set_pixel(buffer, pitch, cx + y, cy + x, color);
                    set_pixel(buffer, pitch, cx - y, cy + x, color);
                    set_pixel(buffer, pitch, cx - x, cy + y, color);
                    set_pixel(buffer, pitch, cx - x, cy - y, color);
                    set_pixel(buffer, pitch, cx - y, cy - x, color);
                    set_pixel(buffer, pitch, cx + y, cy - x, color);
                    set_pixel(buffer, pitch, cx + x, cy - y, color);

                    y += 1;
                    if err < 0 {
                        err += 2 * y + 1;
                    } else {
                        x -= 1;
                        err += 2 * (y - x + 1);
                    }
                }

                let is_in_circle = |x: i32, y: i32| -> bool {
                    let dx = x as f32 - radius as f32;
                    let dy = y as f32 - radius as f32;
                    let distance = (dx * dx + dy * dy).sqrt();
                    distance <= radius as f32
                };

                for y in 0..=d {
                    for x in 0..=d {
                        if is_in_circle(x as i32, y as i32) {
                            set_pixel(buffer, pitch, x as i32, y as i32, color);
                        }
                    }
                }
            })
            .unwrap();

        texture
    }

    fn aa_circle_fill(
        texture_creator: &'a TextureCreator<SurfaceContext<'a>>,
        radius: u32,
    ) -> Texture<'a> {
        let d = radius * 2;
        let color = Color::WHITE;
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, d + 1, d + 1) // Ensure texture size is d + 1 by d + 1
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .with_lock(None, |buffer, pitch| {
                let (cx, cy) = (radius as i32, radius as i32);
                let samples = 4;

                for y in 0..d {
                    for x in 0..d {
                        let mut alpha_sum = 0.0;

                        for sy in 0..samples {
                            for sx in 0..samples {
                                let sub_x = x as f32 + (sx as f32 + 0.5) / samples as f32;
                                let sub_y = y as f32 + (sy as f32 + 0.5) / samples as f32;
                                let dx = sub_x - cx as f32;
                                let dy = sub_y - cy as f32;
                                let dist = (dx * dx + dy * dy).sqrt();

                                let alpha = if dist < radius as f32 {
                                    1.0
                                } else if dist < radius as f32 + 1.0 {
                                    1.0 - (dist - radius as f32)
                                } else {
                                    0.0
                                };
                                alpha_sum += alpha;
                            }
                        }

                        let alpha = alpha_sum / (samples * samples) as f32;

                        if alpha > 0.0 {
                            blend_pixel(buffer, pitch, x as i32, y as i32, color, alpha);
                        }
                    }
                }
            })
            .unwrap();

        texture
    }
}

fn set_pixel(buffer: &mut [u8], pitch: usize, x: i32, y: i32, color: Color) {
    if x < 0 || y < 0 || x >= pitch as i32 / 4 || y >= pitch as i32 / 4 {
        return;
    }
    let offset = (y as usize * pitch) + (x as usize * 4);
    buffer[offset] = color.r;
    buffer[offset + 1] = color.g;
    buffer[offset + 2] = color.b;
    buffer[offset + 3] = color.a;
}

fn blend_pixel(buffer: &mut [u8], pitch: usize, x: i32, y: i32, color: Color, alpha: f32) {
    if x < 0 || y < 0 || x >= pitch as i32 / 4 || y >= pitch as i32 / 4 {
        return;
    }

    let inv_alpha = 1.0 - alpha;

    let offset = (y as usize * pitch) + (x as usize * 4);
    buffer[offset] = (buffer[offset] as f32 * inv_alpha + color.r as f32 * alpha) as u8;
    buffer[offset + 1] = (buffer[offset + 1] as f32 * inv_alpha + color.g as f32 * alpha) as u8;
    buffer[offset + 2] = (buffer[offset + 2] as f32 * inv_alpha + color.b as f32 * alpha) as u8;
    buffer[offset + 3] = (buffer[offset + 3] as f32 * inv_alpha + color.a as f32 * alpha) as u8;
}
