pub struct Charmap {
    width: usize,
    height: usize,
    char_width: usize,
    char_height: usize,
    pixels: Vec<u8>, /* RGBA */
    bytes: Vec<u8>,  /* bin format */
}

impl Charmap {
    pub fn new(char_width: usize, char_height: usize, columns: usize, rows: usize) -> Self {
        let width = char_width * columns;
        let height = char_height * rows;

        Self {
            width,
            height,
            char_width,
            char_height,
            pixels: vec![0; 2048 * 2048 * 4],
            bytes: vec![0; char_height * rows * columns],
        }
    }

    pub fn from_bytes(
        char_width: usize,
        char_height: usize,
        columns: usize,
        rows: usize,
        data: &[u8],
    ) -> Self {
        let mut charmap = Charmap::new(char_width, char_height, columns, rows);

        for c_idx in 0..256 {
            for row in 0..8 {
                let byte = data[c_idx * 8 + row];
                for col in 0..8 {
                    let bit = (byte >> (7 - col)) & 1;
                    if bit == 1 {
                        let x = (c_idx % 256) * 8 + col;
                        let y = (c_idx / 256) * 8 + row;
                        let offset = (y * (8 * 40) + x) * 4;
                        charmap.pixels[offset..offset + 4].copy_from_slice(&[0xff; 4]);
                    }
                }
            }
        }

        charmap
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
