#[derive(Default, Clone, Copy, PartialEq)]
pub enum Radix {
    Binary,
    Decimal,
    #[default]
    Hex,
    Octal,
}
