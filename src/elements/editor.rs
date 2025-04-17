use super::View;
use egui_dock::egui;

pub struct Editor {
    code_buf: String, /* Editor buffer */
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            code_buf: String::new(),
        }
    }
}

impl View for Editor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);

        if ui.button("Save & Build").clicked() {
            /* todo: save code_buf into fs then load it
             * into the assembler */
        }

        /* fit editor into panel screen */
        let size = egui::Vec2::new(ui.available_width(), ui.available_height());

        ui.add(
            egui::TextEdit::multiline(&mut self.code_buf)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .min_size(size)
                .desired_width(f32::INFINITY),
        );
    }
}
