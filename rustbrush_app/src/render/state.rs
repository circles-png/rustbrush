use rustbrush_utils::{Brush, operations::PaintOperation};
use tracing::error;
use winit::window::Window;
use pixels::{Pixels, SurfaceTexture};

pub struct RenderState<'window> {
    pixels: Pixels<'window>,
    _window: &'window Window,
    buffer_width: u32,
    buffer_height: u32,
    layers: Vec<Vec<u8>>,
    current_layer: usize,
    dirty: bool,
}

impl<'window> RenderState<'window> {
    pub async fn new(window: &'window Window, width: u32, height: u32) -> Self {
        let surface_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, window);

        // transparent layers by default
        let layer_size = (surface_size.width * surface_size.height * 4) as usize;
        let layer1 = vec![0u8; layer_size];
        let layer2 = vec![0u8; layer_size];

        let pixels = Pixels::new(
            width,
            height,
            surface_texture,
        )
            .expect("Failed to create pixels. Cannot continue.");

        Self {
            pixels,
            _window: window,
            buffer_width: width,
            buffer_height: height,
            layers: vec![layer1, layer2],
            current_layer: 0,
            dirty: true,
        }
    }

    pub fn paint(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        PaintOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            brush: &Brush::default().with_opacity(0.15),
            color: [255, 255, 255],
            pixel_buffer_width: self.buffer_width,
            pixel_buffer_height: self.buffer_height,
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
            brush: &Brush::default(),
            color: [0, 0, 0], // doesn't even get used for eraser so doesn't matter
            pixel_buffer_width: self.buffer_width,
            pixel_buffer_height: self.buffer_height,
            cursor_position,
            last_cursor_position,
            is_eraser: true,
        }
            .process();
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        if !self.dirty {
            return Ok(());
        }

        self.dirty = false;

        let frame = self.pixels.frame_mut();
        frame.fill(0); // clears the frame

        // merge layers into the frame
        for layer in &self.layers {
            for (i, chunk) in frame.chunks_mut(4).enumerate() {
                let layer_pixel = &layer[i * 4..(i + 1) * 4];
                
                let alpha = layer_pixel[3] as f32 / 255.0;
                for c in 0..3 {
                    let existing = chunk[c] as f32 / 255.0;
                    let new = layer_pixel[c] as f32 / 255.0;
                    chunk[c] = ((new * alpha + existing * (1.0 - alpha)) * 255.0) as u8;
                }
                
                let existing_alpha = chunk[3] as f32 / 255.0;
                let new_alpha = alpha;
                chunk[3] = ((new_alpha + existing_alpha * (1.0 - new_alpha)) * 255.0) as u8;
            }
        }
        
        self.pixels.render()?;
        Ok(())
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Err(e) = self.pixels.resize_surface(width, height) {
                error!("Failed to resize surface: {:?}", e);
            }
        }
    }
}