use egui_code_editor::ColorTheme;
use egui_dock::egui;

#[derive(Default)]
pub struct GotoLine {
    pub open: bool,
    pub input: String,
    focus_input: bool,
}

impl GotoLine {
    pub fn activate(&mut self) {
        self.open = true;
        self.focus_input = true;
        self.input.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        anchor: egui::Rect,
        theme: ColorTheme,
    ) -> Option<usize> {
        if !self.open {
            return None;
        }

        let mut submitted = None;

        let width = 260.0_f32.min((anchor.width() - 16.0).max(180.0));
        let pos = egui::pos2(anchor.center().x - width / 2.0, anchor.top() + 8.0);

        egui::Area::new(ui.id().with("goto_line_box"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .movable(false)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme.bg())
                    .stroke(egui::Stroke::new(1.0, theme.selection()))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_width(width);

                        ui.horizontal(|ui| {
                            let field = ui.add(
                                egui::TextEdit::singleline(&mut self.input)
                                    .hint_text("Go to line…")
                                    .desired_width(width - 60.0)
                                    // Otherwise the TextEdit consumes Enter
                                    // internally before we ever see it.
                                    .return_key(None),
                            );

                            if self.focus_input {
                                field.request_focus();
                                self.focus_input = false;
                            }

                            if field.has_focus()
                                && ui.input_mut(|i| {
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                })
                                && let Ok(line) = self.input.trim().parse::<usize>()
                            {
                                submitted = Some(line.max(1));
                            }

                            if ui.small_button("x").on_hover_text("Close (Esc)").clicked() {
                                self.close();
                            }
                        });
                    });
            });

        if submitted.is_some() {
            self.close();
        }

        submitted
    }
}
