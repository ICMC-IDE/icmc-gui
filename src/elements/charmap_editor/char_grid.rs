use super::atlas_view::{AtlasRef, cell_at, paint_cell};
use crate::resources::charmap::Charmap;
use egui_dock::egui;

const COLS: usize = 32;
const CELL_SIZE: f32 = 16.0;

pub fn size(charmap: &Charmap) -> egui::Vec2 {
    let rows = charmap.num_chars().div_ceil(COLS);
    egui::vec2(COLS as f32 * CELL_SIZE, rows as f32 * CELL_SIZE)
}

pub fn show(
    ui: &mut egui::Ui,
    charmap: &Charmap,
    atlas: AtlasRef,
    current_color: usize,
    current_char: &mut usize,
) {
    let num_chars = charmap.num_chars();
    let rows = num_chars.div_ceil(COLS);
    let size = egui::vec2(COLS as f32 * CELL_SIZE, rows as f32 * CELL_SIZE);

    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    for char_index in 0..num_chars {
        let col = char_index % COLS;
        let row = char_index / COLS;
        let cell_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(col as f32 * CELL_SIZE, row as f32 * CELL_SIZE),
            egui::vec2(CELL_SIZE, CELL_SIZE),
        );

        paint_cell(
            &painter,
            atlas,
            charmap.char_width(),
            charmap.char_height(),
            cell_rect,
            char_index,
            current_color,
        );

        if char_index == *current_char {
            painter.rect_stroke(
                cell_rect,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::YELLOW),
                egui::StrokeKind::Inside,
            );
        }
    }

    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((col, row)) = cell_at(rect, pos, COLS, rows)
    {
        let index = row * COLS + col;
        if index < num_chars {
            *current_char = index;
        }
    }
}
