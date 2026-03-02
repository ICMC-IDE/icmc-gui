use super::ViewState;
use crate::State;
use crate::resources::syntax;
use egui_code_editor::{CodeEditor, ColorTheme};
use egui_dock::egui;
use std::sync::{Arc, atomic::Ordering};

#[derive(Default)]
pub struct Editor;

impl ViewState for Editor {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut State,
        ctx: &mut egui::Context,
    ) {
        let code_buf = state.code_buf.get_or_insert_with(|| {
            include_str!("../../res/example.asm").to_owned()
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let mut fs = state.fs.lock().unwrap();

                #[cfg(target_family = "wasm")]
                {
                    todo!("Need to implement JS wrapper to fs.js");
                    /*
                    fs.write(".code.asm", state.code_buf.as_bytes());
                    fs.write(".icmc.toml", icmc_syntax.as_bytes());
                    */
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    let open_file = match state.open_file {
                        Some(f) => f.to_str().unwrap(),
                        &mut None => todo!(),
                    };

                    if let Err(e) = fs.write(open_file, code_buf.as_bytes()) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!(
                                "Failed to write .code.asm: {}",
                                e
                            ));
                        }
                        return;
                    }
                }
            }

            if ui.button("Build and Run").clicked() {
                #[cfg(target_family = "wasm")]
                {
                    /* testing */
                    if let Err(asm_res) = std::panic::catch_unwind(|| {
                        match assembler::assemble_from_buf(
                            code_buf,
                            include_str!("../../res/icmc.toml"),
                        ) {
                            Ok(asm) => {
                                let mut emu = state.emulator.lock().unwrap();

                                emu.load(&asm.binary());
                                if let Ok(mut log_panel) =
                                    state.log_panel.lock()
                                {
                                    log_panel.add_log(
                                        "Assembly successful! Binary loaded."
                                            .to_string(),
                                    );
                                }
                            }
                            Err(_) => {
                                if let Ok(mut log_panel) =
                                    state.log_panel.lock()
                                {
                                    log_panel.add_log(
                                        "Assembly error: syntax error"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(
                                "Assembly error: syntax error".to_string(),
                            );
                        }

                        return;
                    }
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    let mut fs = state.fs.lock().unwrap();
                    let icmc_syntax = include_str!("../../res/icmc.toml");

                    let syntax_path = &format!(
                        "{}/icmc.toml",
                        state.ide_path.clone().unwrap().display()
                    );

                    if let Err(e) =
                        fs.write(syntax_path, icmc_syntax.as_bytes())
                    {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!(
                                "Failed to write .icmc.toml: {}",
                                e
                            ));
                        }
                        return;
                    }

                    let open_file = match state.open_file {
                        Some(f) => f.to_str().unwrap(),
                        &mut None => todo!(),
                    };

                    if let Err(e) = fs.write(open_file, code_buf.as_bytes()) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(format!(
                                "Failed to write .code.asm: {}",
                                e
                            ));
                        }
                        return;
                    }

                    if let Ok(mut log_panel) = state.log_panel.lock() {
                        log_panel.auto_scroll();
                    }

                    if let Err(_) = std::panic::catch_unwind(|| {
                        match assembler::assemble(&fs, open_file, syntax_path) {
                            Ok(asm) => {
                                let mut emu = state.emulator.lock().unwrap();

                                emu.load_program(&asm.binary());
                                if let Ok(mut log_panel) =
                                    state.log_panel.lock()
                                {
                                    log_panel.add_log(
                                        "Assembly successful! Binary loaded."
                                            .to_string(),
                                    );
                                }
                            }
                            Err(err) => {
                                if let Ok(mut log_panel) =
                                    state.log_panel.lock()
                                {
                                    log_panel.add_log(format!(
                                        "Assembly error: {}",
                                        err
                                    ));

                                    if let Some((line, col)) =
                                        extract_line_column(&err)
                                    {
                                        log_panel.add_log(format!(
                                            "    at line {}, column {}",
                                            line, col
                                        ));
                                    }
                                }
                            }
                        }
                    }) {
                        if let Ok(mut log_panel) = state.log_panel.lock() {
                            log_panel.add_log(
                                "Assembly error: syntax error".to_string(),
                            );
                        }

                        return;
                    }
                }

                let freq = Arc::clone(&state.freq);
                let emu = Arc::clone(&state.emulator);
                let ctx = ctx.clone();

                let running = Arc::clone(&state.running);

                running.store(true, Ordering::SeqCst);

                #[cfg(not(target_arch = "wasm32"))]
                {
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
                                tokio::time::sleep(sleep_time - elapsed).await;
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

            if ui.button("Clear Editor").clicked() {
                code_buf.clear();
            }

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button("Reset font size").clicked() {
                        state.settings.font_size = 14.0;
                    }

                    if ui.button("-").clicked()
                        && state.settings.font_size >= 4.0
                    {
                        state.settings.font_size -= 2.0;
                    }

                    if ui.button("+").clicked()
                        && state.settings.font_size <= 64.0
                    {
                        state.settings.font_size += 2.0;
                    }

                    ui.label(format!(
                        "Font size: {} pt",
                        state.settings.font_size
                    ));
                },
            );
        });

        let color_theme = if ui.visuals().dark_mode {
            ColorTheme::GITHUB_DARK
        } else {
            ColorTheme::GITHUB_LIGHT
        };

        CodeEditor::default()
            .id_source("asm_editor")
            .with_rows(0)
            .with_fontsize(state.settings.font_size)
            .with_syntax(syntax::icmc())
            .with_theme(color_theme)
            .with_numlines(true)
            .show(ui, code_buf);
    }
}

fn extract_line_column(error_msg: &str) -> Option<(usize, usize)> {
    let mut line = 0;
    let mut col = 0;

    if let Some(pos) = error_msg.find("line ") {
        let rest = &error_msg[pos + 5..];
        line = rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
    }

    if let Some(pos) = error_msg.find("column ") {
        let rest = &error_msg[pos + 7..];
        col = rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
    }

    if line > 0 || col > 0 {
        Some((line, col))
    } else {
        None
    }
}
