// SPDX-License-Identifier: GPL-3.0-only
//! Lightweight Minecraft skin rendering.
//!
//! A skin is a 64×64 (or legacy 64×32) PNG texture. Aero decodes it once, composes
//! a small standing character image on the CPU and hands the encoded PNG to Skia.
//! No 3D engine, no GPU skinning, no per-frame work: the composition happens only
//! when the account or skin changes.
use anyhow::{Context, Result, ensure};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct Skin {
    size: u32,
    pixels: Vec<u8>,
}
impl Skin {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(
            !bytes.is_empty() && bytes.len() <= 2 * 1024 * 1024,
            "Skin file has an invalid size"
        );
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().context("Skin is not a valid PNG")?;
        let info = reader.info().clone();
        ensure!(
            (info.width == 64 && (info.height == 64 || info.height == 32)),
            "Unsupported skin dimensions"
        );
        let mut buffer = vec![0; reader.output_buffer_size()];
        let frame = reader
            .next_frame(&mut buffer)
            .context("Skin PNG is unreadable")?;
        let channels = match frame.color_type {
            png::ColorType::Rgba => 4,
            png::ColorType::Rgb => 3,
            _ => anyhow::bail!("Unsupported skin colour type"),
        };
        let mut pixels = vec![0u8; (info.width * info.height * 4) as usize];
        for index in 0..(info.width * info.height) as usize {
            let source = index * channels;
            pixels[index * 4] = buffer[source];
            pixels[index * 4 + 1] = buffer[source + 1];
            pixels[index * 4 + 2] = buffer[source + 2];
            pixels[index * 4 + 3] = if channels == 4 {
                buffer[source + 3]
            } else {
                255
            };
        }
        Ok(Self {
            size: info.width,
            pixels,
        })
    }

    fn at(&self, x: i32, y: i32) -> [u8; 4] {
        let width = self.size as i32;
        if x < 0 || y < 0 || x >= width || y >= self.size as i32 {
            return [0, 0, 0, 0];
        }
        let offset = ((y * width + x) * 4) as usize;
        [
            self.pixels[offset],
            self.pixels[offset + 1],
            self.pixels[offset + 2],
            self.pixels[offset + 3],
        ]
    }

    /// Legacy 64×32 skins have no dedicated arm/leg side faces; reuse the front.
    fn legacy(&self) -> bool {
        self.size == 32
    }

    /// Original Aero fallback skin, generated from the palette: a light-blue
    /// explorer hoodie with dark trousers. Used while signed out.
    pub fn fallback() -> Self {
        let mut skin = Self {
            size: 64,
            pixels: vec![0; 64 * 64 * 4],
        };
        let ink = [30, 58, 74, 255];
        let skin_tone = [232, 190, 154, 255];
        let hair = [58, 40, 30, 255];
        let hoodie = [24, 42, 54, 255];
        let aqua = [96, 208, 240, 255];
        let white = [244, 250, 252, 255];
        let denim = [38, 56, 74, 255];
        let shoe = [236, 242, 246, 255];
        let mut fill = |x: u32, y: u32, w: u32, h: u32, color: [u8; 4]| {
            for py in y..y + h {
                for px in x..x + w {
                    let offset = ((py * 64 + px) * 4) as usize;
                    skin.pixels[offset..offset + 4].copy_from_slice(&color);
                }
            }
        };
        // Head base + hair cap + face.
        fill(8, 8, 8, 8, skin_tone);
        fill(8, 8, 8, 3, hair);
        fill(8, 8, 1, 4, hair);
        fill(15, 8, 1, 4, hair);
        fill(10, 12, 1, 1, ink);
        fill(13, 12, 1, 1, ink);
        fill(11, 14, 2, 1, [176, 122, 106, 255]);
        fill(24, 8, 8, 8, hair); // back of the head
        fill(0, 8, 8, 8, hair); // right side
        fill(16, 8, 8, 8, hair); // left side
        fill(8, 0, 8, 8, hair); // top
        // Hoodie: body, sleeves, collar.
        fill(20, 20, 8, 12, hoodie);
        fill(20, 20, 8, 2, aqua);
        fill(23, 22, 2, 8, aqua);
        fill(16, 20, 4, 12, hoodie);
        fill(16, 20, 4, 2, aqua);
        fill(28, 20, 4, 12, hoodie);
        fill(28, 20, 4, 2, aqua);
        fill(44, 20, 4, 12, hoodie);
        fill(44, 20, 4, 3, aqua);
        fill(48, 20, 4, 12, hoodie);
        fill(36, 52, 4, 12, hoodie);
        fill(36, 52, 4, 3, aqua);
        fill(32, 52, 4, 12, hoodie);
        // Trousers and shoes.
        fill(4, 20, 4, 12, denim);
        fill(0, 20, 4, 12, denim);
        fill(4, 29, 4, 3, shoe);
        fill(8, 20, 4, 12, denim);
        fill(8, 29, 4, 3, shoe);
        fill(20, 52, 4, 12, denim);
        fill(16, 52, 4, 12, denim);
        fill(20, 61, 4, 3, shoe);
        fill(24, 52, 4, 12, denim);
        fill(24, 61, 4, 3, shoe);
        // Hat brim accent and cuffs.
        fill(40, 20, 4, 3, white);
        fill(40, 29, 4, 3, white);
        // Keep every hidden/overlay face opaque so no transparent seams appear.
        for pixel in skin.pixels.chunks_exact_mut(4) {
            if pixel[3] == 0 {
                pixel.copy_from_slice(&[0, 0, 0, 0]);
            }
        }
        skin
    }
}

const UNIT: i32 = 12; // screen pixels per skin pixel
const MARGIN: i32 = 12;
const WIDTH: i32 = MARGIN * 2 + UNIT * 16;
const HEIGHT: i32 = MARGIN * 2 + UNIT * 32;

struct Canvas {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}
impl Canvas {
    fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }
    fn blend(&mut self, x: i32, y: i32, color: [u8; 4], scale: f32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || color[3] == 0 {
            return;
        }
        let alpha = (color[3] as f32 / 255. * scale).clamp(0., 1.);
        let offset = ((y * self.width + x) * 4) as usize;
        let destination = &mut self.pixels[offset..offset + 4];
        for channel in 0..3 {
            destination[channel] = (color[channel] as f32 * alpha
                + destination[channel] as f32 * (1. - alpha))
                .round() as u8;
        }
        destination[3] = (255. * alpha + destination[3] as f32 * (1. - alpha)).round() as u8;
    }

    /// Copy a skin face, nearest-neighbour, at an arbitrary rectangle.
    fn face(
        &mut self,
        skin: &Skin,
        face: (i32, i32, i32, i32),
        target: (i32, i32, i32, i32),
        shade: f32,
    ) {
        for py in 0..target.3 {
            for px in 0..target.2 {
                let sx = face.0 + px * face.2 / target.2.max(1);
                let sy = face.1 + py * face.3 / target.3.max(1);
                let mut color = skin.at(sx, sy);
                for channel in 0..3 {
                    color[channel] = (color[channel] as f32 * shade).clamp(0., 255.) as u8;
                }
                self.blend(target.0 + px, target.1 + py, color, 1.);
            }
        }
    }

    fn shadow(&mut self, center_x: i32, center_y: i32) {
        for row in 0..=8i32 {
            let half = 54 - row * 3;
            for x in -half..=half {
                let fade = 1. - (row as f32 / 9.);
                let spread = 1. - (x.abs() as f32 / half.max(1) as f32).powi(2);
                let alpha = 0.28 * fade * spread;
                self.blend(center_x + x, center_y + row, [22, 46, 58, 255], alpha);
            }
        }
    }
}

/// Compose the standing character used on the home screen.
pub fn model(skin: &Skin) -> Vec<u8> {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let column = MARGIN + UNIT * 4; // body/head column, 8 skin pixels wide
    let body_top = MARGIN + UNIT * 8;
    let legs_top = body_top + UNIT * 12;
    let arm_y = body_top;
    let depth = 5; // fake side plane, keeps a hint of three dimensions
    canvas.shadow(MARGIN + UNIT * 8, legs_top + UNIT * 12 - 2);

    // Legs (back first), then arms, body and head; each part gets a darker side
    // plane and its top face so the silhouette reads as a Minecraft model.
    let leg_side = if skin.legacy() {
        (4, 20, 4, 12)
    } else {
        (0, 20, 4, 12)
    };
    let leg_left_side = if skin.legacy() {
        (4, 20, 4, 12)
    } else {
        (16, 52, 4, 12)
    };
    canvas.face(
        skin,
        (4, 20, 4, 12),
        (column, legs_top, UNIT * 4, UNIT * 12),
        0.86,
    );
    canvas.face(
        skin,
        leg_side,
        (column + UNIT * 4, legs_top, depth, UNIT * 12),
        0.62,
    );
    canvas.face(
        skin,
        (20, 52, 4, 12),
        (column + UNIT * 4, legs_top, UNIT * 4, UNIT * 12),
        0.9,
    );
    canvas.face(
        skin,
        leg_left_side,
        (column + UNIT * 8, legs_top, depth, UNIT * 12),
        0.66,
    );

    // Arms hang slightly outside the body, as in the reference composition.
    canvas.face(
        skin,
        (44, 20, 4, 12),
        (column - UNIT * 4, arm_y + 2, UNIT * 4, UNIT * 12),
        0.94,
    );
    canvas.face(
        skin,
        (40, 20, 4, 12),
        (column - UNIT * 4 - depth, arm_y + 2, depth, UNIT * 12),
        0.68,
    );
    canvas.face(
        skin,
        (36, 52, 4, 12),
        (column + UNIT * 8, arm_y + 2, UNIT * 4, UNIT * 12),
        0.82,
    );
    canvas.face(
        skin,
        (32, 52, 4, 12),
        (column + UNIT * 12, arm_y + 2, depth, UNIT * 12),
        0.6,
    );

    // Body.
    canvas.face(
        skin,
        (20, 20, 8, 12),
        (column, body_top, UNIT * 8, UNIT * 12),
        1.,
    );
    canvas.face(
        skin,
        (16, 20, 4, 12),
        (column + UNIT * 8, body_top, depth, UNIT * 12),
        0.64,
    );
    canvas.face(
        skin,
        (20, 16, 8, 4),
        (column, body_top - depth, UNIT * 8, depth),
        1.08,
    );

    // Head with a top plane so the face sits slightly behind the crown.
    let head_top = MARGIN;
    canvas.face(
        skin,
        (0, 8, 8, 8),
        (column - depth, head_top, depth, UNIT * 8),
        0.7,
    );
    canvas.face(
        skin,
        (8, 0, 8, 8),
        (column, head_top - depth, UNIT * 8, depth),
        1.1,
    );
    canvas.face(
        skin,
        (8, 8, 8, 8),
        (column, head_top, UNIT * 8, UNIT * 8),
        1.,
    );
    canvas.face(
        skin,
        (16, 8, 8, 8),
        (column + UNIT * 8, head_top, depth, UNIT * 8),
        0.62,
    );
    // A subtle rim light on the silhouette edge keeps the model crisp on the sky.
    for y in 0..(UNIT * 8) {
        canvas.blend(
            column + UNIT * 8 + depth,
            head_top + y,
            [255, 255, 255, 90],
            1.,
        );
    }
    encode_png(canvas.width as u32, canvas.height as u32, &canvas.pixels)
}

/// Minimal PNG writer. Uses stored (uncompressed) deflate blocks so the renderer
/// needs no compression dependency; the composed model is ~150 KB at most.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width * height * 4 + height) as usize);
    for row in 0..height {
        raw.push(0);
        let start = (row * width * 4) as usize;
        raw.extend_from_slice(&rgba[start..start + (width * 4) as usize]);
    }
    let mut zlib = vec![0x78, 0x01];
    let mut offset = 0;
    while offset < raw.len() {
        let size = (raw.len() - offset).min(65_535);
        let last = offset + size == raw.len();
        zlib.push(if last { 1 } else { 0 });
        zlib.extend_from_slice(&(size as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(size as u16)).to_le_bytes());
        zlib.extend_from_slice(&raw[offset..offset + size]);
        offset += size;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    png
}

fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = kind.to_vec();
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + *byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    crc ^ 0xFFFF_FFFF
}
