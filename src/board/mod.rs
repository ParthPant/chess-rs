pub mod events;

use crate::cache::Cache;
use crate::data::{bitboard::BitBoard, piece::BoardPiece, BoardConfig, Square};
use events::{BoardEvent, ElementState, MouseButton, MouseState};
use fontdue;
use resvg::{tiny_skia, usvg};

pub struct Board {
    side_length: u32,
    ruler_offset: u32,
    white_color: [u8; 4],
    black_color: [u8; 4],
    ruler_color: [u8; 4],
    highlight_color: [u8; 4],
    font: fontdue::Font,
    glyph_cache: Cache<usvg::Tree>,
    raster_cache: Cache<tiny_skia::Pixmap>,
    picked_piece: Option<Square>,
    mouse_state: MouseState,
    // TODO: Make a better representation for Moves
    user_move: Option<(Square, Square)>,
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        let font_src = Self::get_font_src();
        let font = fontdue::Font::from_bytes(font_src, fontdue::FontSettings::default()).unwrap();
        Board {
            side_length: 720,
            ruler_offset: 20,
            white_color: [0xe3, 0xc1, 0x6f, 0xff],
            black_color: [0xb8, 0x8b, 0x4a, 0xff],
            ruler_color: [0xff, 0xff, 0xff, 0xff],
            highlight_color: [0x3f, 0x7a, 0xd9, 0x40],
            font,
            glyph_cache: Cache::default(),
            raster_cache: Cache::default(),
            mouse_state: MouseState::default(),
            picked_piece: None,
            user_move: None,
        }
    }
}

impl Board {
    pub fn get_user_move(&mut self) -> Option<(Square, Square)> {
        let umove = self.user_move;
        self.user_move = None;
        umove
    }

    pub fn get_picked_piece(&self) -> Option<Square> {
        self.picked_piece
    }

    pub fn handle_event(&mut self, e: BoardEvent, config: &BoardConfig) {
        self.update_mouse_state(e);

        let sq = self.get_sq_from_pointer();

        if self.mouse_state.get_is_left_pressed() {
            if let None = self.picked_piece {
                if let Some(_) = config.get_at_sq(sq) {
                    self.picked_piece = Some(sq);
                }
            }
        }

        if !self.mouse_state.get_is_left_pressed() {
            if let Some(prev) = self.picked_piece {
                self.user_move = Some((prev, sq));
                self.picked_piece = None;
            }
        }

        if !self.mouse_state.get_is_cursor_in() {
            self.picked_piece = None;
        }
    }

    pub fn draw(&mut self, frame: &mut [u8], config: &BoardConfig, moves: &BitBoard) {
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

        let mut highlight_paint = tiny_skia::Paint::default();
        highlight_paint.set_color_rgba8(
            self.highlight_color[0],
            self.highlight_color[1],
            self.highlight_color[2],
            self.highlight_color[3],
        );

        let check_side = self.get_check_side();
        let glyph_width = (check_side * 0.75) as u32;

        let hline = {
            let mut pb = tiny_skia::PathBuilder::new();
            pb.line_to(self.ruler_offset as f32, 0.0);
            pb.finish().unwrap()
        };
        let vline = {
            let mut pb = tiny_skia::PathBuilder::new();
            pb.line_to(0.0, self.ruler_offset as f32);
            pb.finish().unwrap()
        };

        for i in 0..8 {
            let stroke = tiny_skia::Stroke::default();
            {
                // Y-axis
                let t1 =
                    tiny_skia::Transform::from_translate(0.0, (1 + i) as f32 * check_side as f32);
                pixmap.stroke_path(&hline, &ruler_paint, &stroke, t1, None);

                let t2 = tiny_skia::Transform::from_translate(
                    self.ruler_offset as f32 * 0.2,
                    i as f32 * check_side as f32 + check_side * 0.45,
                );
                self.draw_char(('1' as u8 + (7 - i)) as char, 20.0, t2, &mut pixmap);
            }
            {
                // X-axis
                let t1 = tiny_skia::Transform::from_translate(
                    self.ruler_offset as f32 + i as f32 * check_side as f32,
                    self.side_length as f32,
                );
                pixmap.stroke_path(&vline, &ruler_paint, &stroke, t1, None);

                let t2 = tiny_skia::Transform::from_translate(
                    self.ruler_offset as f32 + i as f32 * check_side as f32 + check_side * 0.45,
                    self.side_length as f32 + self.ruler_offset as f32 * 0.2,
                );
                self.draw_char(('A' as u8 + i) as char, 17.0, t2, &mut pixmap);
            }
        }

        // Draw the checkboard and all the arrangement of pieces
        let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, check_side, check_side).unwrap();
        for y in 0..8 {
            for x in 0..8 {
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

                let t = tiny_skia::Transform::from_translate(
                    x as f32 * check_side + self.ruler_offset as f32,
                    (7 - y) as f32 * check_side,
                );
                pixmap.fill_rect(rect, paint, t, None);
                if let Some(_) = self.picked_piece {
                    if moves.is_set((x, y).try_into().unwrap()) {
                        pixmap.fill_rect(rect, &highlight_paint, t, None);
                    }
                }

                if let Some(picked_sq) = self.picked_piece {
                    if (x, y) == picked_sq.into() {
                        continue;
                    }
                }

                if let Some(p) = config.get_at_sq((x, y).try_into().unwrap()).to_owned() {
                    let tree = self.get_glyph_tree(&p);
                    let transform = tiny_skia::Transform::from_translate(
                        // TODO: Fix magic number
                        x as f32 * check_side + self.ruler_offset as f32 + check_side / 8.0,
                        (7 - y) as f32 * check_side + check_side / 8.0,
                    );
                    let fit = usvg::FitTo::Width(glyph_width);
                    resvg::render(&tree, fit, transform, pixmap.as_mut());
                }
            }
        }

        // Draw the picked piece if any
        if let Some(sq) = self.picked_piece {
            let p = config.get_at_sq(sq).unwrap();
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
                // if position.0 as u32 > self.ruler_offset && (position.1 as u32) < self.side_length {
                let position = (position.0, position.1);
                self.mouse_state.update_pos(position);
                // }
            }
            BoardEvent::CursorLeft => {
                self.mouse_state.unset_cursor_in();
            }
        }
    }

    fn get_sq_from_pointer(&self) -> Square {
        let pos = self.mouse_state.get_pos();
        let check_side = self.get_check_side() as usize;
        let off = self.ruler_offset as usize;
        let x = (pos.0.clamp(off, off + self.side_length as usize - 1) - off) / check_side as usize;
        let y = 7 - (pos.1.clamp(0, self.side_length as usize - 1) / check_side as usize);
        (x, y).try_into().unwrap()
    }

    fn get_font_src() -> Vec<u8> {
        let filename = "assets/fonts/Roboto-Bold.ttf";
        std::fs::read(filename).unwrap()
    }

    fn draw_char(
        &mut self,
        c: char,
        px: f32,
        t: tiny_skia::Transform,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let pm = {
            match self.raster_cache.get(&c.to_string()) {
                Some(x) => x,
                None => {
                    log::info!("Rasterizing {}", c);
                    let (metrics, bitmap) = self.font.rasterize(c, px);
                    let mut p: Vec<u8> = bitmap
                        .into_iter()
                        .map(|x| vec![x, x, x, x])
                        .flatten()
                        .collect();
                    let x = tiny_skia::PixmapMut::from_bytes(
                        &mut p,
                        metrics.width as u32,
                        metrics.height as u32,
                    )
                    .unwrap()
                    .to_owned();
                    self.raster_cache.put(&c.to_string(), &x);
                    x
                }
            }
        };
        pixmap.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            t,
            None,
        );
    }
}
