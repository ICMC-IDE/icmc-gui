use egui_code_editor::Syntax;
use std::collections::BTreeSet;

pub fn icmc() -> Syntax {
    Syntax {
        language: "ICMC Assembly",
        case_sensitive: true,
        comment: ";",
        comment_multiline: ["/*", "*/"],
        hyperlinks: BTreeSet::from(["http"]),
        keywords: include_str!("../../res/syntax/keywords").lines().collect(),
        types: BTreeSet::from([]),
        special: include_str!("../../res/syntax/regs").lines().collect(),
    }
}
