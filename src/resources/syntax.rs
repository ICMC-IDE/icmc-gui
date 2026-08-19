use egui_code_editor::Syntax;
use std::collections::BTreeSet;

pub fn icmc() -> Syntax {
    Syntax::new("ICMC Assembly")
        .with_comment(";")
        .with_keywords(include_str!("../../res/syntax/keywords").lines().collect::<BTreeSet<_>>())
        .with_special(include_str!("../../res/syntax/regs").lines().collect::<BTreeSet<_>>())
}
