#![warn(clippy::all)]

use icmc_gui::IdeApp;

/* native */
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "ICMC IDE (native)",                          /* title */
        native_options,                               /* options */
        Box::new(|_cc| Ok(Box::<IdeApp>::default())), /* creation ctx */
    )
}

/* todo wasm32 target */
