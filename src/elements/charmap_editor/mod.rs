mod atlas_view;
mod char_grid;
mod glyph_editor;
mod palette_grid;
mod stamp_screen;

use crate::State;
use crate::elements::ViewState;
use crate::resources::charmap::Charmap;
use egui_dock::egui;

struct AtlasTexture {
    handle: egui::TextureHandle,
    atlas_w: usize,
    atlas_h: usize,
}

/* Charmap Editor based on
 * https://github.com/ICMC-IDE/icmc-ide/blob/main/src/scripts/elements/screen-editor.ts */
pub struct CharmapEditor {
    current_char: usize,
    current_color: usize,
    scratch: Vec<(u8, u8)>,
    scratch_cols: usize,
    scratch_rows: usize,
    atlas: Option<AtlasTexture>,
    glyph_drag_last: Option<(usize, usize)>,
}

impl Default for CharmapEditor {
    fn default() -> Self {
        Self {
            current_char: 65,
            current_color: 0,
            scratch: Vec::new(),
            scratch_cols: 0,
            scratch_rows: 0,
            atlas: None,
            glyph_drag_last: None,
        }
    }
}

impl CharmapEditor {
    fn ensure_atlas(&mut self, ctx: &egui::Context, state: &mut State) {
        let rebuild = self.atlas.is_none() || state.settings.charmap.needs_reload_editor;
        if !rebuild {
            return;
        }

        let charmap = &state.settings.charmap;
        let atlas_w = charmap.char_width() * charmap.num_colors();
        let atlas_h = charmap.char_height() * charmap.num_chars();
        let image = egui::ColorImage::from_rgba_unmultiplied([atlas_w, atlas_h], charmap.pixels());
        let handle = ctx.load_texture("charmap_atlas", image, egui::TextureOptions::NEAREST);

        self.atlas = Some(AtlasTexture {
            handle,
            atlas_w,
            atlas_h,
        });
        state.settings.charmap.needs_reload_editor = false;
    }

    fn sync_scratch_size(&mut self, screen_size: (u32, u32)) {
        let (cols, rows) = (screen_size.0 as usize, screen_size.1 as usize);
        if cols != self.scratch_cols || rows != self.scratch_rows {
            self.scratch = vec![(0, 0); cols * rows];
            self.scratch_cols = cols;
            self.scratch_rows = rows;
        }
    }
}

impl ViewState for CharmapEditor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.horizontal(|ui| {
            if ui.button("Import").clicked() {
                #[cfg(target_arch = "wasm32")]
                {
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(cm) = std::panic::catch_unwind(|| {
                            std::fs::read_to_string(path)
                                .ok()
                                .and_then(|s| mif::parser::parse_mif(&s))
                                .and_then(|parsed| {
                                    Some(Charmap::from_bytes(8, 8, 30, 40, parsed))
                                })
                                .unwrap_or_else(Charmap::default)
                        }) {
                            state.settings.charmap = cm;

                            let num_chars = state.settings.charmap.num_chars();
                            let num_colors = state.settings.charmap.num_colors();
                            if self.current_char >= num_chars {
                                self.current_char = 0;
                            }
                            if self.current_color >= num_colors {
                                self.current_color = 0;
                            }
                            self.atlas = None;
                        };
                    }
                }
            }

            if ui.button("Export").clicked() {
                #[cfg(target_arch = "wasm32")]
                {
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        let mif = mif::Mif::new(
                            state.settings.charmap.bytes(),
                            mif::Radix::Uns,
                            mif::Radix::Bin,
                        );

                        if let Err(e) = std::fs::write(path, format!("{}", mif)) {
                            if let Ok(mut log_panel) = state.log_panel.lock() {
                                log_panel.add_log(format!("Failed to export charmap: {e}"));
                            }
                        }
                    }
                }
            }
        });

        ui.separator();

        self.sync_scratch_size(state.settings.screen_size);
        self.ensure_atlas(ui.ctx(), state);

        let Some(atlas) = &self.atlas else { return };
        let atlas_ref = atlas_view::AtlasRef {
            texture_id: atlas.handle.id(),
            w: atlas.atlas_w,
            h: atlas.atlas_h,
        };

        let spacing = ui.spacing().item_spacing.x;

        let char_grid_size = char_grid::size(&state.settings.charmap);
        let glyph_size = glyph_editor::size();
        let screen_size = stamp_screen::size((self.scratch_cols, self.scratch_rows));
        let palette_size = palette_grid::size(&state.settings.charmap);

        let min_width = char_grid_size
            .x
            .max(glyph_size.x)
            .max(screen_size.x)
            .max(palette_size.x)
            + 16.0;
        ui.set_min_width(min_width);

        let available = ui.available_width();
        let row1_narrow = available < char_grid_size.x + spacing + glyph_size.x;

        let show_chars = |ui: &mut egui::Ui, editor: &mut Self, state: &mut State| {
            ui.label("Characters");
            char_grid::show(
                ui,
                &state.settings.charmap,
                atlas_ref,
                editor.current_color,
                &mut editor.current_char,
            );
        };
        let show_glyph = |ui: &mut egui::Ui, editor: &mut Self, state: &mut State| {
            ui.label("Current Glyph");
            let toggled = glyph_editor::show(
                ui,
                &state.settings.charmap,
                atlas_ref,
                editor.current_char,
                editor.current_color,
                &mut editor.glyph_drag_last,
            );
            if let Some((x, y)) = toggled {
                state.settings.charmap.toggle_pixel(x, y);
            }
        };

        if row1_narrow {
            ui.vertical(|ui| {
                show_chars(ui, self, state);
                ui.add_space(4.0);
                show_glyph(ui, self, state);
            });
        } else {
            ui.horizontal(|ui| {
                ui.vertical(|ui| show_chars(ui, self, state));
                ui.vertical(|ui| show_glyph(ui, self, state));
            });
        }

        ui.separator();

        let row2_narrow = available < screen_size.x + spacing + palette_size.x;

        let show_screen = |ui: &mut egui::Ui, editor: &mut Self, state: &mut State| {
            ui.label("Screen (scratch)");
            stamp_screen::show(
                ui,
                &state.settings.charmap,
                atlas_ref,
                &mut editor.scratch,
                (editor.scratch_cols, editor.scratch_rows),
                (editor.current_char, editor.current_color),
            );
        };
        let show_palette = |ui: &mut egui::Ui, editor: &mut Self, state: &mut State| {
            ui.label("Palette");
            palette_grid::show(ui, &state.settings.charmap, &mut editor.current_color);
        };

        if row2_narrow {
            ui.vertical(|ui| {
                show_screen(ui, self, state);
                ui.add_space(4.0);
                show_palette(ui, self, state);
            });
        } else {
            ui.horizontal(|ui| {
                ui.vertical(|ui| show_screen(ui, self, state));
                ui.vertical(|ui| show_palette(ui, self, state));
            });
        }
    }
}
