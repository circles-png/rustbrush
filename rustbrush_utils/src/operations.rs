use crate::Brush;

/// Paints a brush stroke on the pixel buffer
/// Example usage:
/// ```rust
/// PaintOperation {
///     pixel_buffer: &mut self.layers[self.current_layer],
///     brush: &Brush::default(),
///     color: [255, 0, 0], // Red
///     buffer_width: self.buffer_width,
///     buffer_height: self.buffer_height,
///     cursor_position,
///     last_cursor_position,
///     is_eraser: false,
/// }
///     .process();
/// ```
pub struct PaintOperation<'a> {
    /// Pixel buffer to paint on. This should be a 1D (flat) array of RGBA values
    /// Example (4 pixels):
    /// ```rust
    /// [r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a]
    /// ```
    pub pixel_buffer: &'a mut Vec<u8>,
    pub pixel_buffer_width: u32,
    pub pixel_buffer_height: u32,
    pub brush: &'a Brush,
    pub color: [u8; 3],
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub is_eraser: bool,
}

impl PaintOperation<'_> {
    pub fn process(self) {
        let (x0, y0) = (self.last_cursor_position.0, self.last_cursor_position.1);
        let (x1, y1) = (self.cursor_position.0, self.cursor_position.1);

        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = dx.mul_add(dx, dy * dy).sqrt();

        let min_spacing = self.brush.radius() * self.brush.spacing();
        let steps = (distance / min_spacing).max(1.0) as i32;

        let stamp = self.brush.compute_stamp(self.color);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = dx.mul_add(t, x0);
            let y = dy.mul_add(t, y0);

            for pixel in &stamp.pixels {
                let px = (x + pixel.x as f32) as i32;
                let py = (y + pixel.y as f32) as i32;

                if px >= 0
                    && px < self.pixel_buffer_width as i32
                    && py >= 0
                    && py < self.pixel_buffer_height as i32
                {
                    let index = (py * self.pixel_buffer_width as i32 + px) as usize * 4;

                    let alpha = f32::from(pixel.color[3]) / 255.0;
                    if self.is_eraser {
                        let current_alpha = f32::from(self.pixel_buffer[index + 3]) / 255.0;
                        let erase_strength = alpha * self.brush.opacity();
                        self.pixel_buffer[index + 3] =
                            ((current_alpha * (1.0 - erase_strength)) * 255.0) as u8;
                    } else {
                        for c in 0..4 {
                            let current = f32::from(self.pixel_buffer[index + c]) / 255.0;
                            let new = f32::from(pixel.color[c]) / 255.0;
                            let result = new.mul_add(1.0 - current, current);
                            self.pixel_buffer[index + c] = (result * 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}
