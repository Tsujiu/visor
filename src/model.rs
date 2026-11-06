//! Tipos que a interface e os codecs compartilham.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Support {
    Full,
    ReadOnly,
    Partial,
    External,
    Unsupported,
}

impl Support {
    pub fn label(self) -> &'static str {
        match self {
            Self::Full => "completo",
            Self::ReadOnly => "somente leitura",
            Self::Partial => "parcial",
            Self::External => "conversor externo",
            Self::Unsupported => "não suportado",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Raster,
    Texture,
    Vector,
    Raw,
    Animation,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::Raster => "Raster",
            Self::Texture => "Texturas e jogos",
            Self::Vector => "Vetoriais",
            Self::Raw => "RAW",
            Self::Animation => "Animações",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

impl Frame {
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y as usize * self.width as usize) + x as usize) * 4;
        self.rgba.get(i..i + 4).map(|p| [p[0], p[1], p[2], p[3]])
    }
}

#[derive(Clone, Debug)]
pub struct ImageInfo {
    pub id: &'static str,
    pub name: String,
    pub category: Category,
    pub support: Support,
    pub bit_depth: u8,
    pub color: String,
    pub original_w: u32,
    pub original_h: u32,
    pub display_w: u32,
    pub display_h: u32,
    pub reduced: bool,
    pub has_alpha: bool,
    pub frame_count: usize,
    pub loop_forever: bool,
    pub mipmaps: u32,
    pub exif: Vec<(String, String)>,
    pub icc: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Decoded {
    pub frames: Vec<Frame>,
    pub info: ImageInfo,
}

impl Decoded {
    pub fn current(&self) -> Option<&Frame> {
        self.frames.first()
    }
}

pub(crate) struct Built {
    pub frames: Vec<Frame>,
    pub bit_depth: u8,
    pub color: String,
    pub mipmaps: u32,
    pub notes: Vec<String>,
    pub id: &'static str,
    pub loop_forever: bool,
}

pub(crate) enum Master {
    Still {
        image: image::DynamicImage,
        bit_depth: u8,
        color: String,
        mipmaps: u32,
        notes: Vec<String>,
        id: &'static str,
    },
    Frames(Built),
}

pub const MAX_DIM: u32 = 40_000;
pub const MAX_PIXELS: u64 = 80_000_000;
pub const MAX_ALLOC: u64 = 512 * 1024 * 1024;
pub const MAX_FILE: u64 = 512 * 1024 * 1024;
pub const MAX_SIDE: u32 = 8192;
pub const MAX_FRAMES: usize = 180;

pub fn check_dims(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Err(format!("dimensões rejeitadas ({width}×{height})"));
    }
    if width as u64 * height as u64 > MAX_PIXELS {
        return Err("imagem acima do limite de 80 megapixels".to_owned());
    }
    Ok(())
}

pub fn has_alpha(pixels: &[u8]) -> bool {
    pixels.chunks_exact(4).any(|px| px[3] != 255)
}

pub fn downscale(frame: Frame, max_side: u32) -> (Frame, bool) {
    let longest = frame.width.max(frame.height);
    if longest <= max_side || longest == 0 {
        return (frame, false);
    }
    let Frame {
        rgba,
        width,
        height,
        delay_ms,
    } = frame;
    let expected = width as usize * height as usize * 4;
    if rgba.len() != expected {
        return (
            Frame {
                rgba,
                width,
                height,
                delay_ms,
            },
            false,
        );
    }
    let buffer = image::RgbaImage::from_raw(width, height, rgba).expect("o tamanho já foi conferido");
    let scale = max_side as f32 / longest as f32;
    let tw = ((width as f32) * scale).round().max(1.0) as u32;
    let th = ((height as f32) * scale).round().max(1.0) as u32;
    let small = image::imageops::thumbnail(&buffer, tw, th);
    let (width, height) = small.dimensions();
    (
        Frame {
            rgba: small.into_raw(),
            width,
            height,
            delay_ms,
        },
        true,
    )
}

pub fn bgra_to_rgba(pixels: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        let bgra = pixel.to_le_bytes();
        out.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
    }
    out
}
