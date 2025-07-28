use crate::elements::{Editor, Screen, StatePanel, LogPanel, View, ViewState, FileExplorer};
use egui_dock::{egui, DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use icmc_emulator::Emulator;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::thread::JoinHandle;

/* Emulator state */
pub struct State<'a> {
    pub emulator: Arc<Mutex<Emulator>>,
    pub fs: Arc<Mutex<fs::Fs>>,
    pub freq: Arc<Mutex<f64>>,
    pub emu_handle: &'a mut Option<JoinHandle<()>>,
    pub running: Arc<AtomicBool>,
    pub log_panel: Arc<Mutex<LogPanel>>, 
}

/* Tab manager */
pub struct TabViewer<'a> {
    editor: &'a mut Editor,
    screen: &'a mut Screen,
    state_panel: &'a mut StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: &'a mut FileExplorer, // ← novo

    ctx: &'a mut egui::Context,
    state: &'a mut State<'a>,
    nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
}


impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
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

            "Log" => {
                if let Ok(mut log_panel) = self.log_panel.lock() {
                    log_panel.ui(ui, self.state, self.ctx);
                }
            }

            "File Explorer" => {
                self.file_explorer.ui(ui, self.ctx);
            }
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

/* Main app */
pub struct IdeApp {
    /* Tab/Dock related */
    tree: DockState<String>,
    focused: String,

    /* Core */
    emulator: Arc<Mutex<Emulator>>,     /* Emulator backend*/
    fs: Arc<Mutex<fs::Fs>>,             /* Filesystem */
    freq: Arc<Mutex<f64>>,              /* Emulator running frequency */
    emu_handle: Option<JoinHandle<()>>, /* Emulator thread handle */
    running: Arc<AtomicBool>,           /* Emulator thread status */

    /* Elements */
    editor: Editor,
    screen: Screen,
    state_panel: StatePanel,
    log_panel: Arc<Mutex<LogPanel>>,
    file_explorer: FileExplorer,
}

impl IdeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut tree = DockState::new(vec!["Code Editor".to_owned()]);

        let emulator = Arc::new(Mutex::new(icmc_emulator::Emulator::new()));
        let fs = Arc::new(Mutex::new(fs::Fs::new()));
        let freq = Arc::new(Mutex::new(1.0));
        let emu_handle = None;
        let running = Arc::new(AtomicBool::new(false));
        
        let [_, _] =
            tree.main_surface_mut()
                .split_right(NodeIndex::root(), 0.3, vec!["Screen".to_owned()]);

        let [a, b] = 
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["File Explorer".to_owned()]);
        let [_, _] = tree.main_surface_mut()
            .split_below(a, 0.8, vec!["Log".to_owned()]);

        let [_, _] = tree
            .main_surface_mut()
            .split_below(b, 0.5, vec!["State".to_owned()]);

        Self {
            tree,
            focused: "Screen".to_owned(),

            emulator,
            fs,
            freq,
            emu_handle,
            running,

            editor: Editor::default(),
            screen: Screen::new(cc),
            state_panel: StatePanel::default(),
            log_panel: Arc::new(Mutex::new(LogPanel::default())),
            file_explorer: FileExplorer::new(),
        }
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut nodes = Vec::new();

        let mut state = State {
            emulator: self.emulator.clone(),
            fs: self.fs.clone(),
            freq: self.freq.clone(),
            emu_handle: &mut self.emu_handle,
            running: self.running.clone(),
            log_panel: self.log_panel.clone(),
        };

        let mut tab_viewer = TabViewer {
            editor: &mut self.editor,
            screen: &mut self.screen,
            state_panel: &mut self.state_panel,
            file_explorer: &mut self.file_explorer, // ← novo
            ctx: &mut ctx.clone(),
            state: &mut state,
            nodes: &mut nodes,
            log_panel: self.log_panel.clone(),
        };

        /* top menu */
        egui::TopBottomPanel::top("Top Bar").show(ctx, |ui| {
            egui::widgets::global_theme_preference_switch(ui);
        });

        /* dock area */
        DockArea::new(&mut self.tree)
            .show_add_buttons(false)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tab_bar.fill_tab_bar = true;
                style
            })
            .show(ctx, &mut tab_viewer);

        nodes.drain(..).for_each(|(surface, node)| {
            self.tree.set_focused_node_and_surface((surface, node));
            self.tree.push_to_focused_leaf(self.focused.clone());
        });
    }
}
