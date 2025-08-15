use core::f32;

use super::ViewState;
use crate::resources::syntax;
use crate::State;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use egui_dock::egui;

pub struct Editor {
    font_size: f32,
}

impl Default for Editor {
    fn default() -> Self {
        Self { font_size: 14.0 }
    }
}

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State, _ctx: &mut egui::Context) {
        let code_buf = state
            .code_buf
            .get_or_insert_with(|| include_str!("../../res/example.asm").to_owned());

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Save & Build").clicked() {
                let mut fs = state.fs.lock().unwrap();
                let mut emu = state.emulator.lock().unwrap();
                let icmc_syntax = include_str!("../../res/icmc.toml");

                #[cfg(target_family = "wasm")]
                {
                    todo!("Need to implement JS wrapper to fs.js");
                    /*
                    fs.write(".code.asm", state.code_buf.as_bytes());
                    fs.write(".icmc.toml", icmc_syntax.as_bytes());
                    */
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    let syntax_path =
                        &format!("{}/icmc.toml", state.ide_path.clone().unwrap().display());

                    let open_file = match state.open_file {
                        Some(f) => f.to_str().unwrap(),
                        &mut None => todo!(),
                    };

                    if let Err(e) = fs.write(open_file, code_buf.as_bytes()) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!("Failed to write .code.asm: {}", e));
                        }
                        return;
                    }

                    if let Err(e) = fs.write(syntax_path, icmc_syntax.as_bytes()) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!("Failed to write .icmc.toml: {}", e));
                        }
                        return;
                    }

                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.auto_scroll();
                    }

                    match assembler::assemble(&fs, open_file, syntax_path) {
                        Ok(asm) => {
                            emu.load(&asm.binary());
                            if let Ok(mut log_panel) = state.log_panel.lock() {
                                log_panel
                                    .add_log("Assembly successful! Binary loaded.".to_string());
                            }
                        }
                        Err(err) => {
                            if let Ok(mut log_panel) = state.log_panel.lock() {
                                log_panel.add_log(format!("Assembly error: {}", err));

                                if let Some((line, col)) = extract_line_column(&err) {
                                    log_panel
                                        .add_log(format!("    at line {}, column {}", line, col));
                                }
                            }
                        }
                    }
                }
            }

            if ui.button("Clear Editor").clicked() {
                code_buf.clear();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("-").clicked() {
                    self.font_size -= 2.0;
                }

                if ui.button("+").clicked() {
                    self.font_size += 2.0;
                }

                ui.label("Font size:");
            });
        });

        let color_theme = if ui.visuals().dark_mode {
            ColorTheme::GITHUB_DARK
        } else {
            ColorTheme::GITHUB_LIGHT
        };

        use std::collections::BTreeSet;

        CodeEditor::default()
            .id_source("asm_editor")
            .with_rows(0)
            .with_fontsize(self.font_size)
            .with_syntax(syntax::icmc())
            .with_theme(color_theme)
            .with_numlines(true)
            .show(ui, code_buf);
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
