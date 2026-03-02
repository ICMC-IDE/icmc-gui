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
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut State,
        ctx: &mut egui::Context,
    ) {
        let mut freq = state.freq.lock().unwrap();

        ui.set_min_size(ui.available_size());
        ui.set_max_size(ui.available_size());

        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                let freq = Arc::clone(&state.freq);
                let emu = Arc::clone(&state.emulator);
                let ctx = ctx.clone();

                if let Some(handle) = &state.emu_handle {
                    handle.abort();
                    let mut emu = emu.lock().unwrap();
                    emu.reset();
                }

                let running = Arc::clone(&state.running);

                running.store(true, Ordering::SeqCst);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    use std::thread;
                    use std::time::{Duration, Instant};

                    *state.emu_handle = Some(state.rt.spawn(async move {
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

                            let sleep_time =
                                Duration::from_secs_f64(1.0 / freq_val);
                            let elapsed = start.elapsed();

                            if elapsed < sleep_time {
                                thread::sleep(sleep_time - elapsed);
                            }
                        }
                    }));
                }

                #[cfg(target_arch = "wasm32")]
                {
                    use gloo_timers::future::TimeoutFuture;
                    use std::cell::RefCell;
                    use std::rc::Rc;
                    use wasm_bindgen_futures::spawn_local;

                    fn performance_now() -> f64 {
                        web_sys::window().unwrap().performance().unwrap().now()
                    }

                    running.store(true, Ordering::SeqCst);

                    let ticks_pending = Rc::new(RefCell::new(0.0));
                    let last_tick = Rc::new(RefCell::new(performance_now()));

                    spawn_local(async move {
                        while running.load(Ordering::SeqCst) {
                            let now = performance_now();
                            let elapsed = now - *last_tick.borrow();
                            *last_tick.borrow_mut() = now;

                            let freq_val = {
                                let f = freq.lock().unwrap();
                                *f
                            };

                            *ticks_pending.borrow_mut() +=
                                elapsed * freq_val * 1e-3;

                            if *ticks_pending.borrow() > 1_000_000.0 {
                                *ticks_pending.borrow_mut() = 1_000_000.0;
                            }

                            {
                                let mut emu = emu.lock().unwrap();

                                if emu.state() == icmc_emulator::State::Halted {
                                    running.store(false, Ordering::SeqCst);
                                    break;
                                }

                                let ticks_done = emu
                                    .tick(*ticks_pending.borrow() as isize)
                                    as f64;
                                *ticks_pending.borrow_mut() -= ticks_done;
                            }

                            ctx.request_repaint();

                            TimeoutFuture::new(0).await;
                        }
                    });
                }
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
