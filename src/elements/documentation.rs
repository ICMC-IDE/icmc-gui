use super::View;
use egui_dock::egui;
use egui_extras::{Column, TableBuilder};

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum Row {
    #[serde(rename = "header")]
    Header { title: String },

    #[serde(rename = "instruction")]
    Instruction {
        mnem: String,
        opcode: String,
        pseudo: String,
    },
}

pub struct Documentation;

impl Default for Documentation {
    fn default() -> Self {
        Self {}
    }
}

impl View for Documentation {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut egui::Context) {
        let rows = toml::from_str::<toml::Value>(include_str!("../../res/doc.toml"))
            .expect("Couldn't parse documentation TOML file")
            .get("row")
            .expect("Couldn't find entry 'instruction' in documentation file")
            .clone()
            .try_into::<Vec<Row>>()
            .expect("Couldn't parse instructions array from documentation file");

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(false)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Mnemonic");
                });
                header.col(|ui| {
                    ui.strong("Opcode");
                });
                header.col(|ui| {
                    ui.strong("Pseudo-Code");
                });
            })
            .body(|mut body| {
                for row in rows {
                    match row {
                        Row::Header { title } => {
                            body.row(22.0, |mut row| {
                                row.col(|ui| {
                                    ui.strong(title);
                                });
                            });
                        }
                        Row::Instruction {
                            mnem,
                            opcode,
                            pseudo,
                        } => {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(mnem);
                                });
                                row.col(|ui| {
                                    ui.label(opcode);
                                });
                                row.col(|ui| {
                                    ui.label(pseudo);
                                });
                            });
                        }
                    }
                }
            });
    }
}
