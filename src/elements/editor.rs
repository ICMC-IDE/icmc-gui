use super::ViewState;
use crate::State;
use crate::resources::syntax;
use egui_code_editor::{CodeEditor, ColorTheme};
use egui_dock::egui;

#[derive(Default)]
pub struct Editor;

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        let code_buf = state
            .code_buf
            .get_or_insert_with(|| {
                include_str!("../../res/example.asm").to_owned()
            })
            .clone();

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                #[cfg(target_family = "wasm")]
                {
                    todo!("Need to implement JS wrapper to fs.js");
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    let open_file = match state.open_file {
                        Some(f) => f.to_str().unwrap(),
                        &mut None => todo!(),
                    };

                    if let Err(e) =
                        std::fs::write(open_file, code_buf.as_bytes())
                    {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!(
                                "Failed to write .code.asm: {}",
                                e
                            ));
                        }
                        return;
                    }
                }
            }

            if ui.button("Build and Run").clicked() {
                let icmc_syntax = include_str!("../../res/icmc.toml");

                if let Ok(mut log_panel) = state.log_panel.lock() {
                    log_panel.auto_scroll();
                }

                match assembler::assemble_from_buf(&code_buf, icmc_syntax) {
                    Ok(asm) => {
                        state
                            .emulator
                            .lock()
                            .unwrap()
                            .load_program(&asm.binary());
                        state.spawn_run_loop();
                    }
                    Err(err) => {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!("Error: {}", err));
                        }
                    }
                };
            }

            if ui.button("Clear Editor").clicked() {
                if let Some(buf) = state.code_buf.as_mut() {
                    buf.clear();
                }
            }

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button("Reset font size").clicked() {
                        state.settings.font_size = 14.0;
                    }

                    if ui.button("-").clicked()
                        && state.settings.font_size >= 4.0
                    {
                        state.settings.font_size -= 2.0;
                    }

                    if ui.button("+").clicked()
                        && state.settings.font_size <= 64.0
                    {
                        state.settings.font_size += 2.0;
                    }

                    ui.label(format!(
                        "Font size: {} pt",
                        state.settings.font_size
                    ));
                },
            );
        });

        let color_theme = if ui.visuals().dark_mode {
            ColorTheme::GITHUB_DARK
        } else {
            ColorTheme::GITHUB_LIGHT
        };

        /* Save with ctrl+S */
        ui.input_mut(|i| {
            let modifiers = egui::Modifiers {
                ctrl: true,
                ..Default::default()
            };

            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                modifiers,
                egui::Key::S,
            )) {
                let open_file = match state.open_file {
                    Some(f) => f.to_str().unwrap(),
                    &mut None => todo!(),
                };

                if let Err(e) = std::fs::write(open_file, &code_buf.as_bytes())
                {
                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.add_log(format!(
                            "Failed to write .code.asm: {}",
                            e
                        ));
                    }
                    return;
                }
            }
        });

        let mut_code_buf = state.code_buf.get_or_insert_with(|| {
            include_str!("../../res/example.asm").to_owned()
        });

        CodeEditor::default()
            .id_source("asm_editor")
            .with_rows(0)
            .with_fontsize(state.settings.font_size)
            .with_theme(color_theme)
            .with_numlines(true)
            .show(ui, mut_code_buf, &syntax::icmc());
    }
}
