use crate::State;
use egui_dock::egui;

pub mod documentation;
pub mod editor;
pub mod file_explorer;
pub mod log;
pub mod memory_editor;
pub mod screen;
pub mod state;

pub use documentation::Documentation;
pub use editor::Editor;
pub use file_explorer::FileExplorer;
pub use log::LogPanel;
pub use memory_editor::MemEditor;
pub use screen::Screen;
pub use state::StatePanel;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut egui::Context);
}

/* View trait that requires reference to emulator's state */
pub trait ViewState {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State, ctx: &mut egui::Context);
}
