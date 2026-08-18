use super::ViewState;
use crate::State;
use egui_dock::egui;
use std::fs;
use std::path::PathBuf;

pub struct FileExplorer {
    root_path: PathBuf,
    current_path: PathBuf,
    entries: Vec<fs::DirEntry>,
}

impl FileExplorer {
    pub fn new(path: Option<PathBuf>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let root_path =
            path.or(Some(std::env::current_dir().unwrap())).unwrap();

        #[cfg(target_arch = "wasm32")]
        let root_path = path.or(Some(PathBuf::from("."))).unwrap();

        let entries = Self::read_dir(&root_path);

        Self {
            root_path: root_path.clone(),
            current_path: root_path,
            entries,
        }
    }

    fn read_dir(path: &PathBuf) -> Vec<fs::DirEntry> {
        fs::read_dir(path)
            .map(|read_dir| read_dir.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_else(|_| vec![])
    }
}

impl ViewState for FileExplorer {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut State,
        _ctx: &mut egui::Context,
    ) {
        let paths: Vec<std::path::PathBuf> =
            self.entries.iter().map(|e| e.path()).collect();

        ui.horizontal(|ui| {
            if ui.button("New File").clicked() {
                #[cfg(target_arch = "wasm32")]
                {
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(self.current_path.clone())
                        .save_file()
                    {
                        if let Err(e) = std::fs::File::create(path) {
                            eprintln!("Couldn't create file: {}", e);
                        } else {
                            self.entries = Self::read_dir(&self.current_path);
                        }
                    }
                }
            }

            if ui.button("Open File").clicked() {
                #[cfg(target_arch = "wasm32")]
                {
                    wasm_bindgen_futures::spawn_local(async {
                        if let Some(path) =
                            rfd::AsyncFileDialog::new().pick_file().await
                        {
                            let file_name = path.file_name();
                            let data = path.read();
                        }
                    });
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let dest = self
                            .current_path
                            .join(path.file_name().unwrap_or_default());

                        if let Err(e) = std::fs::copy(&path, &dest) {
                            eprintln!("Couldn't copy file: {}", e);
                        } else {
                            self.entries = Self::read_dir(&self.current_path);
                        }
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Change workspace directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.root_path = path.clone();
                    self.current_path = path.clone();
                    self.entries = Self::read_dir(&self.root_path);
                }
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            for path in paths {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if path.is_dir() {
                    if ui
                        .selectable_label(false, format!("📂 {}", name))
                        .clicked()
                    {
                        self.current_path = path.clone();
                        self.entries = Self::read_dir(&self.current_path);
                    }
                } else {
                    let is_selected = state
                        .open_file
                        .as_ref()
                        .map(|p| p == &path)
                        .unwrap_or(false);

                    if is_selected {
                        ui.strong(format!("📄 {}", name));
                    } else if ui
                        .selectable_label(false, format!("📄 {}", name))
                        .clicked()
                    {
                        *state.open_file =
                            Some(path.canonicalize().unwrap_or(path.clone()));
                        *state.code_buf =
                            Some(fs::read_to_string(&path).unwrap());
                    }
                }
            }
        });

        if self.current_path.parent().is_some()
            && self.current_path != self.root_path
        {
            if ui.button("⬅ Back").clicked() {
                self.current_path =
                    self.current_path.parent().unwrap().to_path_buf();
                self.entries = Self::read_dir(&self.current_path);
            }
        }
    }
}
