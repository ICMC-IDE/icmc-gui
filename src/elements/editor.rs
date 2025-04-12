use super::View;

pub struct Editor {
    code_buf: String,
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
        ui.add(
            egui::TextEdit::multiline(&mut self.code_buf)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_rows(40)
                .desired_width(f32::INFINITY),
        );
    }
}
