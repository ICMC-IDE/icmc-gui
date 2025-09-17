use crate::resources::{charmap::Charmap, radix::Radix};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize)]
pub struct SettingsInner {
    pub font_size: f32,

    /// Screen size in characters
    pub screen_size: (u32, u32),

    pub radix: Radix,

    #[serde(skip)]
    pub charmap: Charmap,

    /* internal */
    #[serde(skip)]
    pub input_enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(flatten)]
    inner: SettingsInner,

    #[serde(skip)]
    pub needs_save: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            inner: SettingsInner {
                font_size: 14.0,
                screen_size: (40, 30),
                radix: Default::default(),
                charmap: Default::default(),
                input_enabled: false,
            },
            needs_save: false,
        }
    }
}

impl Settings {
    pub fn clear_save_flag(&mut self) {
        self.needs_save = false;
    }
}

impl Deref for Settings {
    type Target = SettingsInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Settings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.needs_save = true;
        &mut self.inner
    }
}
