pub mod events;

use crate::cache::Cache;
use crate::data::piece::BoardPiece;
use crate::data::BoardConfig;
use events::{BoardEvent, ElementState, MouseButton, MouseState};
use resvg::{tiny_skia, usvg};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Board {
    side_length: u32,
    ruler_offset: u32,
    white_color: [u8; 4],
    black_color: [u8; 4],
    ruler_color: [u8; 4],
    glyph_cache: Cache<usvg::Tree>,
    picked_piece: Option<(BoardPiece, (usize, usize))>,
    mouse_state: MouseState,
    board_config: Rc<RefCell<BoardConfig>>,
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        Board {
            side_length: 720,
            ruler_offset: 10,
            white_color: [0xe3, 0xc1, 0x6f, 0xff],
            black_color: [0xb8, 0x8b, 0x4a, 0xff],
            ruler_color: [0xff, 0xff, 0xff, 0xff],
            glyph_cache: Cache::default(),
            mouse_state: MouseState::default(),
            picked_piece: None,
            board_config: Rc::new(RefCell::new(BoardConfig::default())),
        }
    }
}

impl Board {
    pub fn get_config(&self) -> Rc<RefCell<BoardConfig>> {
        self.board_config.clone()
    }

    pub fn set_from_fen_str(&mut self, s: &str) {
        self.board_config = Rc::new(RefCell::new(BoardConfig::from_fen_str(s)));
    }

    pub fn handle_event(&mut self, e: BoardEvent) {
        self.update_mouse_state(e);

        let check_side = self.get_check_side() as u32;

        let pos = self.mouse_state.get_pos();
        let x = (pos.0 as u32 / check_side) as usize;
        let y = (pos.1 as u32 / check_side) as usize;

        if self.mouse_state.get_is_left_pressed() {
            if let None = self.picked_piece {
                if let Some(p) = self.board_config.borrow().get_at_xy(x, y) {
                    self.picked_piece = Some((p, (x, y)));
                }
            }
        }

        if !self.mouse_state.get_is_left_pressed() {
            if let Some((_, prev)) = self.picked_piece {
                let mut config = self.board_config.borrow_mut();
                if let None = config.get_at_xy(x, y) {
                    config.move_xy_to_xy(prev, (x, y));
                }
                self.picked_piece = None;
            }
        }

        if !self.mouse_state.get_is_cursor_in() {
            self.picked_piece = None;
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        let config = self.board_config.borrow().clone();
        let size = self.get_draw_area_side();
        let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();

        let mut white_paint = tiny_skia::Paint::default();
        white_paint.set_color_rgba8(
            self.white_color[0],
            self.white_color[1],
            self.white_color[2],
            self.white_color[3],
        );

        let mut black_paint = tiny_skia::Paint::default();
        black_paint.set_color_rgba8(
            self.black_color[0],
            self.black_color[1],
            self.black_color[2],
            self.black_color[3],
        );

        let mut ruler_paint = tiny_skia::Paint::default();
        ruler_paint.set_color_rgba8(
            self.ruler_color[0],
            self.ruler_color[1],
            self.ruler_color[2],
            self.ruler_color[3],
        );

        let check_side = self.get_check_side();
        let glyph_width = (check_side * 0.75) as u32;

        for i in 0..8 {
            let stroke = tiny_skia::Stroke::default();
            {
                // Y-axis
                let line = {
                    let mut pb = tiny_skia::PathBuilder::new();
                    pb.line_to(self.ruler_offset as f32, 0.0);
                    pb.finish().unwrap()
                };
                let transform =
                    tiny_skia::Transform::from_translate(0.0, (1 + i) as f32 * check_side as f32);
                pixmap.stroke_path(&line, &ruler_paint, &stroke, transform, None);
            }
            {
                // X-axis
                let line = {
                    let mut pb = tiny_skia::PathBuilder::new();
                    pb.line_to(0.0, self.ruler_offset as f32);
                    pb.finish().unwrap()
                };
                let transform = tiny_skia::Transform::from_translate(
                    self.ruler_offset as f32 + i as f32 * check_side as f32,
                    self.side_length as f32,
                );
                pixmap.stroke_path(&line, &ruler_paint, &stroke, transform, None);
            }
        }

        // Draw the checkboard and all the arrangement of pieces
        for y in 0..8 {
            for x in 0..8 {
                let rect = tiny_skia::Rect::from_xywh(
                    x as f32 * check_side + self.ruler_offset as f32,
                    y as f32 * check_side,
                    check_side,
                    check_side,
                )
                .unwrap();

                let paint = if x % 2 == 0 {
                    if y % 2 == 0 {
                        &white_paint
                    } else {
                        &black_paint
                    }
                } else {
                    if y % 2 == 0 {
                        &black_paint
                    } else {
                        &white_paint
                    }
                };

                pixmap.fill_rect(rect, paint, tiny_skia::Transform::identity(), None);

                if let Some((_, (picked_x, picked_y))) = self.picked_piece {
                    if x == picked_x && y == picked_y {
                        continue;
                    }
                }

                if let Some(p) = config.get_at_xy(x, y).to_owned() {
                    let tree = self.get_glyph_tree(&p);
                    let transform = tiny_skia::Transform::from_translate(
                        // TODO: Fix magic number
                        x as f32 * check_side + self.ruler_offset as f32 + check_side / 8.0,
                        y as f32 * check_side + check_side / 8.0,
                    );
                    let fit = usvg::FitTo::Width(glyph_width);
                    resvg::render(&tree, fit, transform, pixmap.as_mut());
                }
            }
        }

        // Draw the picked piece if any
        if let Some((p, _)) = self.picked_piece {
            let tree = self.get_glyph_tree(&p);

            let pos = self.mouse_state.get_pos();
            let y = pos.1;
            let x = pos.0;

            let transform = tiny_skia::Transform::from_translate(
                x as f32 - glyph_width as f32 / 2.0,
                y as f32 - glyph_width as f32 / 2.0,
            );
            let fit = usvg::FitTo::Width(glyph_width);
            resvg::render(&tree, fit, transform, pixmap.as_mut());
        }

        frame.copy_from_slice(pixmap.data());
    }

    pub fn get_draw_area_side(&self) -> u32 {
        self.side_length + self.ruler_offset
    }

    fn get_check_side(&self) -> f32 {
        (self.side_length / 8) as f32
    }

    fn get_glyph_tree(&mut self, p: &BoardPiece) -> usvg::Tree {
        let glyph_path = Board::get_glyph_path(p);
        match self.glyph_cache.get(&glyph_path) {
            Some(t) => t,
            None => {
                log::info!("Importing glyph {}", glyph_path);
                let str = std::fs::read_to_string(&glyph_path).unwrap_or_else(|e| {
                    log::error!("std::fs::read_to_string {}: {}", &glyph_path, e);
                    panic!();
                });
                let t = usvg::Tree::from_str(&str, &usvg::Options::default()).unwrap_or_else(|e| {
                    log::error!("usvg::Tree::from_str: {}", e);
                    panic!();
                });
                self.glyph_cache.put(&glyph_path, &t);
                t
            }
        }
    }

    fn get_glyph_path(p: &BoardPiece) -> String {
        let s = format!("assets/pieces/{}.svg", p);
        s.to_owned()
    }

    fn update_mouse_state(&mut self, e: BoardEvent) {
        match e {
            BoardEvent::MouseInput { button, state } => match button {
                MouseButton::Left => match state {
                    ElementState::Pressed => self.mouse_state.set_left_pressed(),
                    ElementState::Released => self.mouse_state.set_left_released(),
                },
                MouseButton::Right => match state {
                    ElementState::Pressed => self.mouse_state.set_right_pressed(),
                    ElementState::Released => self.mouse_state.set_right_released(),
                },
                _ => {}
            },
            BoardEvent::CursorMoved { position } => {
                self.mouse_state.set_cursor_in();
                if position.0 as u32 > self.ruler_offset && (position.1 as u32) < self.side_length {
                    let position = (position.0 - self.ruler_offset as usize, position.1);
                    self.mouse_state.update_pos(position);
                }
            }
            BoardEvent::CursorLeft => {
                self.mouse_state.unset_cursor_in();
            }
        }
    }
}
