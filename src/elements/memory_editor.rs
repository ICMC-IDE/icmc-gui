use super::ViewState;
use crate::State;
use egui_dock::egui;

pub struct MemEditor {
    hovered_index: Option<usize>,
    selected_index: Option<usize>,
    last_selected_index: Option<usize>,
    address: String,
    edit_hex: String,
    edit_octal: String,
    edit_signed: String,
    edit_unsigned: String,
    edit_binary: String,
}

impl Default for MemEditor {
    fn default() -> Self {
        Self {
            hovered_index: None,
            selected_index: Some(0),
            last_selected_index: None,
            address: String::new(),
            edit_hex: String::new(),
            edit_octal: String::new(),
            edit_signed: String::new(),
            edit_unsigned: String::new(),
            edit_binary: String::new(),
        }
    }
}

impl MemEditor {
    fn update_fields(
        &mut self,
        addr: usize,
        value: u16,
        skip_field: Option<&str>,
    ) {
        self.address = format!("0x{:04X}", addr);
        if skip_field != Some("hex") {
            self.edit_hex = format!("{:04X}", value);
        }
        if skip_field != Some("octal") {
            self.edit_octal = format!("{:06o}", value);
        }
        if skip_field != Some("signed") {
            self.edit_signed = format!("{}", value as i16);
        }
        if skip_field != Some("unsigned") {
            self.edit_unsigned = format!("{}", value);
        }
        if skip_field != Some("binary") {
            self.edit_binary = format!("{:016b}", value);
        }
    }

    pub fn export(&mut self) {}

    pub fn import(&mut self) {}
}

impl ViewState for MemEditor {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        ui.add_space(10.0);
        ui.set_min_size(ui.available_size());
        ui.set_max_size(ui.available_size());

        ui.horizontal(|ui| {
            if ui.button("Export").clicked() {
                self.export();
            }
            if ui.button("Import").clicked() {
                self.import();
            }
        });

        let mut emu = state.emulator.lock().unwrap();
        let ram_len = emu.ram().len();

        if self.selected_index != self.last_selected_index {
            if let Some(addr) = self.selected_index {
                if addr < ram_len {
                    let value = emu.ram()[addr];
                    self.update_fields(addr, value, None);
                }
            }
            self.last_selected_index = self.selected_index;
        }

        ui.columns(2, |columns| {
            columns[0].label("Address");
            columns[1].label("Value");

            columns[0].vertical(|ui| {
                egui::ScrollArea::vertical().show_rows(
                    ui,
                    1.0 / ui.available_height(),
                    ram_len,
                    |ui, row_range| {
                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing =
                                        egui::Vec2::splat(3.0);
                                    for i in row_range.clone() {
                                        let is_hovered =
                                            self.hovered_index == Some(i);
                                        let is_selected =
                                            self.selected_index == Some(i);

                                        let visuals = ui.visuals();

                                        let highlight_color = if visuals
                                            .dark_mode
                                        {
                                            egui::Color32::from_rgb(
                                                100, 150, 255,
                                            )
                                        } else {
                                            egui::Color32::from_rgb(0, 100, 200)
                                        };

                                        let hover_color = if visuals.dark_mode {
                                            egui::Color32::from_rgb(80, 80, 80)
                                        } else {
                                            egui::Color32::from_rgb(
                                                220, 220, 220,
                                            )
                                        };

                                        let text_color_on_highlight =
                                            if visuals.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::WHITE
                                            };

                                        let text_color_on_hover =
                                            if visuals.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::BLACK
                                            };

                                        let hex =
                                            format!("{:04X}", emu.ram()[i]);

                                        let style = if is_selected {
                                            egui::RichText::new(hex)
                                                .background_color(
                                                    highlight_color,
                                                )
                                                .color(text_color_on_highlight)
                                        } else if is_hovered {
                                            egui::RichText::new(hex)
                                                .background_color(hover_color)
                                                .color(text_color_on_hover)
                                        } else {
                                            egui::RichText::new(hex)
                                        };

                                        let response = ui.add(
                                            egui::Button::new(style)
                                                .frame(false),
                                        );
                                        if response.hovered() {
                                            self.hovered_index = Some(i);
                                        }
                                        if response.clicked() {
                                            self.selected_index = Some(i);
                                        }
                                    }
                                });
                            });
                            columns[1].vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing =
                                        egui::Vec2::splat(3.0);
                                    for i in row_range.clone() {
                                        let is_hovered =
                                            self.hovered_index == Some(i);
                                        let is_selected =
                                            self.selected_index == Some(i);

                                        let visuals = ui.visuals();

                                        let highlight_color = if visuals
                                            .dark_mode
                                        {
                                            egui::Color32::from_rgb(
                                                100, 150, 255,
                                            )
                                        } else {
                                            egui::Color32::from_rgb(0, 100, 200)
                                        };

                                        let hover_color = if visuals.dark_mode {
                                            egui::Color32::from_rgb(80, 80, 80)
                                        } else {
                                            egui::Color32::from_rgb(
                                                220, 220, 220,
                                            )
                                        };

                                        let text_color_on_highlight =
                                            if visuals.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::WHITE
                                            };

                                        let text_color_on_hover =
                                            if visuals.dark_mode {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::BLACK
                                            };

                                        let byte = emu.ram()[i] as u8;
                                        let ch = if (0x20..=0x7E).contains(&byte)
                                        {
                                            byte as char
                                        } else {
                                            '.'
                                        };

                                        let style = if is_selected {
                                            egui::RichText::new(ch)
                                                .background_color(
                                                    highlight_color,
                                                )
                                                .color(text_color_on_highlight)
                                        } else if is_hovered {
                                            egui::RichText::new(ch)
                                                .background_color(hover_color)
                                                .color(text_color_on_hover)
                                        } else {
                                            egui::RichText::new(ch)
                                        };

                                        let response = ui.add(
                                            egui::Button::new(style)
                                                .frame(false),
                                        );
                                        if response.hovered() {
                                            self.hovered_index = Some(i);
                                        }
                                        if response.clicked() {
                                            self.selected_index = Some(i);
                                        }
                                    }
                                });
                            });
                        });
                    },
                );
            });

            columns[1].vertical(|ui| {
                if let Some(addr) = self.selected_index {
                    if addr < ram_len {
                        ui.label("Address");
                        let response =
                            ui.text_edit_singleline(&mut self.address);
                        let enter_pressed =
                            ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if response.lost_focus() || enter_pressed {
                            let addr_str =
                                self.address.trim_start_matches("0x");
                            if let Ok(new_addr) =
                                u16::from_str_radix(addr_str, 16)
                            {
                                if (new_addr as usize) < ram_len {
                                    self.selected_index =
                                        Some(new_addr as usize);
                                    self.last_selected_index = None; // força update dos campos
                                    self.address =
                                        format!("0x{:04X}", new_addr);
                                    // formata como 0xABCD
                                }
                            }
                        }

                        ui.label("Instruction");
                        ui.add_enabled(
                            false,
                            egui::TextEdit::singleline(&mut "-".to_string()),
                        );

                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.label("Hex");
                                if ui
                                    .text_edit_singleline(&mut self.edit_hex)
                                    .changed()
                                {
                                    if let Ok(val) = u16::from_str_radix(
                                        &self.edit_hex.trim(),
                                        16,
                                    ) {
                                        emu.store(addr as u16, val);
                                        self.update_fields(
                                            addr,
                                            val,
                                            Some("hex"),
                                        );
                                    }
                                }
                            });
                            columns[1].vertical(|ui| {
                                ui.label("Octal");
                                if ui
                                    .text_edit_singleline(&mut self.edit_octal)
                                    .changed()
                                {
                                    if let Ok(val) = u16::from_str_radix(
                                        &self.edit_octal.trim(),
                                        8,
                                    ) {
                                        emu.store(addr as u16, val);
                                        self.update_fields(
                                            addr,
                                            val,
                                            Some("octal"),
                                        );
                                    }
                                }
                            });
                        });

                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                ui.label("Signed");
                                if ui
                                    .text_edit_singleline(&mut self.edit_signed)
                                    .changed()
                                {
                                    if let Ok(val) =
                                        self.edit_signed.trim().parse::<i16>()
                                    {
                                        emu.store(addr as u16, val as u16);
                                        self.update_fields(
                                            addr,
                                            val as u16,
                                            Some("signed"),
                                        );
                                    }
                                }
                            });
                            columns[1].vertical(|ui| {
                                ui.label("Unsigned");
                                if ui
                                    .text_edit_singleline(
                                        &mut self.edit_unsigned,
                                    )
                                    .changed()
                                {
                                    if let Ok(val) =
                                        self.edit_unsigned.trim().parse::<u16>()
                                    {
                                        emu.store(addr as u16, val);
                                        self.update_fields(
                                            addr,
                                            val,
                                            Some("unsigned"),
                                        );
                                    }
                                }
                            });
                        });

                        ui.label("Binary");
                        if ui
                            .text_edit_singleline(&mut self.edit_binary)
                            .changed()
                        {
                            if let Ok(val) =
                                u16::from_str_radix(&self.edit_binary.trim(), 2)
                            {
                                emu.store(addr as u16, val);
                                self.update_fields(addr, val, Some("binary"));
                            }
                        }
                    }
                }
            });
        });
    }
}
