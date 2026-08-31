use egui_code_editor::{ColorTheme, TokenType};
use egui_dock::egui;

#[derive(Default)]
pub struct Gutter {
    line_count: usize,
    text: String,
}

impl Gutter {
    pub fn update(&mut self, code: &str) {
        let line_count = if code.is_empty() || code.ends_with('\n') {
            code.lines().count() + 1
        } else {
            code.lines().count()
        };

        if line_count == self.line_count {
            return;
        }
        self.line_count = line_count;

        let width = line_count.to_string().len();
        self.text.clear();
        for i in 1..=line_count {
            if i > 1 {
                self.text.push('\n');
            }
            use std::fmt::Write as _;
            let _ = write!(self.text, "{i:>width$}");
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: ColorTheme, fontsize: f32) {
        let digits = self.line_count.to_string().len().max(1) as f32;
        let width = digits * fontsize * 0.5;
        let color = theme.type_color(TokenType::Comment(true));

        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap_width: f32| {
            let job = egui::text::LayoutJob::single_section(
                buf.as_str().to_owned(),
                egui::TextFormat::simple(egui::FontId::monospace(fontsize), color),
            );
            ui.fonts_mut(|f| f.layout_job(job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut self.text)
                .id_source("asm_editor_gutter")
                .font(egui::TextStyle::Monospace)
                .interactive(false)
                .frame(egui::Frame::NONE)
                .desired_rows(0)
                .desired_width(width)
                .layouter(&mut layouter),
        );
    }
}
