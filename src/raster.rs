//! Raster comum e animações que o crate image já decodifica.

use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::{AnimationDecoder, ColorType, DynamicImage, ImageDecoder as _, ImageFormat, Limits};

use crate::model::{check_dims, Frame, Master, MAX_ALLOC, MAX_FRAMES};
use crate::sniff::Kind;

pub fn limits_of() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(crate::model::MAX_DIM);
    limits.max_image_height = Some(crate::model::MAX_DIM);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

pub fn describe_color(color: ColorType) -> (u8, String) {
    let (depth, name) = match color {
        ColorType::L8 => (8, "escala de cinza"),
        ColorType::La8 => (8, "cinza com alfa"),
        ColorType::Rgb8 => (8, "RGB"),
        ColorType::Rgba8 => (8, "RGBA"),
        ColorType::L16 => (16, "escala de cinza"),
        ColorType::La16 => (16, "cinza com alfa"),
        ColorType::Rgb16 => (16, "RGB"),
        ColorType::Rgba16 => (16, "RGBA"),
        ColorType::Rgb32F => (32, "RGB ponto flutuante"),
        ColorType::Rgba32F => (32, "RGBA ponto flutuante"),
        _ => (8, "cor"),
    };
    (depth, name.to_owned())
}

pub fn decode_path(path: &Path, kind: Kind) -> Result<Master, String> {
    match kind {
        Kind::Gif => decode_gif(path),
        Kind::Png => decode_png(path),
        Kind::Webp => decode_webp(path),
        _ => decode_still(path, image_format(kind)),
    }
}

fn image_format(kind: Kind) -> ImageFormat {
    match kind {
        Kind::Png => ImageFormat::Png,
        Kind::Jpeg => ImageFormat::Jpeg,
        Kind::Gif => ImageFormat::Gif,
        Kind::Bmp => ImageFormat::Bmp,
        Kind::Tiff => ImageFormat::Tiff,
        Kind::Webp => ImageFormat::WebP,
        Kind::Ico => ImageFormat::Ico,
        Kind::Tga => ImageFormat::Tga,
        Kind::Qoi => ImageFormat::Qoi,
        Kind::Pnm => ImageFormat::Pnm,
        Kind::Hdr => ImageFormat::Hdr,
        Kind::Exr => ImageFormat::OpenExr,
        _ => ImageFormat::Png,
    }
}

fn id_of(kind: Kind) -> &'static str {
    match kind {
        Kind::Jpeg => "jpeg",
        Kind::Bmp => "bmp",
        Kind::Tiff => "tiff",
        Kind::Ico => "ico",
        Kind::Tga => "tga",
        Kind::Qoi => "qoi",
        Kind::Pnm => "pnm",
        Kind::Hdr => "hdr",
        Kind::Exr => "exr",
        Kind::Webp => "webp",
        Kind::Png => "png",
        Kind::Gif => "gif",
        _ => "png",
    }
}

fn decode_still(path: &Path, format: ImageFormat) -> Result<Master, String> {
    match decode_still_inner(path, format) {
        Ok(master) => Ok(master),
        Err(err) if format == ImageFormat::Ico => std::fs::read(path)
            .map_err(|read| format!("não abriu o arquivo: {read}"))
            .and_then(|bytes| crate::simple::decode_ico(&bytes))
            .map(Master::Frames)
            .map_err(|_| err),
        Err(err) => Err(err),
    }
}

fn decode_still_inner(path: &Path, format: ImageFormat) -> Result<Master, String> {
    let file = File::open(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    let reader = image::ImageReader::with_format(BufReader::new(file), format);
    let mut decoder = reader
        .into_decoder()
        .map_err(|err| format!("formato não suportado: {err}"))?;
    decoder
        .set_limits(limits_of())
        .map_err(|err| format!("imagem grande demais: {err}"))?;
    let (bit_depth, color) = describe_color(decoder.color_type());
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|err| format!("não decodificou: {err}"))?;
    image.apply_orientation(orientation);
    check_dims(image.width(), image.height())?;
    finish_still(image, bit_depth, color, format)
}

fn finish_still(
    image: DynamicImage,
    bit_depth: u8,
    color: String,
    format: ImageFormat,
) -> Result<Master, String> {
    let kind = match format {
        ImageFormat::Jpeg => Kind::Jpeg,
        ImageFormat::Bmp => Kind::Bmp,
        ImageFormat::Tiff => Kind::Tiff,
        ImageFormat::Ico => Kind::Ico,
        ImageFormat::Tga => Kind::Tga,
        ImageFormat::Qoi => Kind::Qoi,
        ImageFormat::Pnm => Kind::Pnm,
        ImageFormat::Hdr => Kind::Hdr,
        ImageFormat::OpenExr => Kind::Exr,
        ImageFormat::WebP => Kind::Webp,
        _ => Kind::Png,
    };
    Ok(Master::Still {
        image,
        bit_depth,
        color,
        mipmaps: 1,
        notes: Vec::new(),
        id: id_of(kind),
    })
}

fn decode_png(path: &Path) -> Result<Master, String> {
    let file = File::open(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    let decoder = PngDecoder::new(BufReader::new(file)).map_err(|err| format!("PNG: {err}"))?;
    if decoder.is_apng().unwrap_or(false) {
        let (width, height) = decoder.dimensions();
        check_dims(width, height)?;
        let anim = decoder.apng().map_err(|err| format!("APNG: {err}"))?;
        return frames_from(anim, width, height, "png", "RGBA", true);
    }
    decode_still(path, ImageFormat::Png)
}

fn decode_webp(path: &Path) -> Result<Master, String> {
    let file = File::open(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    let decoder = WebPDecoder::new(BufReader::new(file)).map_err(|err| format!("WebP: {err}"))?;
    if decoder.has_animation() {
        let (width, height) = decoder.dimensions();
        check_dims(width, height)?;
        return frames_from(decoder, width, height, "webp", "RGBA", true);
    }
    decode_still(path, ImageFormat::WebP)
}

fn decode_gif(path: &Path) -> Result<Master, String> {
    let file = File::open(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    let decoder = GifDecoder::new(BufReader::new(file)).map_err(|err| format!("GIF: {err}"))?;
    let (width, height) = decoder.dimensions();
    check_dims(width, height)?;
    frames_from(decoder, width, height, "gif", "RGBA", true)
}

fn frames_from<'a, D>(
    decoder: D,
    width: u32,
    height: u32,
    id: &'static str,
    color: &str,
    loop_forever: bool,
) -> Result<Master, String>
where
    D: AnimationDecoder<'a>,
{
    let mut canvas = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 0]));
    let mut frames = Vec::new();
    let mut notes = Vec::new();
    for frame in decoder.into_frames() {
        if frames.len() == MAX_FRAMES {
            notes.push(format!("animação cortada em {MAX_FRAMES} quadros"));
            break;
        }
        let frame = frame.map_err(|err| format!("quadro da animação: {err}"))?;
        let delay = delay_ms(frame.delay());
        blit(&mut canvas, frame.buffer(), frame.left(), frame.top());
        let (width, height) = canvas.dimensions();
        frames.push(Frame {
            rgba: canvas.as_raw().clone(),
            width,
            height,
            delay_ms: delay,
        });
        let bytes = frames.len() as u64 * width as u64 * height as u64 * 4;
        if bytes > MAX_ALLOC {
            notes.push("animação interrompida pelo limite de memória".to_owned());
            break;
        }
    }
    if frames.is_empty() {
        return Err("animação sem quadros".to_owned());
    }
    if frames.len() > 1 {
        notes.push(
            "descarte de quadro é aproximado: cada quadro é desenhado sobre o anterior".to_owned(),
        );
    }
    Ok(Master::Frames(crate::model::Built {
        frames,
        bit_depth: 8,
        color: color.to_owned(),
        mipmaps: 1,
        notes,
        id,
        loop_forever,
    }))
}

fn delay_ms(delay: image::Delay) -> u32 {
    let (numer, denom) = delay.numer_denom_ms();
    if denom == 0 {
        10
    } else {
        numer.saturating_div(denom).min(60_000)
    }
}

fn blit(canvas: &mut image::RgbaImage, sprite: &image::RgbaImage, left: u32, top: u32) {
    for y in 0..sprite.height() {
        for x in 0..sprite.width() {
            let dx = left as i64 + x as i64;
            let dy = top as i64 + y as i64;
            if dx < 0 || dy < 0 || dx >= canvas.width() as i64 || dy >= canvas.height() as i64 {
                continue;
            }
            let src = sprite.get_pixel(x, y);
            if src[3] == 0 {
                continue;
            }
            if src[3] == 255 {
                canvas.put_pixel(dx as u32, dy as u32, *src);
                continue;
            }
            let dst = canvas.get_pixel(dx as u32, dy as u32);
            let alpha = u32::from(src[3]);
            let inv = 255 - alpha;
            let mix = |s: u8, d: u8| -> u8 {
                ((u32::from(s) * alpha + u32::from(d) * inv) / 255) as u8
            };
            canvas.put_pixel(
                dx as u32,
                dy as u32,
                image::Rgba([
                    mix(src[0], dst[0]),
                    mix(src[1], dst[1]),
                    mix(src[2], dst[2]),
                    src[3].max(dst[3]),
                ]),
            );
        }
    }
}

pub fn read_memory_image(bytes: &[u8], format: ImageFormat) -> Result<DynamicImage, String> {
    let reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut decoder = reader
        .into_decoder()
        .map_err(|err| format!("não decodificou: {err}"))?;
    decoder
        .set_limits(limits_of())
        .map_err(|err| format!("imagem grande demais: {err}"))?;
    DynamicImage::from_decoder(decoder).map_err(|err| format!("não decodificou: {err}"))
}
