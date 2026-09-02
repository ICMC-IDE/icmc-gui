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

impl Editor {
    pub fn open_find(&mut self) {
        self.goto.close();
        self.find.open_find();
    }

    pub fn open_replace(&mut self) {
        self.goto.close();
        self.find.open_replace();
    }

    pub fn activate_goto(&mut self) {
        self.find.close();
        self.goto.activate();
    }
}

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.add_space(10.0);

        let color_theme = if ui.visuals().dark_mode {
            ColorTheme::GITHUB_DARK
        } else {
            ColorTheme::GITHUB_LIGHT
        };

        /* Save with ctrl+S */
        if ui.input_mut(|i| {
            let modifiers = egui::Modifiers {
                ctrl: true,
                ..Default::default()
            };

            i.consume_shortcut(&egui::KeyboardShortcut::new(modifiers, egui::Key::S))
        }) {
            state.save_file();
        }

        /* Find (ctrl+F), Replace (ctrl+H), Go to Line (ctrl+G), Close (Esc) */
        let ctrl = egui::Modifiers {
            ctrl: true,
            ..Default::default()
        };

        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::F))) {
            self.open_find();
        }

        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::H))) {
            self.open_replace();
        }

        if ui.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(ctrl, egui::Key::G))) {
            self.activate_goto();
        }

        let mut focus_editor = false;

        if (self.find.open || self.goto.open)
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.find.close();
            self.goto.close();
            focus_editor = true;
        }

        let mut_code_buf = state
            .code_buf
            .get_or_insert_with(|| include_str!("../../../res/example.asm").to_owned());

        let editor_rect = ui.available_rect_before_wrap();

        let row_height = ui
            .fonts_mut(|f| f.row_height(&egui::FontId::monospace(state.settings.font_size)));
        let min_rows = (editor_rect.height() / row_height).floor().max(1.0) as usize;

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
                    self.gutter
                        .show(ui, color_theme, state.settings.font_size, min_rows);

                    let output = CodeEditor::default()
                        .id_source("asm_editor")
                        .with_rows(min_rows)
                        .with_fontsize(state.settings.font_size)
                        .with_theme(color_theme)
                        .with_numlines(false)
                        .vscroll(false)
                        .show(ui, mut_code_buf, syntax::icmc());

                    if let Some(range) = &pending_jump {
                        let start = egui::text::CCursor::new(char_index(mut_code_buf, range.start));
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
    text[start..].find('\n').map_or(text.len(), |i| start + i)
}
