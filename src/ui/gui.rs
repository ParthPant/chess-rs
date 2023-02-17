use std::cell::RefCell;
use std::rc::Rc;

use crate::data::BoardConfig;
use crate::data::piece::Color;
use egui::Context;

pub(super) struct Gui {
    board_config: Rc<RefCell<BoardConfig>>,
    fen: String,
}

impl Gui {
    /// Create a `Gui`.
    pub(super) fn new(board_config: Rc<RefCell<BoardConfig>>) -> Self {
        Self {
            board_config,
            fen: "".to_string(),
        }
    }

    /// Create the UI using egui.
    pub(super) fn ui(&mut self, ctx: &Context) {
        egui::SidePanel::left("Left Panel")
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let mut config = self.board_config.borrow_mut();

                ui.strong("chess-rs");

                ui.heading("In Play");
                ui.label({
                    match config.get_active_color() {
                        Color::White => "White",
                        Color::Black => "Black",
                    }
                });

                ui.separator();

                ui.heading("Board Configuration");
                ui.label(
                    egui::RichText::new(config.get_fen())
                        .size(10.0)
                        .monospace(),
                );
                egui::CollapsingHeader::new("Edit").show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.fen));
                    if ui.button("Load Fen").clicked() {
                        config.load_fen(&self.fen);
                    }
                });

                ui.separator();

                if ui.button("Reset").clicked() {
                    config.reset();
                }
            });
    }
}
