use crate::elements::{Editor, Screen, StatePanel, View, ViewState};
use egui_dock::{egui, DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use icmc_emulator::Emulator;

/* Emulator state */
pub struct State<'a> {
    pub emulator: &'a mut Emulator,
    pub fs: &'a fs::Fs,
}

/* Tab manager */
pub struct TabViewer<'a> {
    editor: &'a mut Editor,
    screen: &'a mut Screen,
    state_panel: &'a mut StatePanel,

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
                screen.ui(ui, state);
            }
            "State" => {
                let state_panel = &mut self.state_panel;
                let state = &mut self.state;
                state_panel.ui(ui, state);
            }
            "Code Editor" => self.editor.ui(ui),
            _ => {
                ui.label(tab.as_str());
            }
        }
    }
}

/* Main app */
pub struct IdeApp {
    /* Tab/Dock related */
    tree: DockState<String>,
    focused: String,

    /* Core */
    emulator: icmc_emulator::Emulator,
    fs: fs::Fs,

    /* Elements */
    editor: Editor,
    screen: Screen,
    state_panel: StatePanel,
}

impl Default for IdeApp {
    /* Default dock state (screen, state panel and editor) */
    fn default() -> Self {
        let mut tree = DockState::new(vec!["Code Editor".to_owned()]);

        let emulator = icmc_emulator::Emulator::new();
        let fs = fs::Fs::new();

        let [_, b] =
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["Screen".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(b, 0.5, vec!["State".to_owned()]);

        Self {
            tree,
            focused: "Screen".to_owned(),

            emulator,
            fs,

            editor: Editor::default(),
            screen: Screen::default(),
            state_panel: StatePanel::default(),
        }
    }
}

impl IdeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut nodes = Vec::new();

        let mut state = State {
            emulator: &mut self.emulator,
            fs: &mut self.fs,
        };

        let mut tab_viewer = TabViewer {
            editor: &mut self.editor,
            screen: &mut self.screen,
            state_panel: &mut self.state_panel,
            state: &mut state,
            nodes: &mut nodes,
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
