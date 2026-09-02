use super::atlas_view::{AtlasRef, cell_at, paint_cell};
use crate::resources::charmap::Charmap;
use egui_dock::egui;

const SIZE: f32 = 256.0;

pub fn size() -> egui::Vec2 {
    egui::vec2(SIZE, SIZE)
}

pub fn show(
    ui: &mut egui::Ui,
    charmap: &Charmap,
    atlas: AtlasRef,
    current_char: usize,
    current_color: usize,
    drag_last_cell: &mut Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(SIZE, SIZE), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    paint_cell(
        &painter,
        atlas,
        charmap.char_width(),
        charmap.char_height(),
        rect,
        current_char,
        current_color,
    );
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::GRAY),
        egui::StrokeKind::Outside,
    );

    let pointer_cell = |response: &egui::Response| {
        response
            .interact_pointer_pos()
            .and_then(|pos| cell_at(rect, pos, charmap.char_width(), charmap.char_height()))
    };

    let mut cell_to_toggle = None;

    if response.drag_started() || response.clicked() {
        cell_to_toggle = pointer_cell(&response);
    } else if response.dragged()
        && let Some(cell) = pointer_cell(&response)
        && Some(cell) != *drag_last_cell
    {
        cell_to_toggle = Some(cell);
    }

    if response.drag_stopped() {
        *drag_last_cell = None;
    }

    if let Some(cell) = cell_to_toggle {
        *drag_last_cell = Some(cell);
        let (local_x, local_y) = cell;
        let global_y = charmap.char_height() * current_char + local_y;
        return Some((local_x, global_y));
    }

    None
}
