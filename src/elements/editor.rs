use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct Editor {
    code_buf: String, /* Editor buffer */
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            code_buf: include_str!("../../res/example.asm").to_owned(),
        }
    }
}

impl ViewState for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State, ctx: &mut egui::Context) {
        ui.add_space(10.0);

        if ui.button("Save & Build").clicked() {
            let mut fs = state.fs.lock().unwrap();
            let mut emu = state.emulator.lock().unwrap();
            let icmc_syntax = include_str!("../../res/icmc.toml");

            /* TODO: stop saving code in "./.code.asm" and implement
             * a file explorer */
            match fs.write(".code.asm", self.code_buf.as_bytes()) {
                Ok(_) => (),
                Err(_) => {
                    println!("Couldn't write code file");
                }
            }

            match fs.write(".icmc.toml", icmc_syntax.as_bytes()) {
                Ok(_) => (),
                Err(err) => {
                    println!("Couldn't write syntax file");
                }
            }

            let asm = match assembler::assemble(&fs, ".code.asm", ".icmc.toml") {
                Ok(res) => Some(res),
                Err(err) => {
                    println!("Couldn't assemble code");
                    None
                }
            };

            if let Some(asm) = asm {
                emu.load(&asm.binary());
            }
        }

        /* fit editor into panel screen */
        let size = egui::Vec2::new(ui.available_width(), ui.available_height());

        ui.add(
            egui::TextEdit::multiline(&mut self.code_buf)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .min_size(size)
                .desired_width(f32::INFINITY),
        );
    }
}
