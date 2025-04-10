#![warn(clippy::all)]

use icmc_gui::IDEApp; 

/* native */
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default(),
		..Default::default()
	};

	eframe::run_native(
		"ICMC IDE (native)", /* title */
		native_options, /* options */
		Box::new(|cc| Ok(Box::new(IDEApp::new(cc)))), /* creation ctx */
	)
}

/* todo: wasm32 target */
