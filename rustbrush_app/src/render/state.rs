use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use rustbrush_utils::{operations::PaintOperation, Brush};
use tracing::error;
use winit::window::Window;

pub struct Canvas {
    layers: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    current_layer: usize,
    dirty: bool,
}

impl Canvas {
    pub fn paint(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        PaintOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            brush: &Brush::default().with_opacity(0.15),
            color: [255, 255, 255],
            pixel_buffer_width: self.width,
            pixel_buffer_height: self.height,
            cursor_position,
            last_cursor_position,
            is_eraser: false,
        }
        .process();
    }

    pub fn erase(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        PaintOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            brush: &Brush::default().with_opacity(0.15),
            color: [0, 0, 0], // doesn't even get used for eraser so doesn't matter
            pixel_buffer_width: self.width,
            pixel_buffer_height: self.height,
            cursor_position,
            last_cursor_position,
            is_eraser: true,
        }
        .process();
    }
}

pub struct RenderState<'window> {
    pixels: Pixels<'window>,
    pub canvas: Canvas,
}

impl RenderState<'_> {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let surface_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, window);

        // transparent layers by default
        let layer_size = (surface_size.width * surface_size.height * 4) as usize;
        let layer1 = vec![0u8; layer_size];
        let layer2 = vec![0u8; layer_size];

        let pixels = Pixels::new(width, height, surface_texture)
            .expect("Failed to create pixels. Cannot continue.");

        Self {
            pixels,
            canvas: Canvas {
                layers: vec![layer1, layer2],
                width,
                height,
                current_layer: 0,
                dirty: true,
            },
        }
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.canvas.dirty {
            return Ok(());
        }

        self.canvas.dirty = false;

        let frame = self.pixels.frame_mut();
        frame.fill(0); // clears the frame

        // merge layers into the frame
        for layer in &self.canvas.layers {
            for (i, chunk) in frame.chunks_mut(4).enumerate() {
                let layer_pixel = &layer[i * 4..(i + 1) * 4];

                let alpha = f32::from(layer_pixel[3]) / 255.0;
                for c in 0..3 {
                    let existing = f32::from(chunk[c]) / 255.0;
                    let new = f32::from(layer_pixel[c]) / 255.0;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    {
                        chunk[c] = (new.mul_add(alpha, existing * (1.0 - alpha)) * 255.0) as u8;
                    }
                }

                let existing_alpha = f32::from(chunk[3]) / 255.0;
                let new_alpha = alpha;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    chunk[3] = (existing_alpha.mul_add(1.0 - new_alpha, new_alpha) * 255.0) as u8;
                }
            }
        }

        self.pixels.render()?;
        Ok(())
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if let Err(e) = self.pixels.resize_surface(width, height) {
            error!("Failed to resize surface: {:?}", e);
        }
    }
}
