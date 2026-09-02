use super::atlas_view::{AtlasRef, cell_at, paint_cell};
use crate::resources::charmap::Charmap;
use egui_dock::egui;

const CELL_SIZE: f32 = 10.0;

pub fn size(dims: (usize, usize)) -> egui::Vec2 {
    egui::vec2(dims.0 as f32 * CELL_SIZE, dims.1 as f32 * CELL_SIZE)
}

pub fn show(
    ui: &mut egui::Ui,
    charmap: &Charmap,
    atlas: AtlasRef,
    scratch: &mut [(u8, u8)],
    size: (usize, usize),
    current: (usize, usize),
) {
    let (cols, rows) = size;
    let (current_char, current_color) = current;
    if cols == 0 || rows == 0 {
        return;
    }

    let px_size = egui::vec2(cols as f32 * CELL_SIZE, rows as f32 * CELL_SIZE);
    let (rect, response) = ui.allocate_exact_size(px_size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    for row in 0..rows {
        for col in 0..cols {
            let (char_index, color_index) = scratch[row * cols + col];
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
                char_index as usize,
                color_index as usize,
            );
        }
    }

    if (response.clicked() || response.dragged())
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((col, row)) = cell_at(rect, pos, cols, rows)
    {
        scratch[row * cols + col] = (current_char as u8, current_color as u8);
    }
}
