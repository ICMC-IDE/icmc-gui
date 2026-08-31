mod find_replace;
mod goto_line;
mod gutter;

use super::ViewState;
use crate::State;
use crate::resources::syntax;
use egui_code_editor::{CodeEditor, ColorTheme};
use egui_dock::egui;
use find_replace::FindReplace;
use goto_line::GotoLine;
use gutter::Gutter;
use std::ops::Range;

#[derive(Default)]
pub struct Editor {
    find: FindReplace,
    goto: GotoLine,
    gutter: Gutter,
    editor_id: Option<egui::Id>,
}

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
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

                    let code_buf = state.code_buf.get_or_insert_with(|| {
                        include_str!("../../../res/example.asm").to_owned()
                    });

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
                let icmc_syntax = include_str!("../../../res/icmc.toml");

                if let Ok(mut log_panel) = state.log_panel.lock() {
                    log_panel.auto_scroll();
                }

                let code_buf = state.code_buf.get_or_insert_with(|| {
                    include_str!("../../../res/example.asm").to_owned()
                });

                match assembler::assemble_from_buf(code_buf.as_str(), icmc_syntax) {
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

                let code_buf = state.code_buf.get_or_insert_with(|| {
                    include_str!("../../../res/example.asm").to_owned()
                });

                if let Err(e) = std::fs::write(open_file, code_buf.as_bytes())
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

        /* Find (ctrl+F), Replace (ctrl+H), Go to Line (ctrl+G), Close (Esc) */
        let ctrl = egui::Modifiers {
            ctrl: true,
            ..Default::default()
        };

        if ui.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::F))
        }) {
            self.goto.close();
            self.find.open_find();
        }

        if ui.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::H))
        }) {
            self.goto.close();
            self.find.open_replace();
        }

        if ui.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::G))
        }) {
            self.find.close();
            self.goto.activate();
        }

        let mut focus_editor = false;

        if (self.find.open || self.goto.open)
            && ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
            })
        {
            self.find.close();
            self.goto.close();
            focus_editor = true;
        }

        let mut_code_buf = state.code_buf.get_or_insert_with(|| {
            include_str!("../../../res/example.asm").to_owned()
        });

        let editor_rect = ui.available_rect_before_wrap();

        let find_was_open = self.find.open;
        let find_jump = self.find.show(ui, editor_rect, color_theme, mut_code_buf);
        focus_editor |= find_was_open && !self.find.open;

        let goto_was_open = self.goto.open;
        let goto_line = self.goto.show(ui, editor_rect, color_theme);
        focus_editor |= goto_was_open && !self.goto.open;

        let pending_jump: Option<Range<usize>> = if let Some(line) = goto_line {
            let offset = line_end_byte_offset(mut_code_buf, line);
            Some(offset..offset)
        } else {
            find_jump
        };

        if let Some(range) = &pending_jump {
            if let Some(editor_id) = self.editor_id {
                let start = egui::text::CCursor::new(char_index(mut_code_buf, range.start));
                let end = egui::text::CCursor::new(char_index(mut_code_buf, range.end));
                let cursor_range = egui::text::CCursorRange::two(start, end);

                let mut ted_state =
                    egui::widgets::text_edit::TextEditState::load(ui.ctx(), editor_id)
                        .unwrap_or_default();
                ted_state.cursor.set_char_range(Some(cursor_range));
                ted_state.store(ui.ctx(), editor_id);
            }
        }

        if focus_editor && let Some(editor_id) = self.editor_id {
            ui.ctx().memory_mut(|mem| mem.request_focus(editor_id));
        }

        self.gutter.update(mut_code_buf);

        let output = egui::ScrollArea::vertical()
            .id_salt("asm_editor_scroll")
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    self.gutter.show(ui, color_theme, state.settings.font_size);

                    let output = CodeEditor::default()
                        .id_source("asm_editor")
                        .with_rows(0)
                        .with_fontsize(state.settings.font_size)
                        .with_theme(color_theme)
                        .with_numlines(false)
                        .vscroll(false)
                        .show(ui, mut_code_buf, syntax::icmc());

                    if let Some(range) = &pending_jump {
                        let start =
                            egui::text::CCursor::new(char_index(mut_code_buf, range.start));
                        let rect = output
                            .galley
                            .pos_from_cursor(start)
                            .translate(output.galley_pos.to_vec2());
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }

                    if self.find.open {
                        let matches = self.find.matches(mut_code_buf);
                        find_replace::paint_highlights(
                            ui,
                            &output,
                            mut_code_buf,
                            &matches,
                            self.find.current(),
                            color_theme,
                        );
                    }

                    output
                })
                .inner
            })
            .inner;

        self.editor_id = Some(output.response.id);
    }
}

fn char_index(text: &str, byte_idx: usize) -> usize {
    text[..byte_idx].chars().count()
}

fn line_start_byte_offset(text: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }

    text.match_indices('\n')
        .nth(line - 2)
        .map(|(i, _)| i + 1)
        .unwrap_or(text.len())
}

fn line_end_byte_offset(text: &str, line: usize) -> usize {
    let start = line_start_byte_offset(text, line);
    text[start..]
        .find('\n')
        .map_or(text.len(), |i| start + i)
}
