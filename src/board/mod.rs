pub mod events;
mod piece;

use crate::cache::Cache;
use events::{BoardEvent, ElementState, MouseButton, MouseState};
use piece::{BoardPiece, BoardPiece::*, Piece::*};
use resvg::{tiny_skia, usvg};

pub struct Board {
    side_length: u32,
    white_color: [u8; 4],
    black_color: [u8; 4],
    glyph_cache: Cache<usvg::Tree>,
    picked_piece: Option<(BoardPiece, (usize, usize))>,
    mouse_state: MouseState,
    board_config: [[Option<BoardPiece>; 8]; 8],
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        Board {
            side_length: 720,
            white_color: [0xe3, 0xc1, 0x6f, 0xff],
            black_color: [0xb8, 0x8b, 0x4a, 0xff],
            glyph_cache: Cache::default(),
            mouse_state: MouseState::default(),
            picked_piece: None,
            board_config: [[Some(Black(Rook)), Some(Black(Knight)), Some(Black(Bishop)), Some(Black(Queen)), Some(Black(King)), Some(Black(Bishop)), Some(Black(Knight)), Some(Black(Rook))],
                           [Some(Black(Pawn)); 8],
                           [None; 8],
                           [None; 8],
                           [None; 8],
                           [None; 8],
                           [Some(White(Pawn)); 8],
                           [Some(White(Rook)), Some(White(Knight)), Some(White(Bishop)), Some(White(Queen)), Some(White(King)), Some(White(Bishop)), Some(White(Knight)), Some(White(Rook))]]
        }
    }
}

impl Board {
    pub fn get_side_length(&self) -> u32 {
        self.side_length
    }

    pub fn handle_event(&mut self, e: BoardEvent) {
        self.update_mouse_state(e);

        let check_side = self.get_check_side() as u32;

        let pos = self.mouse_state.get_pos();
        let x = (pos.1 as u32 / check_side) as usize;
        let y = (pos.0 as u32 / check_side) as usize;

        if self.mouse_state.get_is_left_pressed() {
            if let None = self.picked_piece {
                if let Some(p) = self.board_config[x][y] {
                    // log::debug!("Picking {:?} form check {}, {}", p, x, y);
                    self.board_config[x][y] = None;
                    self.picked_piece = Some((p, (x, y)));
                }
            }
        }

        if !self.mouse_state.get_is_left_pressed() {
            if let Some((p, (px, py))) = self.picked_piece {
                if let None = self.board_config[x][y] {
                    // log::debug!("Place {:?} at check {}, {}", p, x, y);
                    self.board_config[x][y] = Some(p);
                } else {
                    self.board_config[px][py] = Some(p);
                }
                self.picked_piece = None;
            }
        }

        if !self.mouse_state.get_is_cursor_in() {
            if let Some((p, (px, py))) = self.picked_piece {
                self.board_config[px][py] = Some(p);
            }
            self.picked_piece = None;
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        let mut pixmap = tiny_skia::Pixmap::new(self.side_length, self.side_length).unwrap();

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

        let check_side = self.get_check_side();
        let glyph_width = (check_side * 0.75) as u32;

        // Draw the checkboard and all the arrangement of pieces
        for x in 0..8 {
            for y in 0..8 {
                let rect = tiny_skia::Rect::from_xywh(
                    x as f32 * check_side,
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
                if let Some(p) = self.board_config[y][x] {
                    let tree = self.get_glyph_tree(&p);
                    let transform = tiny_skia::Transform::from_translate(
                        // TODO: Fix magic number
                        x as f32 * check_side + check_side / 8.0,
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
                    log::error!("Error Importing {}: {}", &glyph_path, e);
                    panic!();
                });
                let t = usvg::Tree::from_str(&str, &usvg::Options::default()).unwrap();
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
                self.mouse_state.update_pos(position);
            }
            BoardEvent::CursorLeft => {
                self.mouse_state.unset_cursor_in();
            }
        }
    }
}
