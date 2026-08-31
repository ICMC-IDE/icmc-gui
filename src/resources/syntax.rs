use egui_code_editor::Syntax;
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub fn icmc() -> &'static Syntax {
    static SYNTAX: OnceLock<Syntax> = OnceLock::new();
    SYNTAX.get_or_init(|| {
        Syntax::new("ICMC Assembly")
            .with_comment(";")
            .with_keywords(include_str!("../../res/syntax/keywords").lines().collect::<BTreeSet<_>>())
            .with_special(include_str!("../../res/syntax/regs").lines().collect::<BTreeSet<_>>())
    })
}
