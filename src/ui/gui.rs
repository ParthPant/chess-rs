use egui::Context;

pub(super) struct Gui {
}

impl Gui {
    /// Create a `Gui`.
    pub(super) fn new() -> Self {
        Self {}
    }

    /// Create the UI using egui.
    pub(super) fn ui(&mut self, ctx: &Context) {
        egui::SidePanel::left("Left Panel")
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                ui.label("This example demonstrates using egui with pixels.");
            });
        // egui::Window::new("Hello egui!")
        //     .show(ctx, |ui| {
        //         ui.label("This example demonstrates using egui with pixels.");
        //     });
    }
}

