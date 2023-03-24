use crate::ai::{NegaMaxAI, AI};
use crate::board::{events::BoardEvent, Board};
use crate::data::{BoardConfig, Color, MoveList, Square};
use crate::generator::MoveGenerator;
use crate::ui::GuiFramework;

use log;
use pixels::{Error, Pixels, SurfaceTexture};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

pub struct App;

impl App {
    pub fn run() -> Result<(), Error> {
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
        let config = Rc::new(RefCell::new(BoardConfig::default()));
        let generator = MoveGenerator::default();
        let ai = NegaMaxAI::default();

        let (mut pixels, mut framework) = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let board_size = board.get_draw_area_side();

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

        let mut moves = MoveList::new();
        let mut picked_sq: Option<Square> = None;
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
                                board.handle_event(board_event, &(*config).borrow());
                            }
                            CursorMoved { position, .. } => {
                                if let Ok(pos) = pixels.window_pos_to_pixel(position.into()) {
                                    let board_event = BoardEvent::CursorMoved { position: pos };
                                    board.handle_event(board_event, &(*config).borrow());
                                } else {
                                    let board_event = BoardEvent::CursorLeft;
                                    board.handle_event(board_event, &(*config).borrow());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MainEventsCleared => {
                    if config.borrow().get_active_color() == Color::Black {
                        let start = Instant::now();
                        let ai_move = ai.get_best_move(&config.borrow(), &generator);
                        let duration = start.elapsed();
                        log::info!("AI generated move in {:?}", duration);
                        if ai_move.is_some() {
                            config.borrow_mut().apply_move(ai_move.unwrap());
                        }
                    } else {
                        if let Some(user_move) = board.get_user_move() {
                            // log::debug!("User Move: {}", user_move);
                            if moves.has_target_sq(user_move.to) {
                                if !user_move.is_empty_prom() {
                                    config.borrow_mut().apply_move(user_move);
                                    board.clear_user_move();
                                }
                            }
                        }
                        let sq = board.get_picked_piece();
                        if sq != picked_sq {
                            picked_sq = sq;
                            if let Some(sq) = sq {
                                let p = config.borrow().get_at_sq(sq).unwrap();
                                // let start = Instant::now();
                                moves = generator.gen_piece_moves(p, sq, &(*config).borrow());
                                // let duration = start.elapsed();
                                // log::debug!("Generated {} moves in {:?}", moves.len(), duration)
                            }
                        }
                    }
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Redraw here
                    board.draw(
                        pixels.get_frame_mut(),
                        &generator,
                        &(*config).borrow(),
                        &moves,
                    );
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
}
