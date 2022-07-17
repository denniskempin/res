use eframe::CreationContext;
use eframe::Frame;
use egui::Context;

pub struct EmulatorApp {}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        EmulatorApp {}
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, _: &Context, _frame: &mut Frame) {}
}
