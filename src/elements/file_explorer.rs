use super::ViewState;
use crate::State;
use egui_dock::egui;
use egui_material_icons::MaterialIcon;
use egui_material_icons::icons::{
    ICON_CHEVRON_RIGHT, ICON_CODE, ICON_DESCRIPTION, ICON_EXPAND_MORE, ICON_FOLDER,
    ICON_FOLDER_OPEN,
};
use std::fs;
use std::path::{Path, PathBuf};

const CHEVRON_WIDTH: f32 = 16.0;
const INDENT_WIDTH: f32 = 16.0;

struct Entry {
    path: PathBuf,
    name: String,
    is_dir: bool,
    expanded: bool,
    children: Vec<Entry>,
}

impl Entry {
    fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let is_dir = path.is_dir();

        Self {
            path,
            name,
            is_dir,
            expanded: false,
            children: Vec::new(),
        }
    }

    fn sync_children(&mut self) {
        if !self.is_dir {
            return;
        }

        let mut old = std::mem::take(&mut self.children);

        let mut fresh: Vec<Entry> = fs::read_dir(&self.path)
            .map(|read_dir| {
                read_dir
                    .filter_map(|e| e.ok())
                    .map(|e| Entry::new(e.path()))
                    .collect()
            })
            .unwrap_or_default();

        fresh.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        for entry in &mut fresh {
            if let Some(pos) = old.iter().position(|o| o.path == entry.path) {
                let prev = old.remove(pos);
                entry.expanded = prev.expanded;
                entry.children = prev.children;
            }
        }

        self.children = fresh;
    }
}

enum InlineEdit {
    None,
    Renaming {
        path: PathBuf,
        buffer: String,
        focus_requested: bool,
    },
    Creating {
        parent: PathBuf,
        is_dir: bool,
        buffer: String,
        focus_requested: bool,
    },
}

pub struct FileExplorer {
    root: Entry,
    inline_edit: InlineEdit,
    pending_delete: Option<PathBuf>,
}

impl FileExplorer {
    pub fn new(path: Option<PathBuf>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let root_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

        #[cfg(target_arch = "wasm32")]
        let root_path = path.unwrap_or_else(|| PathBuf::from("."));

        let mut root = Entry::new(root_path);
        root.expanded = true;
        root.sync_children();

        Self {
            root,
            inline_edit: InlineEdit::None,
            pending_delete: None,
        }
    }

    pub fn workspace_path(&self) -> &Path {
        &self.root.path
    }

    pub fn set_workspace(&mut self, path: PathBuf) {
        let mut root = Entry::new(path);
        root.expanded = true;
        root.sync_children();

        self.root = root;
        self.inline_edit = InlineEdit::None;
        self.pending_delete = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_file(&mut self, path: &Path, state: &mut State) {
        let dest = self.root.path.join(path.file_name().unwrap_or_default());

        if let Err(e) = std::fs::copy(path, &dest) {
            if let Ok(mut log_panel) = state.log_panel.lock() {
                log_panel.add_log(format!("Couldn't copy file: {e}"));
            }
        } else {
            self.root.sync_children();
        }
    }
}

impl ViewState for FileExplorer {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            show_entry(
                ui,
                &mut self.root,
                0,
                true,
                state,
                &mut self.inline_edit,
                &mut self.pending_delete,
            );
        });
    }
}

fn file_icon(path: &Path) -> MaterialIcon {
    match path.extension().and_then(|e| e.to_str()) {
        Some(
            "rs" | "asm" | "toml" | "py" | "js" | "ts" | "json" | "c" | "cpp" | "h" | "hpp"
                | "go" | "rb" | "java" | "sh" | "md",
        ) => ICON_CODE,
        _ => ICON_DESCRIPTION,
    }
}

fn show_entry(
    ui: &mut egui::Ui,
    entry: &mut Entry,
    depth: usize,
    is_root: bool,
    state: &mut State,
    inline_edit: &mut InlineEdit,
    pending_delete: &mut Option<PathBuf>,
) {
    show_row(ui, entry, depth, is_root, state, inline_edit, pending_delete);

    if entry.is_dir && entry.expanded {
        if let Some(del_path) = pending_delete.clone()
            && entry.children.iter().any(|c| c.path == del_path)
        {
            delete_entry(entry, &del_path, state);
            *pending_delete = None;
        }

        for child in &mut entry.children {
            show_entry(ui, child, depth + 1, false, state, inline_edit, pending_delete);
        }

        show_creating_row(ui, entry, depth + 1, inline_edit);
    }
}

#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
fn delete_entry(parent: &mut Entry, del_path: &Path, state: &mut State) {
    #[cfg(not(target_arch = "wasm32"))]
    match trash::delete(del_path) {
        Ok(()) => {
            parent.children.retain(|c| c.path != del_path);

            if state.settings.open_file.as_deref() == Some(del_path) {
                state.settings.open_file = None;
                *state.code_buf = None;
                *state.binary_file = false;
            }
        }
        Err(e) => {
            if let Ok(mut log_panel) = state.log_panel.lock() {
                log_panel.add_log(format!("Couldn't delete {}: {e}", del_path.display()));
            }
        }
    }
}

fn show_row(
    ui: &mut egui::Ui,
    entry: &mut Entry,
    depth: usize,
    is_root: bool,
    state: &mut State,
    inline_edit: &mut InlineEdit,
    pending_delete: &mut Option<PathBuf>,
) {
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * INDENT_WIDTH);

        if entry.is_dir {
            let chevron = if entry.expanded {
                ICON_EXPAND_MORE
            } else {
                ICON_CHEVRON_RIGHT
            };

            if ui
                .add(egui::Button::new(chevron.rich_text()).frame(false))
                .clicked()
            {
                entry.expanded = !entry.expanded;
                if entry.expanded {
                    entry.sync_children();
                }
            }
        } else {
            ui.add_space(CHEVRON_WIDTH);
        }

        let is_renaming =
            matches!(inline_edit, InlineEdit::Renaming { path, .. } if *path == entry.path);

        let icon = if entry.is_dir {
            if entry.expanded {
                ICON_FOLDER_OPEN
            } else {
                ICON_FOLDER
            }
        } else {
            file_icon(&entry.path)
        };
        ui.label(icon.rich_text());

        if is_renaming {
            show_rename_field(ui, entry, state, inline_edit);
        } else {
            let is_selected =
                !entry.is_dir && state.settings.open_file.as_deref() == Some(entry.path.as_path());

            let response = ui.selectable_label(is_selected, &entry.name);

            if response.clicked() {
                if entry.is_dir {
                    entry.expanded = !entry.expanded;
                    if entry.expanded {
                        entry.sync_children();
                    }
                } else {
                    state.settings.open_file = Some(
                        entry
                            .path
                            .canonicalize()
                            .unwrap_or_else(|_| entry.path.clone()),
                    );

                    match fs::read_to_string(&entry.path) {
                        Ok(content) => {
                            *state.code_buf = Some(content);
                            *state.binary_file = false;
                        }
                        Err(_) => {
                            *state.code_buf = None;
                            *state.binary_file = true;
                        }
                    }
                }
            }

            attach_context_menu(&response, entry, is_root, inline_edit, pending_delete);
        }
    });
}

fn show_rename_field(
    ui: &mut egui::Ui,
    entry: &mut Entry,
    state: &mut State,
    inline_edit: &mut InlineEdit,
) {
    let InlineEdit::Renaming {
        buffer,
        focus_requested,
        ..
    } = inline_edit
    else {
        unreachable!()
    };

    let response = ui.add(
        egui::TextEdit::singleline(buffer)
            .desired_width(160.0)
            .return_key(None),
    );

    let just_requested = !*focus_requested;
    if just_requested {
        response.request_focus();
        *focus_requested = true;
    }

    let commit = response.has_focus()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    let cancel_key = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let cancel_blur = !just_requested && !commit && !response.has_focus();

    let new_name = buffer.trim().to_string();

    if commit {
        if !new_name.is_empty() && new_name != entry.name {
            let new_path = entry.path.with_file_name(&new_name);

            if fs::rename(&entry.path, &new_path).is_ok() {
                if state.settings.open_file.as_deref() == Some(entry.path.as_path()) {
                    state.settings.open_file = Some(new_path.clone());
                }
                entry.path = new_path;
                entry.name = new_name;
            } else if let Ok(mut log_panel) = state.log_panel.lock() {
                log_panel.add_log(format!("Couldn't rename {}", entry.name));
            }
        }
        *inline_edit = InlineEdit::None;
    } else if cancel_key || cancel_blur {
        *inline_edit = InlineEdit::None;
    }
}

fn show_creating_row(
    ui: &mut egui::Ui,
    parent: &mut Entry,
    depth: usize,
    inline_edit: &mut InlineEdit,
) {
    let is_here =
        matches!(inline_edit, InlineEdit::Creating { parent: p, .. } if *p == parent.path);
    if !is_here {
        return;
    }

    let InlineEdit::Creating {
        is_dir,
        buffer,
        focus_requested,
        ..
    } = inline_edit
    else {
        unreachable!()
    };

    let mut commit = false;
    let mut cancel = false;

    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * INDENT_WIDTH + CHEVRON_WIDTH);

        let icon = if *is_dir { ICON_FOLDER } else { ICON_DESCRIPTION };
        ui.label(icon.rich_text());

        let response = ui.add(
            egui::TextEdit::singleline(buffer)
                .desired_width(160.0)
                .return_key(None),
        );

        let just_requested = !*focus_requested;
        if just_requested {
            response.request_focus();
            *focus_requested = true;
        }

        commit = response.has_focus()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        let cancel_key =
            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let cancel_blur = !just_requested && !commit && !response.has_focus();
        cancel = cancel_key || cancel_blur;
    });

    if commit {
        let name = buffer.trim().to_string();

        if !name.is_empty() {
            let target = parent.path.join(&name);

            let result = if *is_dir {
                fs::create_dir(&target)
            } else {
                fs::File::create(&target).map(|_| ())
            };

            if result.is_ok() {
                parent.sync_children();
            }
        }
        *inline_edit = InlineEdit::None;
    } else if cancel {
        *inline_edit = InlineEdit::None;
    }
}

fn attach_context_menu(
    response: &egui::Response,
    entry: &Entry,
    is_root: bool,
    inline_edit: &mut InlineEdit,
    pending_delete: &mut Option<PathBuf>,
) {
    response.context_menu(|ui| {
        let new_parent = if entry.is_dir {
            entry.path.clone()
        } else {
            entry
                .path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| entry.path.clone())
        };

        if ui.button("New File").clicked() {
            *inline_edit = InlineEdit::Creating {
                parent: new_parent.clone(),
                is_dir: false,
                buffer: String::new(),
                focus_requested: false,
            };
            ui.close();
        }

        if ui.button("New Folder").clicked() {
            *inline_edit = InlineEdit::Creating {
                parent: new_parent,
                is_dir: true,
                buffer: String::new(),
                focus_requested: false,
            };
            ui.close();
        }

        if !is_root {
            ui.separator();

            if ui.button("Rename").clicked() {
                *inline_edit = InlineEdit::Renaming {
                    path: entry.path.clone(),
                    buffer: entry.name.clone(),
                    focus_requested: false,
                };
                ui.close();
            }

            if ui.button("Delete").clicked() {
                *pending_delete = Some(entry.path.clone());
                ui.close();
            }
        }

        ui.separator();

        if ui.button("Copy Path").clicked() {
            ui.ctx().copy_text(entry.path.display().to_string());
            ui.close();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Reveal in File Manager").clicked() {
            let _ = opener::reveal(&entry.path);
            ui.close();
        }
    });
}
