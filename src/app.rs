use crate::elements::{
    CharmapEditor, Documentation, Editor, FileExplorer, LogPanel, MemEditor,
    Screen, StatePanel, View, ViewState,
};
use crate::resources::{radix::Radix, settings::Settings};
use egui_dock::dock_state::tree::Split;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, egui};
use icmc_emulator::Emulator;
use std::{
    collections::{HashMap, HashSet},
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
                    ticks_pending = (ticks_pending
                        + (now - last).as_secs_f64() * freq_val)
                        .min(1_000_000.0);
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
                    ticks_pending = (ticks_pending
                        + (now - last) * 1e-3 * freq_val)
                        .min(1_000_000.0);
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

/* Main app */
pub struct IdeApp {
    /* Tab/Dock related */
    tree: DockState<String>,
    open_tabs: HashSet<String>,
    _nodes: HashMap<String, NodeIndex>,

    /* Core */
    egui_ctx: egui::Context,
    emulator: Arc<Mutex<Emulator>>, /* Emulator backend*/
    freq: Arc<Mutex<f64>>,          /* Emulator running frequency */

    #[cfg(not(target_arch = "wasm32"))]
    emu_handle: Option<tokio::task::JoinHandle<()>>, /* Emulator thread handle */
    #[cfg(not(target_arch = "wasm32"))]
    rt: runtime::Runtime,           /* Tokio runtime */

    running: Arc<AtomicBool>, /* Emulator thread status */

    code_buf: Option<String>,
    ide_path: Option<PathBuf>,
    open_file: Option<PathBuf>,

    settings: Settings,

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
    ) -> Self {
        let mut tree = DockState::new(vec!["Code Editor".to_owned()]);
        let mut nodes = HashMap::new();
        nodes.insert("Code Editor".to_owned(), NodeIndex::root());

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

        /* Open docks in default configuration */
        for tab in &["Screen", "State", "Log"] {
            let (parent, fraction, split) = match *tab {
                "Screen" => (NodeIndex::root(), 0.3, Split::Left),
                "State" => {
                    (nodes.get("Screen").copied().unwrap(), 0.5, Split::Below)
                }
                "Log" => (
                    nodes.get("Code Editor").copied().unwrap(),
                    0.7,
                    Split::Below,
                ),
                _ => unreachable!(),
            };

            let [a, b] = tree.main_surface_mut().split(
                parent,
                split,
                fraction,
                egui_dock::Node::leaf((*tab).to_owned()),
            );

            nodes.insert((*tab).to_owned(), b);

            if *tab == "Screen" {
                /* Code Editor is not root anymore here */
                nodes.insert("Code Editor".to_owned(), a);
            }
        }

        let mut open_tabs = HashSet::new();

        for node in tree[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }

        let root_path = ide_path
            .as_ref()
            .map(|path| PathBuf::from(path).join("workspace"));

        let example_path = root_path
            .as_ref()
            .map(|root| root.join("example.asm"))
            .unwrap_or_else(|| PathBuf::from("example.asm"));

        let settings_path = ide_path
            .as_ref()
            .map(|path| PathBuf::from(path).join("settings.toml"));

        let settings: Settings = settings_path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|toml_str| toml::from_str(&toml_str).ok())
            .unwrap_or_default();

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
            _nodes: nodes,

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

    fn find_node_index(&self, name: &str) -> Option<NodeIndex> {
        let surface = &self.tree[SurfaceIndex::main()];

        for (i, node) in surface.iter().enumerate() {
            if let Some(tabs) = node.tabs() {
                if tabs.iter().any(|t| t == name) {
                    return Some(NodeIndex::from(i));
                }
            }
        }
        None
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            egui::widgets::global_theme_preference_switch(ui);

            for tab in &[
                "Code Editor",
                "Screen",
                "State",
                "Log",
                "File Explorer",
                "Documentation",
                "Memory Editor",
                "Charmap Editor",
            ] {
                if cfg!(target_arch = "wasm32")
                    && (tab == &"File Explorer" || tab == &"Charmap Editor")
                {
                    continue;
                }

                let is_open = self.open_tabs.contains(*tab);

                if ui.selectable_label(is_open, *tab).clicked() {
                    if let Some(idx) = self.tree.find_tab(&tab.to_string()) {
                        self.tree.remove_tab(idx);
                        self.open_tabs.remove(*tab);
                    } else {
                        if *tab == "Documentation" || *tab == "Charmap Editor" {
                            self.tree.add_window(vec![tab.to_string()]);
                        } else {
                            let surf = self.tree.main_surface_mut();
                            let empty_root = surf.iter().all(|node| {
                                node.tabs().map_or(true, |t| t.is_empty())
                            });

                            if empty_root {
                                surf.push_to_focused_leaf((*tab).to_string());
                            } else {
                                let (parent, fraction, split) = match *tab {
                                    "Log" => (
                                        self.find_node_index("Code Editor")
                                            .unwrap_or(NodeIndex::root()),
                                        0.7,
                                        Split::Below,
                                    ),
                                    "Memory Editor" => (
                                        self.find_node_index("Code Editor")
                                            .unwrap_or(NodeIndex::root()),
                                        0.6,
                                        Split::Right,
                                    ),
                                    "File Explorer" => (
                                        self.find_node_index("Code Editor")
                                            .unwrap_or(NodeIndex::root()),
                                        0.8,
                                        Split::Right,
                                    ),
                                    _ => (NodeIndex::root(), 0.8, Split::Right),
                                };

                                self.tree.main_surface_mut().split(
                                    parent,
                                    split,
                                    fraction,
                                    egui_dock::Node::leaf((*tab).to_string()),
                                );
                            }
                        }
                        self.open_tabs.insert((*tab).to_string());
                    }
                }
            }

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Max),
                |ui| {
                    ui.menu_button("Radix", |ui| {
                        ui.selectable_value(
                            &mut self.settings.radix,
                            Radix::Binary,
                            "Binary",
                        );
                        ui.selectable_value(
                            &mut self.settings.radix,
                            Radix::Decimal,
                            "Decimal",
                        );
                        ui.selectable_value(
                            &mut self.settings.radix,
                            Radix::Hex,
                            "Hexadecimal",
                        );
                        ui.selectable_value(
                            &mut self.settings.radix,
                            Radix::Octal,
                            "Octal",
                        );
                    });
                },
            );
        });

        ui.add_space(2.0);
    }
}

impl eframe::App for IdeApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.open_tabs.is_empty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        /* save altered settings to file */
        if self.settings.needs_save {
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

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        /* top menu */
        egui::Panel::top("Top Bar").show(ui, |ui| {
            self.bar_contents(ui, frame);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let focused_tab = self.tree.find_active_focused().map(|(_, tab)| tab.clone());

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

        self.screen.destroy(gl);
    }
}
