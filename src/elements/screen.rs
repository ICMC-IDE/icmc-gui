use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct Screen;

impl Default for Screen {
    fn default() -> Self {
        Self {}
    }
}

/* todo: render charmap into canvas */
impl ViewState for Screen {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            let size = ui.available_width().min(ui.available_height()) - 25.0;
            let square_size = egui::Vec2::splat(size);

            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                ui.set_min_size(square_size);
                ui.set_max_size(square_size);
                ui.allocate_space(square_size);
            });
        });
    }
}
