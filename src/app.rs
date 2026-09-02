use crate::elements::{
    CharmapEditor, Documentation, Editor, FileExplorer, LogPanel, MemEditor, Screen, StatePanel,
    View, ViewState,
};
use crate::resources::{radix::Radix, settings::Settings};
use egui_dock::dock_state::tree::Split;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, egui};
use icmc_emulator::Emulator;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime;

/* Emulator state */
pub struct State<'a> {
    pub egui_ctx: egui::Context,
    pub emulator: Arc<Mutex<Emulator>>,
    pub freq: Arc<Mutex<f64>>,

    #[cfg(not(target_arch = "wasm32"))]
    pub rt: &'a mut runtime::Runtime,
    #[cfg(not(target_arch = "wasm32"))]
    pub emu_handle: &'a mut Option<tokio::task::JoinHandle<()>>,

    pub running: Arc<AtomicBool>,
    pub code_buf: &'a mut Option<String>,
    pub log_panel: Arc<Mutex<LogPanel>>,
    pub ide_path: &'a mut Option<PathBuf>,
    pub open_file: &'a mut Option<PathBuf>,
    pub settings: &'a mut Settings,
}

impl State<'_> {
    pub fn spawn_run_loop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(handle) = &self.emu_handle {
            handle.abort();
        }

        let freq = Arc::clone(&self.freq);
        let emu = Arc::clone(&self.emulator);
        let running = Arc::clone(&self.running);
        let ctx = self.egui_ctx.clone();

        running.store(true, Ordering::SeqCst);

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{Duration, Instant};

            *self.emu_handle = Some(self.rt.spawn(async move {
                let mut last = Instant::now();
                let mut ticks_pending = 0.0;

                while running.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    let freq_val = *freq.lock().unwrap();
                    ticks_pending =
                        (ticks_pending + (now - last).as_secs_f64() * freq_val).min(1_000_000.0);
                    last = now;

                    {
                        let mut emu = emu.lock().unwrap();
                        let ticks = emu.tick(ticks_pending as isize);

                        if emu.state() != icmc_emulator::State::Paused {
                            running.store(false, Ordering::SeqCst);
                            break;
                        }

                        ticks_pending -= ticks as f64;
                    }

                    ctx.request_repaint();
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_timers::future::TimeoutFuture;
            use wasm_bindgen_futures::spawn_local;

            fn performance_now() -> f64 {
                web_sys::window().unwrap().performance().unwrap().now()
            }

            spawn_local(async move {
                let mut last = performance_now();
                let mut ticks_pending = 0.0;

                while running.load(Ordering::SeqCst) {
                    let now = performance_now();
                    let freq_val = *freq.lock().unwrap();
                    ticks_pending =
                        (ticks_pending + (now - last) * 1e-3 * freq_val).min(1_000_000.0);
                    last = now;

                    {
                        let mut emu = emu.lock().unwrap();
                        let ticks = emu.tick(ticks_pending as isize);

                        if emu.state() != icmc_emulator::State::Paused {
                            running.store(false, Ordering::SeqCst);
                            break;
                        }

                        ticks_pending -= ticks as f64;
                    }

                    ctx.request_repaint();
                    TimeoutFuture::new(0).await;
                }
            });
        }
    }

    pub fn save_file(&mut self) {
        #[cfg(target_family = "wasm")]
        {
            todo!("Need to implement JS wrapper to fs.js");
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let open_file = match self.open_file {
                Some(f) => f.to_str().unwrap(),
                &mut None => todo!(),
            };

            let code_buf = self
                .code_buf
                .get_or_insert_with(|| include_str!("../res/example.asm").to_owned());

            if let Err(e) = std::fs::write(open_file, code_buf.as_bytes()) {
                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.add_log(format!("Failed to write .code.asm: {}", e));
                }
            }
        }
    }

    pub fn build_and_run(&mut self) {
        let icmc_syntax = include_str!("../res/icmc.toml");

        if let Ok(mut log_panel) = self.log_panel.lock() {
            log_panel.auto_scroll();
        }

        let code_buf = self
            .code_buf
            .get_or_insert_with(|| include_str!("../res/example.asm").to_owned());

        match assembler::assemble_from_buf(code_buf.as_str(), icmc_syntax) {
            Ok(asm) => {
                self.emulator.lock().unwrap().load_program(&asm.binary());
                self.spawn_run_loop();

                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.clear_logs();
                    log_panel.add_log(format!("Assembly successful"));
                }
            }
            Err(err) => {
                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.clear_logs();
                    log_panel.add_log(format!("Error: {}", err));
                }
            }
        };
    }

    pub fn clear_code_buffer(&mut self) {
        if let Some(buf) = self.code_buf.as_mut() {
            buf.clear();
        }
    }
}

/* Tab manager */
pub struct TabViewer<'a> {
    charmap_editor: &'a mut CharmapEditor,
    editor: &'a mut Editor,
    doc: &'a mut Documentation,
    screen: &'a mut Screen,
    state_panel: &'a mut StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: &'a mut FileExplorer,
    mem_editor: Arc<Mutex<MemEditor>>,

    open_tabs: &'a mut HashSet<String>,
    state: &'a mut State<'a>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.as_str())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Charmap Editor" => self.charmap_editor.ui(ui, self.state),
            "Screen" => self.screen.ui(ui, self.state),
            "State" => self.state_panel.ui(ui, self.state),
            "Code Editor" => self.editor.ui(ui, self.state),
            "File Explorer" => self.file_explorer.ui(ui, self.state),
            "Documentation" => self.doc.ui(ui),

            "Log" => {
                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.ui(ui, self.state);
                }
            }

            "Memory Editor" => {
                if let Ok(mut mem_editor) = self.mem_editor.lock() {
                    mem_editor.ui(ui, self.state);
                }
            }

            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> OnCloseResponse {
        self.open_tabs.remove(tab);
        OnCloseResponse::Close
    }
}

const PANEL_NAMES: [&str; 8] = [
    "Code Editor",
    "Screen",
    "State",
    "Log",
    "File Explorer",
    "Documentation",
    "Memory Editor",
    "Charmap Editor",
];

fn find_node_index(tree: &DockState<String>, name: &str) -> Option<NodeIndex> {
    let surface = &tree[SurfaceIndex::main()];

    for (i, node) in surface.iter().enumerate() {
        if let Some(tabs) = node.tabs() {
            if tabs.iter().any(|t| t == name) {
                return Some(NodeIndex::from(i));
            }
        }
    }
    None
}

fn toggle_panel(
    tree: &mut DockState<String>,
    open_tabs: &mut HashSet<String>,
    ide_path: &Option<PathBuf>,
    name: &str,
) {
    if let Some(idx) = tree.find_tab(&name.to_string()) {
        tree.remove_tab(idx);
        open_tabs.remove(name);
    } else {
        if name == "Documentation" || name == "Charmap Editor" {
            tree.add_window(vec![name.to_string()]);
        } else {
            let surf = tree.main_surface_mut();
            let empty_root = surf
                .iter()
                .all(|node| node.tabs().map_or(true, |t| t.is_empty()));

            if empty_root {
                surf.push_to_focused_leaf(name.to_string());
            } else {
                let (parent, fraction, split) = match name {
                    "Log" => (
                        find_node_index(tree, "Code Editor").unwrap_or(NodeIndex::root()),
                        0.7,
                        Split::Below,
                    ),
                    "Memory Editor" => (
                        find_node_index(tree, "Code Editor").unwrap_or(NodeIndex::root()),
                        0.6,
                        Split::Right,
                    ),
                    "File Explorer" => (
                        find_node_index(tree, "Code Editor").unwrap_or(NodeIndex::root()),
                        0.8,
                        Split::Right,
                    ),
                    _ => (NodeIndex::root(), 0.8, Split::Right),
                };

                tree.main_surface_mut().split(
                    parent,
                    split,
                    fraction,
                    egui_dock::Node::leaf(name.to_string()),
                );
            }
        }
        open_tabs.insert(name.to_string());
    }

    if let Some(path) = ide_path {
        crate::resources::settings::save_dock_layout(path, tree);
    }
}

/// A top-level menu-bar button that, once any sibling menu is open, also opens on hover
/// (instead of requiring another click) -- matching typical desktop app menu bars.
fn top_menu_button<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    let inner = ui.menu_button(title, add_contents);

    let ctx = ui.ctx();
    let popup_id = egui::Popup::default_response_id(&inner.response);

    if inner.response.hovered()
        && egui::Popup::is_any_open(ctx)
        && !egui::Popup::is_id_open(ctx, popup_id)
    {
        egui::Popup::open_id(ctx, popup_id);
    }

    inner
}

fn menu_bar(
    ui: &mut egui::Ui,
    tree: &mut DockState<String>,
    open_tabs: &mut HashSet<String>,
    editor: &mut Editor,
    state: &mut State,
    show_about: &mut bool,
) {
    let save_shortcut = egui::KeyboardShortcut::new(
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        egui::Key::S,
    );
    let find_shortcut = egui::KeyboardShortcut::new(
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        egui::Key::F,
    );
    let replace_shortcut = egui::KeyboardShortcut::new(
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        egui::Key::H,
    );
    let goto_shortcut = egui::KeyboardShortcut::new(
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        egui::Key::G,
    );
    let build_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F5);

    if ui.input_mut(|i| i.consume_shortcut(&build_shortcut)) {
        state.build_and_run();
    }

    ui.add_space(2.0);
    ui.horizontal(|ui| {
        /* square off the top-level bar buttons, desktop-app style */
        let widgets = &mut ui.style_mut().visuals.widgets;
        widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
        widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
        widgets.active.corner_radius = egui::CornerRadius::ZERO;
        widgets.open.corner_radius = egui::CornerRadius::ZERO;

        egui::widgets::global_theme_preference_switch(ui);

        let theme = ui.ctx().options(|o| o.theme_preference);
        if theme != state.settings.theme {
            state.settings.theme = theme;
        }

        top_menu_button(ui, "File", |ui| {
            if ui
                .add(
                    egui::Button::new("Save")
                        .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                )
                .clicked()
            {
                state.save_file();
                ui.close();
            }
        });

        top_menu_button(ui, "Edit", |ui| {
            if ui.button("Clear Editor").clicked() {
                state.clear_code_buffer();
                ui.close();
            }

            ui.separator();

            if ui
                .add(
                    egui::Button::new("Find")
                        .shortcut_text(ui.ctx().format_shortcut(&find_shortcut)),
                )
                .clicked()
            {
                editor.open_find();
                ui.close();
            }

            if ui
                .add(
                    egui::Button::new("Replace")
                        .shortcut_text(ui.ctx().format_shortcut(&replace_shortcut)),
                )
                .clicked()
            {
                editor.open_replace();
                ui.close();
            }

            if ui
                .add(
                    egui::Button::new("Go to Line")
                        .shortcut_text(ui.ctx().format_shortcut(&goto_shortcut)),
                )
                .clicked()
            {
                editor.activate_goto();
                ui.close();
            }
        });

        top_menu_button(ui, "View", |ui| {
            ui.menu_button("Radix", |ui| {
                ui.selectable_value(&mut state.settings.radix, Radix::Binary, "Binary");
                ui.selectable_value(&mut state.settings.radix, Radix::Decimal, "Decimal");
                ui.selectable_value(&mut state.settings.radix, Radix::Hex, "Hexadecimal");
                ui.selectable_value(&mut state.settings.radix, Radix::Octal, "Octal");
            });

            ui.menu_button("Font Size", |ui| {
                if ui.button("Reset font size").clicked() {
                    state.settings.font_size = 14.0;
                }

                ui.horizontal(|ui| {
                    if ui.button("-").clicked() && state.settings.font_size >= 4.0 {
                        state.settings.font_size -= 2.0;
                    }

                    ui.label(format!("{} pt", state.settings.font_size));

                    if ui.button("+").clicked() && state.settings.font_size <= 64.0 {
                        state.settings.font_size += 2.0;
                    }
                });
            });

            ui.separator();

            for name in PANEL_NAMES {
                if cfg!(target_arch = "wasm32")
                    && (name == "File Explorer" || name == "Charmap Editor")
                {
                    continue;
                }

                let mut is_open = open_tabs.contains(name);
                if ui.checkbox(&mut is_open, name).changed() {
                    toggle_panel(tree, open_tabs, state.ide_path, name);
                }
            }
        });

        top_menu_button(ui, "Build", |ui| {
            if ui
                .add(
                    egui::Button::new("Build and Run")
                        .shortcut_text(ui.ctx().format_shortcut(&build_shortcut)),
                )
                .clicked()
            {
                state.build_and_run();
                ui.close();
            }
        });

        top_menu_button(ui, "Help", |ui| {
            let mut doc_open = open_tabs.contains("Documentation");
            if ui.checkbox(&mut doc_open, "Documentation").changed() {
                toggle_panel(tree, open_tabs, state.ide_path, "Documentation");
            }

            ui.separator();

            if ui.button("About").clicked() {
                *show_about = true;
                ui.close();
            }
        });
    });
    ui.add_space(2.0);
}

/* Main app */
pub struct IdeApp {
    /* Tab/Dock related */
    tree: DockState<String>,
    open_tabs: HashSet<String>,

    /* Core */
    egui_ctx: egui::Context,
    emulator: Arc<Mutex<Emulator>>, /* Emulator backend*/
    freq: Arc<Mutex<f64>>,          /* Emulator running frequency */

    #[cfg(not(target_arch = "wasm32"))]
    emu_handle: Option<tokio::task::JoinHandle<()>>, /* Emulator thread handle */
    #[cfg(not(target_arch = "wasm32"))]
    rt: runtime::Runtime, /* Tokio runtime */

    running: Arc<AtomicBool>, /* Emulator thread status */

    code_buf: Option<String>,
    ide_path: Option<PathBuf>,
    open_file: Option<PathBuf>,

    settings: Settings,
    show_about: bool,

    /* Elements */
    charmap_editor: CharmapEditor,
    editor: Editor,
    doc: Documentation,
    screen: Screen,
    state_panel: StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: FileExplorer,
    mem_editor: Arc<Mutex<MemEditor>>,
}

impl IdeApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        ide_path: Option<PathBuf>,
        settings: Settings,
    ) -> Self {
        /* supress assembler panics */
        std::panic::set_hook(Box::new(|_info| {}));

        let emulator = Arc::new(Mutex::new(icmc_emulator::Emulator::new()));

        #[cfg(not(target_arch = "wasm32"))]
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let freq = Arc::new(Mutex::new(1.0));
        #[cfg(not(target_arch = "wasm32"))]
        let emu_handle = None;
        let running = Arc::new(AtomicBool::new(false));

        let root_path = ide_path
            .as_ref()
            .map(|path| PathBuf::from(path).join("workspace"));

        let example_path = root_path
            .as_ref()
            .map(|root| root.join("example.asm"))
            .unwrap_or_else(|| PathBuf::from("example.asm"));

        cc.egui_ctx.set_theme(settings.theme);

        let tree = crate::resources::settings::load_dock_layout(ide_path.as_deref());
        let open_tabs: HashSet<String> = tree.iter_all_tabs().map(|(_, tab)| tab.clone()).collect();

        if let Err(e) = std::fs::write(
            example_path.to_str().unwrap(),
            include_str!("../res/example.asm").to_owned().as_bytes(),
        ) {
            eprintln!("Couldn't write example.asm to workspace directory: {e}");
        }

        let charmap = settings.charmap.clone();

        Self {
            tree,
            open_tabs,

            egui_ctx: cc.egui_ctx.clone(),
            emulator,

            #[cfg(not(target_arch = "wasm32"))]
            rt,
            #[cfg(not(target_arch = "wasm32"))]
            emu_handle,

            freq,
            running,

            code_buf: None,
            ide_path: ide_path.clone(),
            open_file: Some(example_path),

            settings,
            show_about: false,

            charmap_editor: CharmapEditor::default(),
            editor: Editor::default(),
            doc: Documentation::default(),
            screen: Screen::new(cc, &charmap),
            state_panel: StatePanel::default(),
            log_panel: Arc::new(Mutex::new(LogPanel::default())),
            mem_editor: Arc::new(Mutex::new(MemEditor::default())),
            file_explorer: FileExplorer::new(root_path),
        }
    }

    fn save_settings_if_needed(&mut self) {
        if !self.settings.needs_save {
            return;
        }

        let toml = toml::to_string(&self.settings).unwrap();

        if let Some(path) = self.ide_path.clone() {
            if std::fs::write(format!("{}/settings.toml", path.display()), toml).is_err() {
                println!("Couldn't write settings.toml");
            }
        } else {
            println!("Couldn't find path");
        }

        self.settings.clear_save_flag();
    }
}

impl eframe::App for IdeApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.open_tabs.is_empty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.save_settings_if_needed();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut state = State {
            egui_ctx: self.egui_ctx.clone(),
            emulator: self.emulator.clone(),
            freq: self.freq.clone(),

            #[cfg(not(target_arch = "wasm32"))]
            emu_handle: &mut self.emu_handle,
            #[cfg(not(target_arch = "wasm32"))]
            rt: &mut self.rt,

            running: self.running.clone(),
            code_buf: &mut self.code_buf,
            log_panel: self.log_panel.clone(),
            ide_path: &mut self.ide_path,
            open_file: &mut self.open_file,
            settings: &mut self.settings,
        };

        /* top menu */
        egui::Panel::top("Top Bar").show(ui, |ui| {
            menu_bar(
                ui,
                &mut self.tree,
                &mut self.open_tabs,
                &mut self.editor,
                &mut state,
                &mut self.show_about,
            );
        });

        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "{} v{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ));
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let focused_tab = self.tree.find_active_focused().map(|(_, tab)| tab.clone());

            if let Some(focused) = &focused_tab {
                state.settings.input_enabled = focused == "Screen";
            }

            let mut tab_viewer = TabViewer {
                charmap_editor: &mut self.charmap_editor,
                editor: &mut self.editor,
                doc: &mut self.doc,
                screen: &mut self.screen,
                state_panel: &mut self.state_panel,
                file_explorer: &mut self.file_explorer,
                open_tabs: &mut self.open_tabs,
                state: &mut state,
                log_panel: self.log_panel.clone(),
                mem_editor: self.mem_editor.clone(),
            };

            /* dock area */
            DockArea::new(&mut self.tree)
                .style({
                    let mut style = Style::from_egui(ui.ctx().global_style().as_ref());
                    style.tab_bar.fill_tab_bar = true;
                    style
                })
                .show_close_buttons(true)
                .show_leaf_close_all_buttons(false)
                .show_inside(ui, &mut tab_viewer);
        });
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(handle) = &self.emu_handle {
            handle.abort();
        }

        /* capture the final layout (e.g. any drag-resized splits) before saving */
        if let Some(path) = &self.ide_path {
            crate::resources::settings::save_dock_layout(path, &self.tree);
        }
        self.save_settings_if_needed();

        self.screen.destroy(gl);
    }
}
