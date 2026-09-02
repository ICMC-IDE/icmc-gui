use super::ViewState;
use crate::{State, resources::radix::Radix};
use egui_dock::egui;
use std::sync::{Arc, atomic::Ordering};

pub struct StatePanel {
    last_status: &'static str,
    last_regs: [u16; 8],
    last_iregs: [u16; 6],
}

impl Default for StatePanel {
    fn default() -> Self {
        Self {
            last_status: "Paused",
            last_regs: [0; 8],
            last_iregs: [0; 6],
        }
    }
}

impl StatePanel {
    fn reg_fmt<'a>(dv: egui::DragValue<'a>, radix: Radix) -> egui::DragValue<'a> {
        match radix {
            Radix::Binary => dv.binary(16, false),
            Radix::Decimal => dv,
            Radix::Hex => dv.hexadecimal(4, false, true),
            Radix::Octal => dv.octal(6, false),
        }
    }

    fn reg_row(ui: &mut egui::Ui, label: &str, value: &mut u16, radix: Radix, enabled: bool) {
        ui.label(label);
        ui.with_layout(
            egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Min),
            |ui| {
                ui.add_enabled(enabled, Self::reg_fmt(egui::DragValue::new(value), radix));
            },
        );
    }
}

impl ViewState for StatePanel {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.set_min_size(ui.available_size());
        ui.set_max_size(ui.available_size());

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                if state.emu_handle.is_some() {
                    state.emulator.lock().unwrap().reset();
                }

                state.spawn_run_loop();
            }

            if ui.button("Stop").clicked() {
                state.running.store(false, Ordering::SeqCst);
            }

            if ui.button("Reset").clicked() {
                let mut emu = state.emulator.lock().unwrap();
                emu.reset();
                state.running.store(false, Ordering::SeqCst);
            }

            if ui.button("Step").clicked() {
                let mut emu = state.emulator.lock().unwrap();
                emu.next();
            };
        });

        /* Current emulator status */
        if let Ok(emu) = state.emulator.try_lock() {
            self.last_status = if Arc::clone(&state.running).load(Ordering::SeqCst) {
                "Running"
            } else {
                match emu.state() {
                    icmc_emulator::State::Paused => "Paused",
                    icmc_emulator::State::BreakPoint => "Breakpoint",
                    icmc_emulator::State::Halted => "Halted",
                    icmc_emulator::State::UnknownInstruction => "Unknown Instruction",
                }
            };
        }

        ui.label(format!("State: {}", self.last_status));

        let mut freq = state.freq.lock().unwrap();
        ui.add(
            egui::Slider::new(&mut *freq, 1.0..=12_000_000.0)
                .logarithmic(true)
                .custom_formatter(|f, _| {
                    let (f_value, f_unit) = if f >= 1_000_000.0 {
                        (f / 1_000_000.0, "MHz")
                    } else if f >= 1_000.0 {
                        (f / 1_000.0, "kHz")
                    } else {
                        (f, "Hz")
                    };

                    format!("{:.1} {}", f_value, f_unit)
                })
                .custom_parser(|s| {
                    let s = s.trim();
                    let (num, mult) = if let Some(n) =
                        s.strip_suffix("MHz").or_else(|| s.strip_suffix("mhz"))
                    {
                        (n, 1e6)
                    } else if let Some(n) = s.strip_suffix("kHz").or_else(|| s.strip_suffix("khz"))
                    {
                        (n, 1e3)
                    } else if let Some(n) = s.strip_suffix("Hz").or_else(|| s.strip_suffix("hz")) {
                        (n, 1.0)
                    } else {
                        (s, 1.0)
                    };
                    num.trim().parse::<f64>().ok().map(|v| v * mult)
                }),
        );

        /* some CPU internals */
        let radix = state.settings.radix;

        if let Ok(mut emu) = state.emulator.try_lock() {
            ui.label("Registers");
            ui.horizontal(|ui| {
                for i in 0..4 {
                    Self::reg_row(ui, &format!("R{i}: "), emu.reg_as_mut_ref(i), radix, true);
                }
            });

            ui.horizontal(|ui| {
                for i in 4..8 {
                    Self::reg_row(ui, &format!("R{i}: "), emu.reg_as_mut_ref(i), radix, true);
                }
            });

            ui.label("Internal Registers");
            ui.horizontal(|ui| {
                Self::reg_row(ui, "FR: ", emu.ireg_as_mut_ref(0), radix, true);
                Self::reg_row(ui, "SP: ", emu.ireg_as_mut_ref(1), radix, true);
                Self::reg_row(ui, "PC: ", emu.ireg_as_mut_ref(2), radix, true);
                Self::reg_row(ui, "IR: ", emu.ireg_as_mut_ref(3), radix, true);
            });

            ui.horizontal(|ui| {
                Self::reg_row(ui, "KB: ", emu.ireg_as_mut_ref(4), radix, true);
                Self::reg_row(ui, "WC: ", emu.ireg_as_mut_ref(5), radix, true);
            });

            for i in 0..8 {
                self.last_regs[i] = emu.reg(i as u16);
            }
            for i in 0..6 {
                self.last_iregs[i] = emu.ireg(i as u16);
            }
        } else {
            ui.label("Registers");
            ui.horizontal(|ui| {
                for i in 0..4 {
                    Self::reg_row(ui, &format!("R{i}: "), &mut self.last_regs[i], radix, false);
                }
            });

            ui.horizontal(|ui| {
                for i in 4..8 {
                    Self::reg_row(ui, &format!("R{i}: "), &mut self.last_regs[i], radix, false);
                }
            });

            ui.label("Internal Registers");
            ui.horizontal(|ui| {
                Self::reg_row(ui, "FR: ", &mut self.last_iregs[0], radix, false);
                Self::reg_row(ui, "SP: ", &mut self.last_iregs[1], radix, false);
                Self::reg_row(ui, "PC: ", &mut self.last_iregs[2], radix, false);
                Self::reg_row(ui, "IR: ", &mut self.last_iregs[3], radix, false);
            });

            ui.horizontal(|ui| {
                Self::reg_row(ui, "KB: ", &mut self.last_iregs[4], radix, false);
                Self::reg_row(ui, "WC: ", &mut self.last_iregs[5], radix, false);
            });
        }

        ui.add_space(8.0);

        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Export cpuram.mif").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(&state.workspace_path)
                .set_file_name("cpuram.mif")
                .save_file()
            {
                let emu = state.emulator.lock().unwrap();
                let mif = mif::Mif::new(emu.rom(), mif::Radix::Uns, mif::Radix::Bin);

                if let Err(e) = std::fs::write(path, format!("{}", mif)) {
                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.add_log(format!("Failed to export cpuram.mif: {e}"));
                    }
                }
            }
        }
    }
}
