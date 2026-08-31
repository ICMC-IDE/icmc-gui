use egui_code_editor::ColorTheme;
use egui_dock::egui;
use std::borrow::Cow;
use std::ops::Range;

#[derive(Default)]
pub struct FindReplace {
    pub open: bool,
    pub replace_open: bool,
    pub query: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    current: usize,
    focus_query: bool,
    last_query: String,
    cache: MatchCache,
}

#[derive(Default)]
struct MatchCache {
    text: String,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    matches: Vec<Range<usize>>,
}

impl FindReplace {
    pub fn open_find(&mut self) {
        self.open = true;
        self.focus_query = true;
    }

    pub fn open_replace(&mut self) {
        self.open = true;
        self.replace_open = true;
        self.focus_query = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.replace_open = false;
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn matches(&mut self, text: &str) -> Vec<Range<usize>> {
        let stale = self.cache.text != text
            || self.cache.query != self.query
            || self.cache.case_sensitive != self.case_sensitive
            || self.cache.whole_word != self.whole_word;

        if stale {
            self.cache.matches =
                find_matches(text, &self.query, self.case_sensitive, self.whole_word);
            self.cache.text = text.to_owned();
            self.cache.query = self.query.clone();
            self.cache.case_sensitive = self.case_sensitive;
            self.cache.whole_word = self.whole_word;
        }

        self.cache.matches.clone()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        anchor: egui::Rect,
        theme: ColorTheme,
        code_buf: &mut String,
    ) -> Option<Range<usize>> {
        if !self.open {
            return None;
        }

        let query_changed = self.query != self.last_query;
        self.last_query = self.query.clone();
        let just_opened = self.focus_query;

        let mut matches = self.matches(code_buf);
        if matches.is_empty() {
            self.current = 0;
        } else if self.current >= matches.len() {
            self.current = matches.len() - 1;
        }

        let mut go_next = false;
        let mut go_prev = false;
        let mut replace_one = false;
        let mut replace_all = false;

        let width = 340.0_f32.min((anchor.width() - 16.0).max(220.0));
        let pos = egui::pos2(anchor.right() - width - 100.0, anchor.top() + 8.0);

        egui::Area::new(ui.id().with("find_replace_bar"))
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

                        // Based on the actual panel width, not the bar's own
                        // (clamped) width — below this, the query field gets
                        // too cramped sharing a row with the match count and
                        // nav buttons, so wrap those onto their own row.
                        let narrow = anchor.width() < 480.0;
                        let field_width = if narrow { width - 100.0 } else { width - 190.0 };

                        ui.horizontal(|ui| {
                            if ui
                                .small_button(if self.replace_open { "-" } else { "+" })
                                .on_hover_text("Toggle replace")
                                .clicked()
                            {
                                self.replace_open = !self.replace_open;
                            }

                            let field = ui.add(
                                egui::TextEdit::singleline(&mut self.query)
                                    .hint_text("Find")
                                    .desired_width(field_width)
                                    .return_key(None),
                            );
                            if self.focus_query {
                                field.request_focus();
                                self.focus_query = false;
                            }

                            if field.has_focus() {
                                if ui.input_mut(|i| {
                                    i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter)
                                }) {
                                    go_prev = true;
                                } else if ui.input_mut(|i| {
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                }) {
                                    go_next = true;
                                }
                            }

                            ui.toggle_value(&mut self.case_sensitive, "Aa")
                                .on_hover_text("Match case");
                            ui.toggle_value(&mut self.whole_word, "Ab")
                                .on_hover_text("Match whole word");

                            if !narrow {
                                ui.label(if matches.is_empty() {
                                    "No results".to_owned()
                                } else {
                                    format!("{}/{}", self.current + 1, matches.len())
                                });

                                if ui
                                    .small_button("Prev")
                                    .on_hover_text("Previous match (Shift+Enter)")
                                    .clicked()
                                {
                                    go_prev = true;
                                }
                                if ui
                                    .small_button("Next")
                                    .on_hover_text("Next match (Enter)")
                                    .clicked()
                                {
                                    go_next = true;
                                }
                                if ui.small_button("x").on_hover_text("Close (Esc)").clicked() {
                                    self.close();
                                }
                            }
                        });

                        if narrow {
                            ui.horizontal(|ui| {
                                ui.label(if matches.is_empty() {
                                    "No results".to_owned()
                                } else {
                                    format!("{}/{}", self.current + 1, matches.len())
                                });

                                if ui
                                    .small_button("Prev")
                                    .on_hover_text("Previous match (Shift+Enter)")
                                    .clicked()
                                {
                                    go_prev = true;
                                }
                                if ui
                                    .small_button("Next")
                                    .on_hover_text("Next match (Enter)")
                                    .clicked()
                                {
                                    go_next = true;
                                }
                                if ui.small_button("x").on_hover_text("Close (Esc)").clicked() {
                                    self.close();
                                }
                            });
                        }

                        if self.replace_open {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.replacement)
                                        .hint_text("Replace")
                                        .desired_width(width - 190.0),
                                );

                                if ui.small_button("Replace").clicked() {
                                    replace_one = true;
                                }
                                if ui.small_button("Replace All").clicked() {
                                    replace_all = true;
                                }
                            });
                        }
                    });
            });

        if replace_all {
            for range in matches.iter().rev() {
                code_buf.replace_range(range.clone(), &self.replacement);
            }
            self.current = 0;
            return None;
        }

        if replace_one && !matches.is_empty() {
            let range = matches[self.current].clone();
            code_buf.replace_range(range, &self.replacement);
            matches = self.matches(code_buf);
            if matches.is_empty() {
                self.current = 0;
                return None;
            }
            self.current = self.current.min(matches.len() - 1);
            return Some(matches[self.current].clone());
        }

        if matches.is_empty() {
            return None;
        }

        if go_next {
            self.current = (self.current + 1) % matches.len();
            return Some(matches[self.current].clone());
        }

        if go_prev {
            self.current = (self.current + matches.len() - 1) % matches.len();
            return Some(matches[self.current].clone());
        }

        if query_changed || just_opened {
            return Some(matches[self.current].clone());
        }

        None
    }
}

pub fn paint_highlights(
    ui: &egui::Ui,
    output: &egui::widgets::text_edit::TextEditOutput,
    text: &str,
    matches: &[Range<usize>],
    current: usize,
    theme: ColorTheme,
) {
    if matches.is_empty() {
        return;
    }

    let painter = ui.painter().with_clip_rect(output.text_clip_rect);
    let fill = with_alpha(theme.selection(), 90);
    let current_stroke = egui::Stroke::new(1.5, theme.cursor());

    for (i, range) in matches.iter().enumerate() {
        let start = egui::text::CCursor::new(char_index(text, range.start));
        let end = egui::text::CCursor::new(char_index(text, range.end));

        let start_rect = output
            .galley
            .pos_from_cursor(start)
            .translate(output.galley_pos.to_vec2());
        let end_rect = output
            .galley
            .pos_from_cursor(end)
            .translate(output.galley_pos.to_vec2());

        let rect = egui::Rect::from_min_max(
            start_rect.min,
            egui::pos2(end_rect.max.x, start_rect.max.y),
        );

        painter.rect_filled(rect, 2.0, fill);
        if i == current {
            painter.rect_stroke(rect, 2.0, current_stroke, egui::StrokeKind::Outside);
        }
    }
}

fn with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn char_index(text: &str, byte_idx: usize) -> usize {
    text[..byte_idx].chars().count()
}

fn find_matches(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }

    let haystack: Cow<str> = if case_sensitive {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.to_ascii_lowercase())
    };
    let needle: Cow<str> = if case_sensitive {
        Cow::Borrowed(query)
    } else {
        Cow::Owned(query.to_ascii_lowercase())
    };

    let mut matches = Vec::new();
    let mut search_from = 0;
    while let Some(offset) = haystack[search_from..].find(needle.as_ref()) {
        let start = search_from + offset;
        let end = start + needle.len();

        if !whole_word || is_whole_word_match(text, start, end) {
            matches.push(start..end);
        }

        search_from = end.max(start + 1);
    }

    matches
}

fn is_whole_word_match(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !is_word_char(c));
    let after_ok = text[end..].chars().next().is_none_or(|c| !is_word_char(c));
    before_ok && after_ok
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
