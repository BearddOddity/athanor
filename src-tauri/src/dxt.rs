//! DXT1/DXT5 texture decompression
//!
//! Converts block-compressed DXT data to RGBA for PNG export.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DxtFormat {
    Dxt1,
    Dxt5,
}

impl DxtFormat {
    pub fn block_size(&self) -> usize {
        match self {
            DxtFormat::Dxt1 => 8,
            DxtFormat::Dxt5 => 16,
        }
    }
}

pub struct DecodedTexture {
    pub width: u32,
    pub height: u32,
    pub format: DxtFormat,
    pub pixels: Vec<u8>, // RGBA
}

fn expand_rgb565(c: u16) -> [u8; 4] {
    let r = ((c >> 11) & 0x1F) as u8;
    let g = ((c >> 5) & 0x3F) as u8;
    let b = (c & 0x1F) as u8;
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
        0xFF,
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn color_lerp(c0: [u8; 4], c1: [u8; 4], t: f32) -> [u8; 4] {
    [
        lerp(c0[0] as f32, c1[0] as f32, t).round().clamp(0.0, 255.0) as u8,
        lerp(c0[1] as f32, c1[1] as f32, t).round().clamp(0.0, 255.0) as u8,
        lerp(c0[2] as f32, c1[2] as f32, t).round().clamp(0.0, 255.0) as u8,
        lerp(c0[3] as f32, c1[3] as f32, t).round().clamp(0.0, 255.0) as u8,
    ]
}

pub fn decompress(data: &[u8], width: u32, height: u32, format: DxtFormat) -> Result<Vec<u8>, crate::AthanorError> {
    let block_size = format.block_size();
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let expected = (blocks_x * blocks_y) as usize * block_size;

    if data.len() < expected {
        return Err(crate::AthanorError::InvalidData(format!(
            "DXT data too short: got {} bytes, expected {}",
            data.len(),
            expected
        )));
    }

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let mut idx = 0usize;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            match format {
                DxtFormat::Dxt1 => {
                    decode_dxt1_block(data, idx, &mut pixels, width, height, bx, by);
                }
                DxtFormat::Dxt5 => {
                    decode_dxt5_block(data, idx, &mut pixels, width, height, bx, by);
                }
            }
            idx += block_size;
        }
    }

    Ok(pixels)
}

pub fn compress(pixels: &[u8], width: u32, height: u32, format: DxtFormat) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let block_size = format.block_size();
    let mut output = vec![0u8; (blocks_x * blocks_y * block_size as u32) as usize];

    let mut idx = 0usize;
    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            match format {
                DxtFormat::Dxt1 => {
                    encode_dxt1_block(pixels, width, height, bx, by, &mut output, idx);
                }
                DxtFormat::Dxt5 => {
                    encode_dxt5_block(pixels, width, height, bx, by, &mut output, idx);
                }
            }
            idx += block_size;
        }
    }

    output
}

fn decode_dxt1_block(
    data: &[u8],
    offset: usize,
    colors: &mut [u8],
    width: u32,
    height: u32,
    block_x: u32,
    block_y: u32,
) {
    let c0 = u16::from_le_bytes([data[offset], data[offset + 1]]);
    let c1 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);

    let col0 = expand_rgb565(c0);
    let col1 = expand_rgb565(c1);

    let mut palette = [col0, col1, [0u8; 4], [0u8; 4]];
    if c0 > c1 {
        palette[2] = color_lerp(col0, col1, 1.0 / 3.0);
        palette[3] = color_lerp(col0, col1, 2.0 / 3.0);
    } else {
        palette[2] = color_lerp(col0, col1, 0.5);
        palette[3] = [0, 0, 0, 0];
    }

    let indices = u32::from_le_bytes([
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    for row in 0..4u32 {
        for col in 0..4u32 {
            let px = block_x * 4 + col;
            let py = block_y * 4 + row;
            if px >= width || py >= height {
                continue;
            }
            let bit_pos = (row * 4 + col) * 2;
            let index = ((indices >> bit_pos) & 0x3) as usize;
            let pixel = palette[index.min(3)];
            let px_idx = ((py * width + px) * 4) as usize;
            colors[px_idx..px_idx + 4].copy_from_slice(&pixel);
        }
    }
}

fn decode_dxt5_alpha(data: &[u8], offset: usize) -> [u8; 16] {
    let a0 = data[offset];
    let a1 = data[offset + 1];

    let mut alpha_vals = [0u8; 16];
    alpha_vals[0] = a0;
    alpha_vals[1] = a1;

    let mut bits: u64 = 0;
    for i in 0..6 {
        bits |= (data[offset + 2 + i] as u64) << (i * 8);
    }

    for i in 0..16 {
        let code = ((bits >> (i * 3)) & 0x7) as u8;
        let a = a0 as u32;
        let b = a1 as u32;
        alpha_vals[i] = match (a0, a1) {
            (a0_val, b_val) if a0_val > b_val => match code {
                0 => a0_val,
                1 => b_val,
                2 => ((2 * a0_val + b_val) / 3) as u8,
                3 => ((a0_val + 2 * b_val) / 3) as u8,
                4 => ((3 * a0_val + b_val) / 4) as u8,
                5 => ((2 * a0_val + 2 * b_val) / 4) as u8,
                6 => ((a0_val + 3 * b_val) / 4) as u8,
                _ => b_val,
            },
            (a0_val, b_val) => match code {
                0 => a0_val,
                1 => b_val,
                2 => ((2 * a0_val + b_val) / 3) as u8,
                3 => ((a0_val + 2 * b_val) / 3) as u8,
                4 => ((3 * a0_val + b_val) / 4) as u8,
                5 => ((2 * a0_val + 2 * b_val) / 4) as u8,
                6 => ((a0_val + 3 * b_val) / 4) as u8,
                _ => 0,
            },
        };
    }
    alpha_vals
}

fn decode_dxt5_block(
    data: &[u8],
    offset: usize,
    colors: &mut [u8],
    width: u32,
    height: u32,
    block_x: u32,
    block_y: u32,
) {
    let alpha_vals = decode_dxt5_alpha(data, offset);
    decode_dxt1_block(data, offset + 8, colors, width, height, block_x, block_y);

    for row in 0..4u32 {
        for col in 0..4u32 {
            let px = block_x * 4 + col;
            let py = block_y * 4 + row;
            if px >= width || py >= height {
                continue;
            }
            let px_idx = ((py * width + px) * 4) as usize;
            colors[px_idx + 3] = alpha_vals[(row * 4 + col) as usize];
        }
    }
}

fn encode_dxt1_block(
    pixels: &[u8],
    width: u32,
    height: u32,
    block_x: u32,
    block_y: u32,
    output: &mut [u8],
    offset: usize,
) {
    let mut colors = [[0u8; 4]; 16];
    for row in 0..4u32 {
        for col in 0..4u32 {
            let px = block_x * 4 + col;
            let py = block_y * 4 + row;
            if px < width && py < height {
                let src_idx = ((py * width + px) * 4) as usize;
                colors[(row * 4 + col) as usize] = [
                    pixels[src_idx],
                    pixels[src_idx + 1],
                    pixels[src_idx + 2],
                    pixels[src_idx + 3],
                ];
            }
        }
    }

    let c0 = rgb_to_rgb565(colors[0][0], colors[0][1], colors[0][2]);
    let c1 = rgb_to_rgb565(colors[15][0], colors[15][1], colors[15][2]);

    output[offset..offset + 2].copy_from_slice(&c0.to_le_bytes());
    output[offset + 2..offset + 4].copy_from_slice(&c1.to_le_bytes());

    let mut indices: u32 = 0;
    for i in 0..16 {
        let idx = if c0 > c1 {
            if colors[i] == colors[0] {
                0
            } else if colors[i] == colors[1] {
                1
            } else if colors[i][0] == ((2 * colors[0][0] as u32 + colors[1][0] as u32) / 3) as u8
                && colors[i][1] == ((2 * colors[0][1] as u32 + colors[1][1] as u32) / 3) as u8
                && colors[i][2] == ((2 * colors[0][2] as u32 + colors[1][2] as u32) / 3) as u8
            {
                2
            } else {
                3
            }
        } else {
            0
        };
        indices |= (idx as u32) << (i * 2);
    }
    output[offset + 4..offset + 8].copy_from_slice(&indices.to_le_bytes());
}

fn encode_dxt5_block(
    pixels: &[u8],
    width: u32,
    height: u32,
    block_x: u32,
    block_y: u32,
    output: &mut [u8],
    offset: usize,
) {
    let mut alphas = [0u8; 16];
    for row in 0..4u32 {
        for col in 0..4u32 {
            let px = block_x * 4 + col;
            let py = block_y * 4 + row;
            if px < width && py < height {
                let src_idx = ((py * width + px) * 4) as usize;
                alphas[(row * 4 + col) as usize] = pixels[src_idx + 3];
            }
        }
    }

    output[offset] = alphas[0];
    output[offset + 1] = alphas[15];

    let mut bits: u64 = 0;
    for i in 0..16 {
        let code = if u32::from(alphas[i]) > (alphas[0] as u32 + alphas[15] as u32) / 2 {
            0
        } else {
            1
        };
        bits |= (code as u64) << (i * 3);
    }
    for i in 0..6 {
        output[offset + 2 + i] = (bits >> (i * 8)) as u8;
    }

    let colors = &mut output[offset + 8..];
    encode_dxt1_block(pixels, width, height, block_x, block_y, colors, 0);
}

fn rgb_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    (((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)) as u16
}