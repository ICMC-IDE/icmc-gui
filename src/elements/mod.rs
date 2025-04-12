pub mod editor;

pub use editor::Editor;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}
