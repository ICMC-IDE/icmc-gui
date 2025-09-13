use crate::elements::{
    Documentation, Editor, FileExplorer, LogPanel, MemEditor, Screen, StatePanel, View, ViewState,
};
use crate::resources::{radix::Radix, settings::Settings};
use egui_dock::dock_state::tree::Split;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, egui};
use icmc_emulator::Emulator;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread::JoinHandle,
};

/* Emulator state */
pub struct State<'a> {
    pub emulator: Arc<Mutex<Emulator>>,
    pub fs: Arc<Mutex<fs::Fs>>,
    pub freq: Arc<Mutex<f64>>,
    pub emu_handle: &'a mut Option<JoinHandle<()>>,
    pub running: Arc<AtomicBool>,
    pub code_buf: &'a mut Option<String>,
    pub log_panel: Arc<Mutex<LogPanel>>,
    pub mem_editor: Arc<Mutex<MemEditor>>,
    pub ide_path: &'a mut Option<PathBuf>,
    pub open_file: &'a mut Option<PathBuf>,
    pub settings: &'a mut Settings,
}

/* Tab manager */
pub struct TabViewer<'a> {
    editor: &'a mut Editor,
    doc: &'a mut Documentation,
    screen: &'a mut Screen,
    state_panel: &'a mut StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: &'a mut FileExplorer,
    mem_editor: Arc<Mutex<MemEditor>>,

    ctx: &'a mut egui::Context,
    open_tabs: &'a mut HashSet<String>,
    state: &'a mut State<'a>,
    tree: &'a mut DockState<String>,
}

impl TabViewer<'_> {
    fn focused_tab(&mut self) -> Option<String> {
        let Some((_, tab)) = self.tree.find_active_focused() else {
            return None;
        };
        Some(tab.to_string())
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if let Some(focused) = self.focused_tab() {
            self.state.settings.input_enabled = &focused == "Screen";
        }

        match tab.as_str() {
            "Screen" => {
                let screen = &mut self.screen;
                let state = &mut self.state;

                screen.ui(ui, state, self.ctx);
            }

            "State" => {
                let state_panel = &mut self.state_panel;
                let state = &mut self.state;
                state_panel.ui(ui, state, self.ctx);
            }

            "Code Editor" => {
                let state = &mut self.state;
                self.editor.ui(ui, state, self.ctx);
            }

            "Log" => {
                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.ui(ui, self.state, self.ctx);
                }
            }

            "File Explorer" => {
                self.file_explorer.ui(ui, self.state, self.ctx);
            }

            "Documentation" => {
                self.doc.ui(ui, self.ctx);
            }

            "Memory Editor" => {
                if let Ok(mut mem_editor) = self.mem_editor.lock() {
                    mem_editor.ui(ui, self.state, self.ctx);
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
    nodes: HashMap<String, NodeIndex>,

    /* Core */
    emulator: Arc<Mutex<Emulator>>,     /* Emulator backend*/
    fs: Arc<Mutex<fs::Fs>>,             /* Filesystem */
    freq: Arc<Mutex<f64>>,              /* Emulator running frequency */
    emu_handle: Option<JoinHandle<()>>, /* Emulator thread handle */
    running: Arc<AtomicBool>,           /* Emulator thread status */

    code_buf: Option<String>,
    ide_path: Option<PathBuf>,
    open_file: Option<PathBuf>,

    settings: Settings,

    /* Elements */
    editor: Editor,
    doc: Documentation,
    screen: Screen,
    state_panel: StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: FileExplorer,
    mem_editor: Arc<Mutex<MemEditor>>,
}

impl IdeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, ide_path: Option<PathBuf>) -> Self {
        let mut tree = DockState::new(vec!["Code Editor".to_owned()]);
        let mut nodes = HashMap::new();
        nodes.insert("Code Editor".to_owned(), NodeIndex::root());

        let emulator = Arc::new(Mutex::new(icmc_emulator::Emulator::new()));
        let fs = Arc::new(Mutex::new(fs::Fs::new()));
        let freq = Arc::new(Mutex::new(1.0));
        let emu_handle = None;
        let running = Arc::new(AtomicBool::new(false));

        /* Open docks in default configuration */
        for tab in &["Screen", "State", "Log"] {
            let (parent, fraction, split) = match *tab {
                "Screen" => (NodeIndex::root(), 0.3, Split::Left),
                "State" => (nodes.get("Screen").copied().unwrap(), 0.5, Split::Below),
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

        let root_path = match ide_path {
            Some(ref path) => Some(PathBuf::from(format!("{}workspace/", path.display()))),
            None => None,
        };
        let example_path = match root_path {
            Some(ref root_path) => {
                PathBuf::from(format!("{}example.asm", root_path.clone().display()))
            }
            None => PathBuf::from("example.asm"),
        };

        let binding = fs.clone();
        let mut fs_unlock = binding.lock().unwrap();
        fs_unlock.write(
            example_path.to_str().unwrap(),
            include_str!("../res/example.asm").to_owned().as_bytes(),
        );

        Self {
            tree,
            open_tabs,
            nodes,

            emulator,
            fs,
            freq,
            emu_handle,
            running,

            code_buf: None,
            ide_path: ide_path.clone(),
            open_file: Some(example_path),

            settings: Default::default(),

            editor: Editor::default(),
            doc: Documentation::default(),
            screen: Screen::new(cc),
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
            ] {
                let is_open = self.open_tabs.contains(*tab);

                if ui.selectable_label(is_open, *tab).clicked() {
                    if let Some(idx) = self.tree.find_tab(&tab.to_string()) {
                        self.tree.remove_tab(idx);
                        self.open_tabs.remove(*tab);
                    } else {
                        if *tab == "Documentation" {
                            self.tree.add_window(vec!["Documentation".to_owned()]);
                        } else {
                            let surf = self.tree.main_surface_mut();
                            let empty_root = surf
                                .iter()
                                .all(|node| node.tabs().map_or(true, |t| t.is_empty()));

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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                ui.menu_button("Radix", |ui| {
                    ui.selectable_value(&mut self.settings.radix, Radix::Binary, "Binary");
                    ui.selectable_value(&mut self.settings.radix, Radix::Decimal, "Decimal");
                    ui.selectable_value(&mut self.settings.radix, Radix::Hex, "Hexadecimal");
                    ui.selectable_value(&mut self.settings.radix, Radix::Octal, "Octal");
                });
            });
        });

        ui.add_space(2.0);
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.open_tabs.is_empty() {
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            });
        }

        /* top menu */
        egui::TopBottomPanel::top("Top Bar").show(ctx, |ui| {
            self.bar_contents(ui, frame);
        });

        let mut state = State {
            emulator: self.emulator.clone(),
            fs: self.fs.clone(),
            freq: self.freq.clone(),
            emu_handle: &mut self.emu_handle,
            running: self.running.clone(),
            code_buf: &mut self.code_buf,
            log_panel: self.log_panel.clone(),
            mem_editor: self.mem_editor.clone(),
            ide_path: &mut self.ide_path,
            open_file: &mut self.open_file,
            settings: &mut self.settings,
        };

        let mut tab_viewer = TabViewer {
            editor: &mut self.editor,
            doc: &mut self.doc,
            screen: &mut self.screen,
            state_panel: &mut self.state_panel,
            file_explorer: &mut self.file_explorer,
            ctx: &mut ctx.clone(),
            open_tabs: &mut self.open_tabs,
            state: &mut state,
            tree: &mut self.tree.clone(),
            log_panel: self.log_panel.clone(),
            mem_editor: self.mem_editor.clone(),
        };

        /* dock area */
        DockArea::new(&mut self.tree)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tab_bar.fill_tab_bar = true;
                style
            })
            .show_close_buttons(true)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut tab_viewer);
    }
}
