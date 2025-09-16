use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Radix {
    Binary,
    Decimal,
    #[default]
    Hex,
    Octal,
}
