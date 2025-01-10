#![warn(clippy::pedantic, clippy::nursery)]

use crate::render::state::RenderState;
use std::sync::Arc;
use tracing::error;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App<'window> {
    render_state: Option<RenderState<'window>>,
    window: Option<Arc<Window>>,
    cursor_position: (f32, f32),
    last_cursor_position: (f32, f32),
    holding_mouse_left: bool,
    holding_mouse_right: bool,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Brushy"))
                .expect("Failed to create window"),
        );

        self.render_state = Some(RenderState::new(Arc::clone(&window), 800, 600));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self
            .window
            .as_mut()
            .filter(|window| window.id() == window_id)
        else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize_surface(new_size.width, new_size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Some(render_state) = &mut self.render_state {
                    if self.holding_mouse_left {
                        render_state
                            .canvas
                            .paint(self.cursor_position, self.last_cursor_position);
                    } else if self.holding_mouse_right {
                        render_state
                            .canvas
                            .erase(self.cursor_position, self.last_cursor_position);
                    }
                    if let Err(e) = render_state.render() {
                        error!("Error rendering frame: {:?}", e);
                    }
                }
                window.request_redraw();
                self.last_cursor_position = self.cursor_position;
            }
            WindowEvent::CursorMoved { position, .. } => {
                #[allow(clippy::cast_possible_truncation)]
                {
                    self.cursor_position = (position.x as f32, position.y as f32);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let new_state = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                let holding_state = match button {
                    MouseButton::Left => Some(&mut self.holding_mouse_left),
                    MouseButton::Right => Some(&mut self.holding_mouse_right),
                    _ => None,
                };
                if let Some(holding_state) = holding_state {
                    *holding_state = new_state;
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput { .. } => {
                window.request_redraw();
            }
            _ => {}
        }
    }
}
