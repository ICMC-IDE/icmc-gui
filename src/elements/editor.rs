use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct Editor {
    code_buf: String, /* Editor buffer */
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            code_buf: include_str!("../../res/example.asm").to_owned(),
        }
    }
}

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State, _ctx: &mut egui::Context) {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Save & Build").clicked() {
                let mut fs = state.fs.lock().unwrap();
                let mut emu = state.emulator.lock().unwrap();
                let icmc_syntax = include_str!("../../res/icmc.toml");

                if let Err(e) = fs.write(".code.asm", self.code_buf.as_bytes()) {
                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.add_log(format!("Failed to write .code.asm: {}", e));
                    }
                    return;
                }

                if let Err(e) = fs.write(".icmc.toml", icmc_syntax.as_bytes()) {
                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.add_log(format!("Failed to write .icmc.toml: {}", e));
                    }
                    return;
                }

                if let Ok(mut log_panel) = state.log_panel.lock() {
                    log_panel.auto_scroll();
                }

                match assembler::assemble(&fs, ".code.asm", ".icmc.toml") {
                    Ok(asm) => {
                        emu.load(&asm.binary());
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log("Assembly successful! Binary loaded.".to_string());
                        }
                    }
                    Err(err) => {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!("Assembly error: {}", err));

                            if let Some((line, col)) = extract_line_column(&err) {
                                log_panel.add_log(format!("    at line {}, column {}", line, col));
                            }
                        }
                    }
                }
            }

            if ui.button("Clear Editor").clicked() {
                self.code_buf.clear();
            }
        });

        ui.add(
            egui::TextEdit::multiline(&mut self.code_buf)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_rows(50)
                .desired_width(f32::INFINITY),
        );
    }
}

fn extract_line_column(error_msg: &str) -> Option<(usize, usize)> {
    let mut line = 0;
    let mut col = 0;

    if let Some(pos) = error_msg.find("line ") {
        let rest = &error_msg[pos + 5..];
        line = rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
    }

    if let Some(pos) = error_msg.find("column ") {
        let rest = &error_msg[pos + 7..];
        col = rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
    }

    if line > 0 || col > 0 {
        Some((line, col))
    } else {
        None
    }
}
