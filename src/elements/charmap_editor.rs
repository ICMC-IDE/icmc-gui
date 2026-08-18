use crate::State;
use crate::elements::ViewState;
use crate::resources::charmap::Charmap;

#[derive(Default)]
pub struct CharmapEditor {}

impl ViewState for CharmapEditor {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut State,
        _ctx: &mut egui::Context,
    ) {
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
                                    Some(Charmap::from_bytes(
                                        8, 8, 30, 40, parsed,
                                    ))
                                })
                                .unwrap_or_else(Charmap::default)
                        }) {
                            state.settings.charmap = cm;
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

                        std::fs::write(path, format!("{}", mif))
                            .expect("Can't write to the workspace directory");
                    }
                }
            }
        });
    }
}
