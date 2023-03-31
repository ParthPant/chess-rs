use std::cell::RefCell;
use std::rc::Rc;

use chrs_lib::ai::NegaMaxAI;
use chrs_lib::data::BoardConfig;
use chrs_lib::data::Color;
use egui::Slider;
use egui::{Color32, Context};

pub struct Gui {
    fen: String,
    bit_board: String,
    show_menu: bool,
    show_about: bool,
}

impl Gui {
    /// Create a `Gui`.
    pub fn new() -> Self {
        Self {
            fen: "".to_string(),
            bit_board: "p".to_string(),
            show_menu: true,
            show_about: false,
        }
    }

    /// Create the UI using egui.
    pub fn ui(&mut self, ctx: &Context, config: &mut BoardConfig, ai: &mut NegaMaxAI) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.visuals_mut().button_frame = false;

            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                ui.toggle_value(&mut self.show_menu, "⚙ Menu");
                ui.separator();
                ui.toggle_value(&mut self.show_about, "ℹ About");
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.visuals_mut().button_frame = false;

            ui.horizontal(|ui| {
                if config.move_history.counter > 0 {
                    ui.label("Recent Moves: ");
                    let end = config.move_history.counter;
                    let start = end.saturating_sub(5);
                    let mut alpha = 0xff;
                    for i in (start..end).rev() {
                        ui.label(
                            egui::RichText::new(format!(
                                "{}",
                                config.move_history.list[i as usize].unwrap()
                            ))
                            .color(
                                egui::Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, alpha),
                            ),
                        );
                        alpha = alpha.saturating_sub(50);
                        if i != start {
                            ui.separator();
                        }
                    }
                } else {
                    ui.label("No moves yet");
                }
            });
        });

        egui::Window::new("ℹ About")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.strong("chess-rs");
                    ui.strong(format!("v{}", env!("CARGO_PKG_VERSION")));
                    ui.label("A Chess Engine written in Rust that runs natively and on the web!");
                    ui.strong("⚖ MIT license");
                    ui.strong("Author: Parth Pant");
                    ui.strong("Email: parthpant4@gmail.com");
                    use egui::special_emojis::{GITHUB, TWITTER};
                    ui.hyperlink_to(
                        format!("{} chess-rs on GitHub", GITHUB),
                        "https://github.com/ParthPant/chess-rs",
                    );
                    ui.hyperlink_to(
                        format!("{} @PantParth", TWITTER),
                        "https://twitter.com/PantParth",
                    );
                });
            });

        egui::SidePanel::left("left_Panel")
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(5.))
            .show_animated(ctx, self.show_menu, |ui| {
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
                        ui.output_mut(|o| o.copied_text = config.get_fen());
                    }
                    ui.add(egui::TextEdit::multiline(&mut self.fen));
                    if ui.button("Load Fen").clicked() {
                        config.load_fen(&self.fen);
                    }
                });

                ui.separator();

                ui.heading("AI");
                ui.add(Slider::new(&mut ai.depth, 2..=8).text("Search Depth"));
                ui.add(Slider::new(&mut ai.quiescence_depth, 2..=8).text("Quiescence Depth"));

                ui.separator();

                ui.label(format!("Nodes Searched: {}", ai.stats.node_count));
                ui.label(format!("Max Depth: {}", ai.stats.max_depth));
                ui.label(format!("Time Taken: {:?}", ai.stats.time));
            });
    }
}
