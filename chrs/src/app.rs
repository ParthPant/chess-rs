use crate::board::{events::BoardEvent, Board};
use crate::ui::GuiFramework;
use chrs_lib::ai::{NegaMaxAI, AI};
use chrs_lib::data::{BoardConfig, Color, GameState, MoveList, Square};
use chrs_lib::generator::MoveGenerator;

use log;
use pixels::{Error, Pixels, SurfaceTexture};
use std::cell::RefCell;
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

pub struct App;

impl App {
    pub async fn run() {
        let event_loop = EventLoop::new();
        let builder = WindowBuilder::new();
        let window_size = LogicalSize::new(WIN_WIDTH, WIN_HEIGHT);
        let window = builder
            .with_maximized(true)
            .with_title("chess-rs")
            .with_inner_size(window_size)
            .build(&event_loop)
            .unwrap();
        let window = Rc::new(window);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            // Retrieve current width and height dimensions of browser client window
            let get_window_size = || {
                let client_window = web_sys::window().unwrap();
                LogicalSize::new(
                    client_window.inner_width().unwrap().as_f64().unwrap(),
                    client_window.inner_height().unwrap().as_f64().unwrap(),
                )
            };

            let window = Rc::clone(&window);

            // Initialize winit window with current dimensions of browser client
            window.set_inner_size(get_window_size());

            let client_window = web_sys::window().unwrap();

            // Attach winit canvas to body element
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");

            // Listen for resize event on browser client. Adjust winit window dimensions
            // on event trigger
            let closure =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let size = get_window_size();
                    window.set_inner_size(size)
                }) as Box<dyn FnMut(_)>);
            client_window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        let mut board = Board::default();
        let mut config = BoardConfig::default();
        // let config = BoardConfig::from_fen_str(
        //     "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        // );
        let generator = MoveGenerator::default();
        let mut ai = NegaMaxAI::default();

        let (mut pixels, mut framework) = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
            let board_size = board.get_draw_area_side();

            let pixels = Pixels::new_async(board_size, board_size, surface_texture)
                .await
                .expect("Pixels Error");
            let framework = GuiFramework::new(
                &event_loop,
                window_size.width,
                window_size.height,
                window.scale_factor() as f32,
                &pixels,
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
                                board.handle_event(board_event, &config);
                            }
                            CursorMoved { position, .. } => {
                                if let Ok(pos) = pixels.window_pos_to_pixel(position.into()) {
                                    let board_event = BoardEvent::CursorMoved { position: pos };
                                    board.handle_event(board_event, &config);
                                } else {
                                    let board_event = BoardEvent::CursorLeft;
                                    board.handle_event(board_event, &config);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MainEventsCleared => {
                    if config.get_state() == GameState::InPlay {
                        let turn = config.get_active_color();
                        if turn == Color::Black {
                            let ai_move = ai.get_best_move(&config, &generator);
                            if let Some(ai_move) = ai_move {
                                log::info!("AI response {:?}", ai.get_stats());
                                config.apply_move(ai_move);
                            } else {
                                log::info!("AI did not generate any move");
                            }
                        } else {
                            if let Some(user_move) = board.get_user_move() {
                                if moves.has_target_sq(user_move.to) {
                                    if !user_move.is_empty_prom() {
                                        config.apply_move(user_move);
                                        board.clear_user_move();
                                    }
                                }
                            }
                            let sq = board.get_picked_piece();
                            if sq != picked_sq {
                                picked_sq = sq;
                                if let Some(sq) = sq {
                                    let p = config.get_at_sq(sq).unwrap();
                                    moves = generator.gen_piece_moves(p, sq, &mut config, false);
                                }
                            }
                        }
                        generator.update_state(&mut config);
                    }
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Redraw here
                    board.draw(pixels.frame_mut(), &generator, &config, &moves);
                    // Prepare egui
                    framework.prepare(&window, &mut config, &mut ai);
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
