use std::cell::RefCell;
use std::rc::Rc;

use crate::boarddata::BoardConfig;
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
                ui.strong("chess-rs");

                ui.separator();

                ui.heading("Current Configuration: ");
                ui.label(
                    egui::RichText::new(self.board_config.borrow().get_fen())
                        .size(10.0)
                        .monospace(),
                );
                egui::CollapsingHeader::new("Edit").show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.fen));
                    if ui.button("Load Fen").clicked() {
                        self.board_config.borrow_mut().load_fen(&self.fen);
                    }
                });

                ui.separator();
            });
    }
}
