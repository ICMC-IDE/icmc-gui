use crate::elements::{Editor, View};
use icmc_emulator::State;

pub struct IDEApp {
    emulator: icmc_emulator::Emulator,
    editor: Editor,
}

impl Default for IDEApp {
    fn default() -> Self {
        Self {
            emulator: icmc_emulator::Emulator::new(),
            editor: Editor::default(),
        }
    }
}

impl IDEApp {
    fn top_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::widgets::global_theme_preference_switch(ui);

        /* add other options (panels) */
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

/* App trait */
impl eframe::App for IDEApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        /* top menu */
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.top_bar(ui, frame);
        });

        /* Code editor (just for testing, will change later) */
        egui::SidePanel::left("Code Editor")
            .exact_width(ctx.screen_rect().width() / 2.0)
            .show(ctx, |ui| {
                self.editor.ui(ui);
            });

        /* Screen panel */
        egui::SidePanel::right("Screen")
            .exact_width(ctx.screen_rect().width() / 2.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        ui.set_min_size(egui::Vec2::splat(300.0));
                    });

                    ui.label(format!(
                        "State: {}",
                        match self.emulator.state() {
                            State::Paused => "Paused",
                            State::BreakPoint => "BreakPoint",
                            State::Halted => "Halted",
                            State::UnknownInstruction => "Unknown Instruction",
                        }
                    ));
                });
            });
    }
}
