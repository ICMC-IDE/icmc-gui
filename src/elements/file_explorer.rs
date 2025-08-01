use super::egui;
use std::fs;
use std::path::PathBuf;

pub struct FileExplorer {
    pub current_path: PathBuf,
    pub entries: Vec<fs::DirEntry>,
}

impl FileExplorer {
    pub fn new() -> Self {
        let start_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let entries = Self::read_dir(&start_path);
        Self {
            current_path: start_path,
            entries,
        }
    }

    fn read_dir(path: &PathBuf) -> Vec<fs::DirEntry> {
        fs::read_dir(path)
            .map(|read_dir| read_dir.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_else(|_| vec![])
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let paths: Vec<std::path::PathBuf> = self.entries.iter().map(|e| e.path()).collect();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for path in paths {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if path.is_dir() {
                    if ui.selectable_label(false, format!("📂 {}", name)).clicked() {
                        self.current_path = path.clone();
                        self.entries = Self::read_dir(&self.current_path);
                    }
                } else {
                    ui.label(format!("📄 {}", name));
                }
            }
        });

        if self.current_path.parent().is_some() {
            if ui.button("⬅ Voltar").clicked() {
                self.current_path = self.current_path.parent().unwrap().to_path_buf();
                self.entries = Self::read_dir(&self.current_path);
            }
        }
    }
}
