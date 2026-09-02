#[derive(Clone)]
pub struct Charmap {
    char_width: usize,
    char_height: usize,
    color_palette: Vec<u8>, /* RGB palette (A -> FF, always) */
    pixels: Vec<u8>,        /* RGBA */
    bytes: Vec<u8>,         /* bin format */
    pub needs_reload: bool,
    pub needs_reload_editor: bool,
}

impl Default for Charmap {
    fn default() -> Self {
        Self::from_bytes(
            8,
            8,
            30,
            40,
            include_bytes!("../../res/charmap.bin").to_vec(),
        )
    }
}

impl Charmap {
    pub fn new(char_width: usize, char_height: usize, num_colors: usize, num_chars: usize) -> Self {
        let screen_width = char_width * num_colors;
        let screen_height = char_height * num_chars;

        Self {
            char_width,
            char_height,
            color_palette: vec![0; num_colors * 4],
            pixels: vec![0; num_chars * char_height],
            bytes: vec![0; screen_width * screen_height * 4],
            needs_reload: true,
            needs_reload_editor: true,
        }
    }

    pub fn from_bytes(
        char_width: usize,
        char_height: usize,
        num_colors: usize,
        num_chars: usize,
        bytes: Vec<u8>,
    ) -> Self {
        let mut charmap = Charmap::new(char_width, char_height, num_colors, num_chars);

        charmap.bytes = bytes;
        charmap.color_palette = Self::parse_palette(include_str!("../../res/8bit.json"));
        charmap.pixels = charmap.render_pixels();

        charmap
    }

    fn parse_palette(json_str: &str) -> Vec<u8> {
        json_str
            .trim()
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .flat_map(|entry| {
                let hex = entry.trim().trim_matches('"');
                let channel = |offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap();

                [channel(1), channel(3), channel(5), 0xff]
            })
            .collect()
    }

    fn render_pixels(&self) -> Vec<u8> {
        let num_colors = self.color_palette.len() / 4;
        let num_chars = self.bytes.len() / self.char_height;
        let screen_width = self.char_width * num_colors;
        let screen_height = self.char_height * num_chars;

        let mut pixels = vec![0u8; screen_width * screen_height * 4];

        for (color_idx, color) in self.color_palette.chunks_exact(4).enumerate() {
            let (mut r, mut g, mut b, a) = (color[0], color[1], color[2], color[3]);

            /* non-white colors are drawn inverted */
            if (r, g, b) != (0xff, 0xff, 0xff) {
                r ^= 0xff;
                g ^= 0xff;
                b ^= 0xff;
            }

            for char_idx in 0..num_chars {
                let row_start = char_idx * self.char_height;

                for byte_idx in 0..self.char_height {
                    let data_byte = self.bytes[row_start + byte_idx];
                    let y_pos = row_start + byte_idx;

                    for bit_offset in 0..self.char_width {
                        let bit_set = (data_byte >> (7 - bit_offset)) & 1 == 1;
                        let x_pos = color_idx * self.char_width + bit_offset;
                        let pixel = (y_pos * screen_width + x_pos) * 4;
                        let rgba = if bit_set {
                            [r, g, b, a]
                        } else {
                            [0, 0, 0, 0xff]
                        };

                        pixels[pixel..pixel + 4].copy_from_slice(&rgba);
                    }
                }
            }
        }

        pixels
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn char_width(&self) -> usize {
        self.char_width
    }

    pub fn char_height(&self) -> usize {
        self.char_height
    }

    pub fn num_colors(&self) -> usize {
        self.color_palette.len() / 4
    }

    pub fn num_chars(&self) -> usize {
        self.bytes.len() / self.char_height
    }

    pub fn palette_rgba(&self, color_index: usize) -> [u8; 4] {
        let o = color_index * 4;
        let (mut r, mut g, mut b, a) = (
            self.color_palette[o],
            self.color_palette[o + 1],
            self.color_palette[o + 2],
            self.color_palette[o + 3],
        );

        if (r, g, b) != (0xff, 0xff, 0xff) {
            r ^= 0xff;
            g ^= 0xff;
            b ^= 0xff;
        }

        [r, g, b, a]
    }

    pub fn toggle_pixel(&mut self, x: usize, y: usize) {
        if y >= self.bytes.len() {
            return;
        }

        let mask = 0b1000_0000u8 >> x.min(self.char_width - 1);
        self.bytes[y] ^= mask;
        self.pixels = self.render_pixels();
        self.needs_reload = true;
        self.needs_reload_editor = true;
    }
}
