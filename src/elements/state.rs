use super::ViewState;
use crate::State;
use egui_dock::egui;
use std::sync::{atomic::Ordering, Arc};
use std::thread;
use std::time::{Duration, Instant};

pub struct StatePanel;

impl Default for StatePanel {
    fn default() -> Self {
        Self {}
    }
}

impl ViewState for StatePanel {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State, ctx: &mut egui::Context) {
        let mut freq = state.freq.lock().unwrap();

        ui.set_min_size(ui.available_size());
        ui.set_max_size(ui.available_size());

        /* Current emulator status */
        if let Ok(emu) = state.emulator.try_lock() {
            let status = if Arc::clone(&state.running).load(Ordering::SeqCst) {
                "Running"
            } else {
                match emu.state() {
                    icmc_emulator::State::Paused => "Paused",
                    icmc_emulator::State::BreakPoint => "Breakpoint",
                    icmc_emulator::State::Halted => "Halted",
                    icmc_emulator::State::UnknownInstruction => "Unknown Instruction",
                }
            };

            ui.label(format!("State: {}", status));
        } else {
            ui.label("State: (emulator busy)");
        }

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                let freq = Arc::clone(&state.freq);
                let emu = Arc::clone(&state.emulator);
                let ctx = ctx.clone();

                let running = Arc::clone(&state.running);

                running.store(true, Ordering::SeqCst);

                /* TODO: improve thread communcation with std::sync::mpsc */

                *state.emu_handle = Some(thread::spawn(move || {
                    while running.load(Ordering::SeqCst) {
                        let start = Instant::now();

                        {
                            let mut emu = emu.lock().unwrap();

                            /* Stop if emulator is halted */
                            if emu.state() == icmc_emulator::State::Halted {
                                running.store(false, Ordering::SeqCst);
                            }

                            emu.next();
                        }

                        /* ensure that egui doesn't stop rendering */
                        ctx.request_repaint();

                        let freq_val = {
                            let f = freq.lock().unwrap();
                            *f
                        };

                        let sleep_time = Duration::from_secs_f64(1.0 / freq_val);
                        let elapsed = start.elapsed();

                        if elapsed < sleep_time {
                            thread::sleep(sleep_time - elapsed);
                        }
                    }
                }));
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

        ui.add(egui::Slider::new(&mut *freq, 1.0..=1000.0).text("Frequency"));

        /* some CPU internals */
        if let Ok(emu) = state.emulator.try_lock() {
            ui.label(format!("PC: {}", emu.pc()));

            for i in 0..8 {
                ui.label(format!("Register {}: {}", i, emu.reg(i)));
            }
        } else {
            ui.label("Registers: (emulator busy)");
        }
    }
}
