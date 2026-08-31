use crate::resources::{charmap::Charmap, radix::Radix};
use egui_dock::{DockState, NodeIndex, egui::ThemePreference};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct SettingsInner {
    pub font_size: f32,

    /// Screen size in characters
    pub screen_size: (u32, u32),

    pub radix: Radix,

    #[serde(default)]
    pub theme: ThemePreference,

    #[serde(skip)]
    pub charmap: Charmap,

    /* internal */
    #[serde(skip)]
    pub input_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(flatten)]
    inner: SettingsInner,

    #[serde(skip)]
    pub needs_save: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            inner: SettingsInner {
                font_size: 14.0,
                screen_size: (40, 30),
                radix: Default::default(),
                theme: Default::default(),
                charmap: Default::default(),
                input_enabled: false,
            },
            needs_save: false,
        }
    }
}

fn default_dock_layout() -> DockState<String> {
    let mut tree = DockState::new(vec!["Code Editor".to_owned()]);
    let surface = tree.main_surface_mut();

    let [code_editor, screen] =
        surface.split_left(NodeIndex::root(), 0.3, vec!["Screen".to_owned()]);
    surface.split_below(screen, 0.5, vec!["State".to_owned()]);
    surface.split_below(code_editor, 0.7, vec!["Log".to_owned()]);

    tree
}

pub fn load_dock_layout(ide_path: Option<&Path>) -> DockState<String> {
    ide_path
        .map(|path| path.join(".dockstate"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|toml_str| toml::from_str(&toml_str).ok())
        .unwrap_or_else(default_dock_layout)
}

pub fn save_dock_layout(ide_path: &Path, tree: &DockState<String>) {
    match toml::to_string(tree) {
        Ok(toml_str) => {
            if let Err(e) = std::fs::write(ide_path.join(".dockstate"), toml_str) {
                eprintln!("Couldn't write .dockstate: {e}");
            }
        }
        Err(e) => eprintln!("Couldn't serialize dock layout: {e}"),
    }
}

impl Settings {
    /* try to load past dock setting */
    pub fn load(ide_path: Option<&Path>) -> Self {
        ide_path
            .map(|path| path.join("settings.toml"))
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|toml_str| toml::from_str(&toml_str).ok())
            .unwrap_or_default()
    }

    pub fn clear_save_flag(&mut self) {
        self.needs_save = false;
    }
}

impl Deref for Settings {
    type Target = SettingsInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Settings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.needs_save = true;
        &mut self.inner
    }
}
