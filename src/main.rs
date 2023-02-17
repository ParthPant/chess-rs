mod board;
mod data;
mod cache;
mod ui;

use crate::board::{events::BoardEvent, Board};
use crate::ui::GuiFramework;
use log;
use pixels::{Error, Pixels, SurfaceTexture};
use pretty_env_logger;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIN_WIDTH: u32 = 854;
const WIN_HEIGHT: u32 = 480;

fn main() -> Result<(), Error> {
    // std::env::set_var("RUST_LOG", "chrs=debug");
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new();
    let window_size = LogicalSize::new(WIN_WIDTH, WIN_HEIGHT);
    let window = builder
        .with_maximized(true)
        .with_title("chess-rs")
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let mut board = Board::default();
    let config = board.get_config();

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let board_size = board.get_side_length();

        // TODO: use new_async for web
        let pixels = Pixels::new(board_size, board_size, surface_texture)?;
        let framework = GuiFramework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            window.scale_factor() as f32,
            &pixels,
            config.clone(),
        );

        (pixels, framework)
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                if !framework.handle_event(&event) {
                    use winit::event::WindowEvent::*;

                    match event {
                        CloseRequested => {
                            log::info!("The close Button was pressed.");
                            control_flow.set_exit();
                        }
                        Resized(size) => {
                            if let Err(err) = pixels.resize_surface(size.width, size.height) {
                                log::error!("Pixels failed to resize error: {}", err);
                                control_flow.set_exit();
                            }
                            framework.resize(size.width, size.height);
                        }
                        ScaleFactorChanged {
                            scale_factor,
                            new_inner_size: _,
                        } => {
                            framework.scale_factor(scale_factor);
                        }
                        MouseInput { state, button, .. } => {
                            let board_event = BoardEvent::MouseInput { state, button };
                            board.handle_event(board_event);
                        }
                        CursorMoved { position, .. } => {
                            if let Ok(pos) = pixels.window_pos_to_pixel(position.into()) {
                                let board_event = BoardEvent::CursorMoved { position: pos };
                                board.handle_event(board_event);
                            } else {
                                let board_event = BoardEvent::CursorLeft;
                                board.handle_event(board_event);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Redraw here
                board.draw(pixels.get_frame_mut());
                // Prepare egui
                framework.prepare(&window);
                // Render everything together
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the board texture
                    context.scaling_renderer.render(encoder, render_target);
                    // Render egui
                    framework.render(encoder, render_target, context);
                    Ok(())
                });

                if let Err(err) = render_result {
                    log::error!("pixels.render_with failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => (),
        }
    });
}
