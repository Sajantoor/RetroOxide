use std::sync::Arc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;
use winit::{application::ApplicationHandler, event_loop::ControlFlow};

use crate::emu::Context;
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
        let size = LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);

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
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(pixels) = self.pixels.as_mut() {
                    pixels.render().unwrap();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.get_next_frame();
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
        while self.context.is_running() {
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
                    break;
                }
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
