#![warn(clippy::all)]

use icmc_gui::IdeApp;
use std::path::PathBuf;

/* native */
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([780.0, 530.0]),
        ..Default::default()
    };

    /* Create local directory if it doesn't exist */
    let ide_dir = if cfg!(unix) {
        match std::env::var_os("HOME") {
            Some(v) => format!("{}/.icmc_ide/", v.into_string().unwrap()),
            None => "./.icmc_ide/".to_owned(),
        }
    } else if cfg!(windows) {
        match std::env::var_os("LOCALAPPDATA") {
            Some(v) => format!("{}\\icmc_ide\\", v.into_string().unwrap()),
            None => ".\\.icmc_ide\\".to_owned(),
        }
    } else {
        todo!();
    };

    let ide_path = PathBuf::from(&ide_dir);

    if !ide_path.exists() {
        std::fs::create_dir(&ide_path).expect("Couldn't create local dir");
        std::fs::create_dir(format!("{}/workspace", ide_path.display()))
            .expect("Couldn't create workspace dir");
    }

    eframe::run_native(
        "ICMC IDE (native)", /* title */
        native_options,      /* options */
        Box::new(|cc| Ok(Box::new(<IdeApp>::new(cc, Some(ide_path))))), /* creation ctx */
    )
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
