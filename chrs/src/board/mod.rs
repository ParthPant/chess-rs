mod embed;
pub mod events;

use crate::cache::Cache;
use chrs_lib::data::{BoardConfig, BoardPiece, Color, GameState, Move, MoveList, Square};
use chrs_lib::generator::MoveGenerator;
use embed::{EmbeddedFonts, SvgSprites};
use events::{BoardEvent, ElementState, MouseButton, MouseState};
use fontdue::{
    layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle},
    Font,
};
use resvg::{tiny_skia, usvg};

const W_PROM_OPTS: [BoardPiece; 4] = [
    BoardPiece::WhiteKnight,
    BoardPiece::WhiteBishop,
    BoardPiece::WhiteRook,
    BoardPiece::WhiteQueen,
];

const B_PROM_OPTS: [BoardPiece; 4] = [
    BoardPiece::BlackKnight,
    BoardPiece::BlackBishop,
    BoardPiece::BlackRook,
    BoardPiece::BlackQueen,
];

pub struct Board {
    side_length: u32,
    ruler_offset: u32,
    white_color: [u8; 4],
    black_color: [u8; 4],
    ruler_color: [u8; 4],
    highlight_color: [u8; 4],
    red_highlight_color: [u8; 4],
    font: Font,
    glyph_cache: Cache<usvg::Tree>,
    raster_cache: Cache<tiny_skia::Pixmap>,
    picked_piece: Option<Square>,
    mouse_state: MouseState,
    user_move: Option<Move>,
    overlay_xywh: (f32, f32, f32, f32),
}

impl Default for Board {
    #[rustfmt::skip]
    fn default() -> Self {
        let font_src = Self::get_font_src();
        let font = fontdue::Font::from_bytes(font_src, fontdue::FontSettings::default()).unwrap();
        let check_side = 720.0 / 8.0;
        let size = 720.0 + 20.0;
        Board {
            side_length: 720,
            ruler_offset: 20,
            white_color: [0xe3, 0xc1, 0x6f, 0xff],
            black_color: [0xb8, 0x8b, 0x4a, 0xff],
            ruler_color: [0xff, 0xff, 0xff, 0xff],
            highlight_color: [0x3f, 0x7a, 0xd9, 0x40],
            red_highlight_color: [0xff, 0x20, 0x20, 0xff],
            font,
            glyph_cache: Cache::default(),
            raster_cache: Cache::default(),
            mouse_state: MouseState::default(),
            picked_piece: None,
            user_move: None,
            overlay_xywh: (size/2.0-2.0*check_side, size/2.0-0.5*check_side, 4.0*check_side, check_side),
        }
    }
}

impl Board {
    pub fn get_user_move(&mut self) -> Option<Move> {
        self.user_move
    }

    pub fn clear_user_move(&mut self) {
        self.user_move = None
    }

    pub fn get_picked_piece(&self) -> Option<Square> {
        self.picked_piece
    }

    fn get_pos_prom_box(&self, pos: &(usize, usize)) -> Option<(usize, usize)> {
        let inside = pos.0 > self.overlay_xywh.0 as usize
            && pos.0 < (self.overlay_xywh.0 + self.overlay_xywh.2) as usize
            && pos.1 > self.overlay_xywh.1 as usize
            && pos.1 < (self.overlay_xywh.1 + self.overlay_xywh.3) as usize;
        if inside {
            let x = pos.0 - self.overlay_xywh.0 as usize;
            let y = pos.1 - self.overlay_xywh.1 as usize;
            return Some((x, y));
        }
        None
    }

    pub fn handle_event(&mut self, e: BoardEvent, config: &BoardConfig) {
        self.update_mouse_state(e);

        if config.get_state() != GameState::InPlay {
            return;
        }

        if let Some(m) = &self.user_move {
            if m.is_empty_prom() {
                if self.mouse_state.get_is_left_pressed() {
                    let pos = self.mouse_state.get_pos();
                    if let Some((x, _)) = self.get_pos_prom_box(&pos) {
                        let i = x / self.overlay_xywh.3 as usize;
                        let prom = match config.get_active_color() {
                            Color::White => W_PROM_OPTS[i],
                            Color::Black => B_PROM_OPTS[i],
                        };
                        self.user_move = Some(Move::new_prom(m.from, m.to, m.p, m.capture, prom));
                    } else {
                        self.clear_user_move();
                    }
                }
                return;
            }
        }

        let sq = self.get_sq_from_pointer();

        if self.mouse_state.get_is_left_pressed() {
            if let None = self.picked_piece {
                if let Some(p) = config.get_at_sq(sq) {
                    if p.get_color() == config.get_active_color() {
                        self.picked_piece = Some(sq);
                    }
                }
            }
        }

        if !self.mouse_state.get_is_left_pressed() {
            if let Some(prev) = self.picked_piece {
                self.user_move = Some(Move::infer(prev, sq, config));
                self.picked_piece = None;
            }
        }

        if !self.mouse_state.get_is_cursor_in() {
            self.picked_piece = None;
        }
    }

    pub fn draw(
        &mut self,
        frame: &mut [u8],
        gen: &MoveGenerator,
        config: &BoardConfig,
        moves: &Option<MoveList>,
    ) {
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

        let mut red_highlight_paint = tiny_skia::Paint::default();
        red_highlight_paint.set_color_rgba8(
            self.red_highlight_color[0],
            self.red_highlight_color[1],
            self.red_highlight_color[2],
            self.red_highlight_color[3],
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
                        &black_paint
                    } else {
                        &white_paint
                    }
                } else {
                    if y % 2 == 0 {
                        &white_paint
                    } else {
                        &black_paint
                    }
                };

                let t = tiny_skia::Transform::from_translate(
                    x as f32 * check_side + self.ruler_offset as f32,
                    (7 - y) as f32 * check_side,
                );
                pixmap.fill_rect(rect, paint, t, None);
                if let Some(_) = self.picked_piece {
                    if moves.is_some()
                        && moves
                            .as_ref()
                            .unwrap()
                            .has_target_sq((x, y).try_into().unwrap())
                    {
                        pixmap.fill_rect(rect, &highlight_paint, t, None);
                    }
                }

                if let Some(picked_sq) = self.picked_piece {
                    if (x, y) == picked_sq.into() {
                        continue;
                    }
                }

                if let Some(p) = config.get_at_sq((x, y).try_into().unwrap()).to_owned() {
                    if (p == BoardPiece::WhiteKing || p == BoardPiece::BlackKing)
                        && config.is_king_in_check(gen, p.get_color())
                    {
                        pixmap.fill_rect(rect, &red_highlight_paint, t, None);
                    }
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

        if let Some(m) = self.user_move {
            if m.is_empty_prom() {
                let transform =
                    tiny_skia::Transform::from_translate(self.overlay_xywh.0, self.overlay_xywh.1);
                self.draw_prom_choice(config.get_active_color(), transform, &mut pixmap);
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

        if let GameState::Mate(mate) = config.get_state() {
            let transform =
                tiny_skia::Transform::from_translate(self.overlay_xywh.0, self.overlay_xywh.1);
            self.draw_text(
                &format!("Check Mate: {}", mate),
                32.0,
                transform,
                &mut pixmap,
            )
        } else if config.get_state() == GameState::StaleMate {
            let transform =
                tiny_skia::Transform::from_translate(self.overlay_xywh.0, self.overlay_xywh.1);
            self.draw_text(&format!("It's a Stalemate"), 32.0, transform, &mut pixmap)
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
                let str = Board::get_svg_src(&glyph_path);
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
        let s = format!("{}.svg", p);
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
        let filename = "Roboto-Bold.ttf";
        let content = EmbeddedFonts::get(filename).expect(&format!("{} not found", filename));
        content.data.as_ref().to_vec()
    }

    fn get_svg_src(filename: &str) -> String {
        let content = SvgSprites::get(filename).expect(&format!("{} not found", filename));
        let content = std::str::from_utf8(content.data.as_ref()).unwrap();
        content.to_string()
    }

    fn draw_prom_choice(
        &mut self,
        color: Color,
        t: tiny_skia::Transform,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let check_side = self.get_check_side();
        let mut pm =
            tiny_skia::Pixmap::new(self.overlay_xywh.2 as u32, self.overlay_xywh.3 as u32).unwrap();
        let glyph_width = (check_side * 0.75) as u32;

        let pieces = match color {
            Color::White => W_PROM_OPTS,
            Color::Black => B_PROM_OPTS,
        };

        pm.fill(tiny_skia::Color::WHITE);
        for (i, p) in pieces.iter().enumerate() {
            let tree = self.get_glyph_tree(&p);
            let glyph_t = tiny_skia::Transform::from_translate(
                i as f32 * check_side + glyph_width as f32 / 8.0,
                glyph_width as f32 / 8.0,
            );
            let fit = usvg::FitTo::Width(glyph_width);
            resvg::render(&tree, fit, glyph_t, pm.as_mut());
        }

        pixmap.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            t,
            None,
        );
    }

    fn draw_text(
        &mut self,
        s: &str,
        px: f32,
        t: tiny_skia::Transform,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let mut pm =
            tiny_skia::Pixmap::new(self.overlay_xywh.2 as u32, self.overlay_xywh.3 as u32).unwrap();
        // pm.fill(tiny_skia::Color::WHITE);

        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: Some(self.overlay_xywh.2),
            max_height: Some(self.overlay_xywh.3),
            horizontal_align: HorizontalAlign::Center,
            ..LayoutSettings::default()
        });
        layout.append(&[&self.font], &TextStyle::new(s, px, 0));

        for glyph in layout.glyphs() {
            let (_, bitmap) = self.font.rasterize_indexed(glyph.key.glyph_index, px);
            let mut bitmap: Vec<u8> = bitmap
                .into_iter()
                .map(|x| vec![0, 0, 0, x])
                .flatten()
                .collect();

            if glyph.char_data.is_whitespace() {
                continue;
            }
            let x = tiny_skia::PixmapMut::from_bytes(
                &mut bitmap,
                glyph.width as u32,
                glyph.height as u32,
            )
            .unwrap()
            .to_owned();
            pm.draw_pixmap(
                glyph.x as i32,
                glyph.y as i32,
                x.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        pixmap.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            t,
            None,
        );
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
