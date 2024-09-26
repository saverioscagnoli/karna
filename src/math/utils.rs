use sdl2::pixels::Color;

/// Helper function that sets the color of a pixel in a buffer.
/// Based on the target endiannes.
///
/// See https://doc.rust-lang.org/reference/conditional-compilation.html#target-endian
fn set_pixel_color(buffer: &mut [u8], i: usize, color: Color) {
    #[cfg(target_endian = "big")]
    {
        buffer[i] = color.r;
        buffer[i + 1] = color.g;
        buffer[i + 2] = color.b;
        buffer[i + 3] = color.a;
    }

    #[cfg(target_endian = "little")]
    {
        buffer[i] = color.a;
        buffer[i + 1] = color.b;
        buffer[i + 2] = color.g;
        buffer[i + 3] = color.r;
    }
}

/// Function that filla buffer with values pixels calculated using the midpoint circle algorithm.
/// The buffer is then used to create a texture for efficient circle rendering.
///
/// See https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
pub(crate) fn fill_midpoint_circle_buffer(buffer: &mut [u8], r: u32, color: Color) {
    let d = (r * 2 + 1) as i32;

    let mut t1 = (r / 16) as i32;
    let mut t2;
    let mut x = r as i32;
    let mut y = 0;

    let center = r as i32;

    while x >= y {
        let points = vec![
            (x, y),
            (y, x),
            (-y, x),
            (-x, y),
            (-x, -y),
            (-y, -x),
            (y, -x),
            (x, -y),
        ];

        for (dx, dy) in points {
            let nx = center + dx;
            let ny = center + dy;

            let i = (ny * d + nx) as usize * 4;
            set_pixel_color(buffer, i, color);
        }

        y += 1;
        t1 += y;
        t2 = t1 - x;

        if t2 >= 0 {
            t1 = t2;
            x -= 1;
        }
    }
}

/// Function that filla buffer with values pixels calculated using the midpoint circle algorithm.
/// This version fills the entire circle, not just the outline.
/// The buffer is then used to create a texture for efficient circle rendering.
///
/// See https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
pub(crate) fn fill_midpoint_circle_filled_buffer(buffer: &mut [u8], r: u32, color: Color) {
    let d = (r * 2 + 1) as i32;

    let mut t1 = (r / 16) as i32;
    let mut t2;
    let mut x = r as i32;
    let mut y = 0;

    let center = r as i32;

    while x >= y {
        for dy in -y..y {
            let x1 = center - x;
            let x2 = center + x;

            for dx in x1..x2 {
                let i = (dy + center) * d + dx;
                set_pixel_color(buffer, i as usize * 4, color);
            }
        }

        for dy in -x..x {
            let y1 = center - y;
            let y2 = center + y;

            for dx in y1..y2 {
                let nx = dx;
                let ny = center + dy;

                let i = ny * d + nx;
                set_pixel_color(buffer, i as usize * 4, color);
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
}
