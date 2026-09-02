use super::atlas_view::cell_at;
use crate::resources::charmap::Charmap;
use egui_dock::egui;

const COLS: usize = 8;
const CELL_SIZE: f32 = 14.0;

pub fn size(charmap: &Charmap) -> egui::Vec2 {
    let rows = charmap.num_colors().div_ceil(COLS);
    egui::vec2(COLS as f32 * CELL_SIZE, rows as f32 * CELL_SIZE)
}

pub fn show(ui: &mut egui::Ui, charmap: &Charmap, current_color: &mut usize) {
    let num_colors = charmap.num_colors();
    let rows = num_colors.div_ceil(COLS);
    let size = egui::vec2(COLS as f32 * CELL_SIZE, rows as f32 * CELL_SIZE);

    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    for i in 0..num_colors {
        let col = i % COLS;
        let row = i / COLS;
        let cell_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(col as f32 * CELL_SIZE, row as f32 * CELL_SIZE),
            egui::vec2(CELL_SIZE, CELL_SIZE),
        );

        let [r, g, b, a] = charmap.palette_rgba(i);
        painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgba_unmultiplied(r, g, b, a));

        if i == *current_color {
            painter.rect_stroke(
                cell_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }
    }

    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((col, row)) = cell_at(rect, pos, COLS, rows)
    {
        let index = row * COLS + col;
        if index < num_colors {
            *current_color = index;
        }
    }
}
