use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::Key;
use winit::window::Window;
use winit::{application::ApplicationHandler, event_loop::ControlFlow};

use crate::emu::Context;
use crate::joypad::joypad::Button;
use crate::ppu::lcd::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug)]
pub struct UI<'a> {
    app: App<'a>,
}

#[derive(Debug)]
struct App<'a> {
    context: Context,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(SCREEN_WIDTH as u16 * 4, SCREEN_HEIGHT as u16 * 4);

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(size)
                    .with_min_inner_size(size),
            )
            .unwrap();

        let window_size = window.inner_size();
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        let pixels =
            Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap();

        self.pixels = Some(pixels);
        self.context.start();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.context.stop();
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    pixels.render().unwrap();
                }
            }

            WindowEvent::Focused(is_focused) => {
                if !is_focused && self.context.is_running() {
                    self.context.pause();
                } else if is_focused && !self.context.is_running() {
                    self.context.start();
                }
            }

            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => match event.logical_key {
                // Key::Named(named_key) => match named_key {
                //     NamedKey::Escape => {
                //         self.context.toggle_pause();
                //     }
                //     _ => {}
                // },
                Key::Character(str) => {
                    let button = key_to_button(&str);
                    if let Some(button) = button {
                        self.context.press_button(button, event.state.is_pressed());
                    }
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.get_next_frame();
    }
}

fn key_to_button(key: &str) -> Option<Button> {
    match key {
        "w" => Some(Button::Up),
        "a" => Some(Button::Left),
        "s" => Some(Button::Down),
        "d" => Some(Button::Right),
        "j" => Some(Button::A),
        "k" => Some(Button::B),
        "u" => Some(Button::Select),
        "i" => Some(Button::Start),
        _ => None,
    }
}

impl<'a> App<'a> {
    pub fn new(context: Context) -> Self {
        App {
            context: context,
            window: None,
            pixels: None,
        }
    }

    fn get_next_frame(&mut self) {
        let buffer = self.context.step();
        if let Some(buffer) = buffer {
            let frame = self.pixels.as_mut().unwrap().frame_mut();
            // update frame
            // if there's a change in the frame, only then render it.
            // Otherwise, no point rendering.
            // Check if there's a change in the frame
            if frame != buffer {
                frame.copy_from_slice(&buffer);
                self.window.as_ref().unwrap().request_redraw();
            }
        }
    }
}

impl<'a> UI<'a> {
    pub fn new(context: Context) -> Self {
        UI {
            app: App::new(context),
        }
    }

    pub fn start(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let _ = event_loop.run_app(&mut self.app);
    }
}
