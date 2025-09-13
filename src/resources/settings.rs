use crate::resources::radix::Radix;

pub struct Settings {
    pub font_size: f32,

    /// Screen size in characters
    pub screen_size: (u32, u32),

    pub radix: Radix,

    /* internal */
    pub input_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            screen_size: (40, 30),
            radix: Default::default(),
            input_enabled: false,
        }
    }
}
