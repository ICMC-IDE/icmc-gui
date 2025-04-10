pub struct IDEApp {
	code_buf: String
}

impl Default for IDEApp {
	fn default() -> Self {
		Self {
			code_buf: String::from("(code example)")
		}
	}
}

impl IDEApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		/* load previous state from cc.storage */

		Default::default()
	}
}

/* App trait */
impl eframe::App for IDEApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		/* Code editor (just for testing, will change later) */
		egui::SidePanel::left("Code Editor")
			.exact_width(ctx.screen_rect().width()/2.0)
			.show(ctx, |ui| {
				ui.add_space(10.0);
				ui.add(
					egui::TextEdit::multiline(&mut self.code_buf)
						.font(egui::TextStyle::Monospace)
						.code_editor()
						.desired_rows(10)
						.desired_width(f32::INFINITY)
				)
			});

		/* Screen panel */
		egui::SidePanel::right("Screen")
			.exact_width(ctx.screen_rect().width()/2.0)
			.show(ctx, |ui| {
				ui.add_space(10.0);
				egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
					ui.set_min_size(egui::Vec2::splat(300.0));
				});
			});
	}
}
