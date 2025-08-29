#[derive(PartialEq)]
pub enum Radix {
    Binary,
    Decimal,
    Hex,
    Octal,
}

impl Default for Radix {
    fn default() -> Self {
        Self::Hex
    }
}
