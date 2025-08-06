use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct Editor;

impl Default for Editor {
    fn default() -> Self {
        Self {}
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

                /* TODO: stop saving code in "./.code.asm" and implement
                 * a file explorer */

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
        });

        ui.add(
            egui::TextEdit::multiline(code_buf)
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
