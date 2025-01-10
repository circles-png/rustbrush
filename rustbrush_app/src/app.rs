use tracing::error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use crate::render::state::RenderState;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    render_state: Option<RenderState<'static>>,
    cursor_position: (f32, f32),
    last_cursor_position: (f32, f32),
    holding_mouse_left: bool,
    holding_mouse_right: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Brushy")
        ).expect("Failed to create window");

        // Initialize WGPU
        // SAFETY: We ensure the Window outlives the Surface by keeping them together
        let wgpu_state = unsafe {
            let window_ref: &'static Window = std::mem::transmute(&window);
            pollster::block_on(RenderState::new(window_ref, 800, 600))
        };

        self.render_state = Some(wgpu_state);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = match &self.window {
            Some(w) if w.id() == window_id => w,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(new_size) => {
                if let Some(wgpu_state) = &mut self.render_state {
                    wgpu_state.resize_surface(new_size.width, new_size.height);
                }
                window.request_redraw();
            },
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_state) = &mut self.render_state {
                    if self.holding_mouse_left {
                        wgpu_state.paint(self.cursor_position, self.last_cursor_position);
                    } else if self.holding_mouse_right {
                        wgpu_state.erase(self.cursor_position, self.last_cursor_position);
                    }
                    match wgpu_state.render() {
                        Ok(_) => {},
                        Err(e) => error!("Error rendering frame: {:?}", e),
                    }
                }
                window.request_redraw();
                self.last_cursor_position = self.cursor_position;
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        if button == winit::event::MouseButton::Left {
                            self.holding_mouse_left = true;
                        } else if button == winit::event::MouseButton::Right {
                            self.holding_mouse_right = true;
                        }
                    },
                    winit::event::ElementState::Released => {
                        if button == winit::event::MouseButton::Left {
                            self.holding_mouse_left = false;
                        } else if button == winit::event::MouseButton::Right {
                            self.holding_mouse_right = false;
                        }
                    },
                }
                window.request_redraw();
            },
            WindowEvent::KeyboardInput {
                ..
            } => {
                window.request_redraw();
            },
            _ => {},
        }
    }
}
