use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct StatePanel;

impl Default for StatePanel {
    fn default() -> Self {
        Self {}
    }
}

impl ViewState for StatePanel {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.add_space(10.0);

        /* Current emulator status */
        ui.label(format!(
            "State: {}",
            match state.emulator.state() {
                icmc_emulator::State::Paused => "Paused",
                icmc_emulator::State::BreakPoint => "Breakpoint",
                icmc_emulator::State::Halted => "Halted",
                icmc_emulator::State::UnknownInstruction => "Unknown Instruction",
            }
        ));

        /* load example program from "./example.asm" for testing. will remove this later */
        if ui.button("Load Program (test)").clicked() {
            let asm =
                assembler::assemble(&state.fs, "example.asm", "icmc.toml").expect("Assembly error");

            state.emulator.load(&asm.binary());
        };

        ui.horizontal(|ui| {
            /* todo: add run, stop and reset */
            if ui.button("Step").clicked() {
                state.emulator.next();
            };
        });

        /* some CPU internals */
        ui.label(format!("PC: {}", state.emulator.pc()));

        for i in 0..8 {
            ui.label(format!("Register {}: {}", i, state.emulator.reg(i)));
        }
    }
}
