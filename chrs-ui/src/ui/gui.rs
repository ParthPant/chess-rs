use std::cell::RefCell;
use std::rc::Rc;

use chrs_core::ai::NegaMaxAI;
use chrs_core::data::BoardConfig;
use chrs_core::data::Color;
use egui::Slider;
use egui::{Color32, Context};

pub struct Gui {
    fen: String,
    bit_board: String,
}

impl Gui {
    /// Create a `Gui`.
    pub fn new() -> Self {
        Self {
            fen: "".to_string(),
            bit_board: "p".to_string(),
        }
    }

    /// Create the UI using egui.
    pub fn ui(&mut self, ctx: &Context, config: &mut BoardConfig, ai: &mut NegaMaxAI) {
        egui::SidePanel::left("Left Panel")
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(5.))
            .show(ctx, |ui| {
                ui.strong("chess-rs");

                ui.heading("In Play");
                ui.label({
                    match config.get_active_color() {
                        Color::White => "White",
                        Color::Black => "Black",
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        config.reset();
                    }

                    if ui.button("Undo").clicked() {
                        config.undo();
                        config.undo();
                    }
                });

                ui.separator();

                ui.heading("Board Configuration");
                egui::CollapsingHeader::new("FEN").show(ui, |ui| {
                    ui.label(egui::RichText::new(config.get_fen()).size(10.0).monospace());
                    if ui
                        .add(egui::Label::new("📋").sense(egui::Sense::click()))
                        .clicked()
                    {
                        ui.output().copied_text = config.get_fen();
                    }
                    ui.add(egui::TextEdit::multiline(&mut self.fen));
                    if ui.button("Load Fen").clicked() {
                        config.load_fen(&self.fen);
                    }
                });

                ui.separator();

                ui.heading("AI");
                ui.add(Slider::new(&mut ai.depth, 2..=5).text("Search Depth"));
                ui.add(Slider::new(&mut ai.quiescence_depth, 2..=5).text("Quiescence Depth"));

                ui.separator();

                ui.label(format!("Nodes Searched: {}", ai.stats.node_count));
                ui.label(format!("Max Depth: {}", ai.stats.max_depth));
                ui.label(format!("Time Taken: {:?}", ai.stats.time));
            });
    }
}
