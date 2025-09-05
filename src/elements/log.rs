use super::ViewState;
use crate::State;
use egui_dock::egui;

#[derive(Default, Clone)]
pub struct LogPanel {
    logs: Vec<String>,
    auto_scroll: bool,
}

impl LogPanel {
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn auto_scroll(&mut self) {
        self.auto_scroll = true;
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }
}

impl ViewState for LogPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _state: &mut State, _ctx: &mut egui::Context) {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Clear Log").clicked() {
                self.clear_logs();
            }
        });

        let font_color = match ui.visuals().dark_mode {
            true => egui::Color32::LIGHT_GRAY,
            false => egui::Color32::DARK_GRAY,
        };

        egui::Frame::canvas(ui.style())
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true);

                scroll_area.show(ui, |ui| {
                    ui.vertical(|ui| {
                        for log in self.logs() {
                            ui.label(egui::RichText::new(log).monospace().color(font_color));
                        }
                        if self.auto_scroll {
                            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                            self.auto_scroll = false;
                        }
                    });
                });
            });
    }
}
