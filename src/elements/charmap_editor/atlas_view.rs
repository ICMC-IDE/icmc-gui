use egui_dock::egui;

#[derive(Clone, Copy)]
pub struct AtlasRef {
    pub texture_id: egui::TextureId,
    pub w: usize,
    pub h: usize,
}

fn glyph_uv(
    cw: usize,
    ch: usize,
    atlas: AtlasRef,
    char_index: usize,
    color_index: usize,
) -> egui::Rect {
    let x0 = (color_index * cw) as f32 / atlas.w as f32;
    let y0 = (char_index * ch) as f32 / atlas.h as f32;

    egui::Rect::from_min_size(
        egui::pos2(x0, y0),
        egui::vec2(cw as f32 / atlas.w as f32, ch as f32 / atlas.h as f32),
    )
}

pub fn paint_cell(
    painter: &egui::Painter,
    atlas: AtlasRef,
    cw: usize,
    ch: usize,
    screen_rect: egui::Rect,
    char_index: usize,
    color_index: usize,
) {
    let uv = glyph_uv(cw, ch, atlas, char_index, color_index);
    let mut mesh = egui::Mesh::with_texture(atlas.texture_id);
    mesh.add_rect_with_uv(screen_rect, uv, egui::Color32::WHITE);
    painter.add(egui::Shape::mesh(mesh));
}

pub fn cell_at(
    rect: egui::Rect,
    pos: egui::Pos2,
    cols: usize,
    rows: usize,
) -> Option<(usize, usize)> {
    if cols == 0 || rows == 0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }

    let local = pos - rect.min;
    let cell_w = rect.width() / cols as f32;
    let cell_h = rect.height() / rows as f32;

    let col = ((local.x / cell_w) as usize).min(cols - 1);
    let row = ((local.y / cell_h) as usize).min(rows - 1);

    Some((col, row))
}
