#[derive(Clone)]
pub struct Charmap {
    char_width: usize,
    char_height: usize,
    color_palette: Vec<u8>, /* RGB palette (A -> FF, always) */
    pixels: Vec<u8>,        /* RGBA */
    bytes: Vec<u8>,         /* bin format */
    pub needs_reload: bool,
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

        let json_str = include_str!("../../res/8bit.json");
        let clean = json_str
            .trim()
            .trim_start_matches("[")
            .trim_end_matches("]")
            .replace('"', "")
            .replace(|c: char| c.is_whitespace(), "");

        let hex_colors: Vec<&str> = clean.split(",").collect();

        let mut palette: Vec<u8> = Vec::with_capacity(hex_colors.len() * 4);

        for hex in hex_colors {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap();

            palette.extend_from_slice(&[r, g, b, 0xff]);
        }

        charmap.color_palette = palette;

        let num_colors = charmap.color_palette.len() / 4;
        let num_chars = charmap.bytes.len() / charmap.char_height;
        let screen_width = charmap.char_width * num_colors;
        let screen_height = charmap.char_height * num_chars;
        let mut pixels = vec![0u8; screen_height * screen_width * 4];

        for color_idx in 0..num_colors {
            let color_start_idx = color_idx * 4;
            let r_base = charmap.color_palette[color_start_idx];
            let g_base = charmap.color_palette[color_start_idx + 1];
            let b_base = charmap.color_palette[color_start_idx + 2];
            let a_base = charmap.color_palette[color_start_idx + 3];

            for char_idx in 0..num_chars {
                for byte_idx in 0..charmap.char_height {
                    let data_byte_index = char_idx * charmap.char_height + byte_idx;
                    let data_byte = charmap.bytes[data_byte_index];

                    for bit_offset in 0..charmap.char_width {
                        let bit_value = (data_byte >> (7 - bit_offset)) & 1;
                        let x_pos = (color_idx * charmap.char_width) + bit_offset;
                        let y_pos = (char_idx * charmap.char_height) + byte_idx;
                        let pixel_index = (y_pos * screen_width + x_pos) * 4;

                        if bit_value == 1 {
                            pixels[pixel_index] = r_base;
                            pixels[pixel_index + 1] = g_base;
                            pixels[pixel_index + 2] = b_base;
                            pixels[pixel_index + 3] = a_base;
                        } else {
                            pixels[pixel_index] = 0;
                            pixels[pixel_index + 1] = 0;
                            pixels[pixel_index + 2] = 0;
                            pixels[pixel_index + 3] = 0xFF;
                        }
                    }
                }
            }
        }

        charmap.pixels = pixels;

        charmap
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
