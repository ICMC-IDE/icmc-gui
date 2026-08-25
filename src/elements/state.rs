use super::ViewState;
use crate::{State, resources::radix::Radix};
use egui_dock::egui;
use std::sync::{Arc, atomic::Ordering};

#[derive(Default)]
pub struct StatePanel;

impl StatePanel {
    fn reg_fmt<'a>(
        dv: egui::DragValue<'a>,
        radix: Radix,
    ) -> egui::DragValue<'a> {
        match radix {
            Radix::Binary => dv.binary(8, false),
            Radix::Decimal => dv,
            Radix::Hex => dv.hexadecimal(4, false, true),
            Radix::Octal => dv.octal(6, false),
        }
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
            let status = if Arc::clone(&state.running).load(Ordering::SeqCst) {
                "Running"
            } else {
                match emu.state() {
                    icmc_emulator::State::Paused => "Paused",
                    icmc_emulator::State::BreakPoint => "Breakpoint",
                    icmc_emulator::State::Halted => "Halted",
                    icmc_emulator::State::UnknownInstruction => {
                        "Unknown Instruction"
                    }
                }
            };

            ui.label(format!("State: {}", status));
        } else {
            ui.label("State: (emulator busy)");
        }

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
                    } else if let Some(n) =
                        s.strip_suffix("kHz").or_else(|| s.strip_suffix("khz"))
                    {
                        (n, 1e3)
                    } else if let Some(n) =
                        s.strip_suffix("Hz").or_else(|| s.strip_suffix("hz"))
                    {
                        (n, 1.0)
                    } else {
                        (s, 1.0)
                    };
                    num.trim().parse::<f64>().ok().map(|v| v * mult)
                }),
        );

        /* some CPU internals */
        if let Ok(mut emu) = state.emulator.try_lock() {
            ui.label("Registers");
            ui.horizontal(|ui| {
                for i in 0..4 {
                    ui.label(format!("R{}: ", i));
                    ui.add(Self::reg_fmt(
                        egui::DragValue::new(emu.reg_as_mut_ref(i)),
                        state.settings.radix,
                    ));
                }
            });

            ui.horizontal(|ui| {
                for i in 4..8 {
                    ui.label(format!("R{}: ", i));
                    ui.add(Self::reg_fmt(
                        egui::DragValue::new(emu.reg_as_mut_ref(i)),
                        state.settings.radix,
                    ));
                }
            });

            ui.label("Internal Registers");
            ui.horizontal(|ui| {
                ui.label(format!("FR: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(0)),
                    state.settings.radix,
                ));

                ui.label(format!("SP: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(1)),
                    state.settings.radix,
                ));

                ui.label(format!("PC: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(2)),
                    state.settings.radix,
                ));

                ui.label(format!("IR: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(3)),
                    state.settings.radix,
                ));
            });

            ui.horizontal(|ui| {
                ui.label(format!("KB: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(4)),
                    state.settings.radix,
                ));

                ui.label(format!("WC: "));
                ui.add(Self::reg_fmt(
                    egui::DragValue::new(emu.ireg_as_mut_ref(5)),
                    state.settings.radix,
                ));
            });
        } else {
            ui.label("Registers: (emulator busy)");
        }
    }
}
