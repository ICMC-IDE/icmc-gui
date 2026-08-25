#![warn(clippy::all)]

use icmc_gui::IdeApp;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

/* native */
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([780.0, 530.0]),
        ..Default::default()
    };

    let ide_path = ide_dir();

    eframe::run_native(
        "ICMC IDE (native)", /* title */
        native_options,      /* options */
        Box::new(|cc| Ok(Box::new(<IdeApp>::new(cc, Some(ide_path))))), /* creation ctx */
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn ide_dir() -> PathBuf {
    if let Some(docs) = dirs::document_dir() {
        let path = docs.join("ICMC IDE");
        if ensure_writable_dir(&path).is_ok() {
            return path;
        }
    }

    let fallback = PathBuf::from(".icmc_ide");
    ensure_writable_dir(&fallback).expect("Couldn't create local dir");
    fallback
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_writable_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path.join("workspace"))?;

    let probe = path.join(".write_test");
    std::fs::write(&probe, b"")?;
    std::fs::remove_file(&probe)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(<IdeApp>::new(cc, None)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
