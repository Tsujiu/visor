//! Exportação relê o arquivo. PNG e TIFF de origem em 16 bits continuam em 16 bits.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::codecs::gif::{GifEncoder, Repeat};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::{Delay, DynamicImage, ExtendedColorType, Frame, ImageFormat};

use crate::decode::open_master;
use crate::model::Master;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportKind {
    Png,
    Jpeg,
    Webp,
    Bmp,
    Tiff,
    Gif,
    Qoi,
    Tga,
    Ico,
    Pnm,
    Hdr,
}

impl ExportKind {
    pub fn all() -> &'static [ExportKind] {
        &[
            Self::Png,
            Self::Jpeg,
            Self::Webp,
            Self::Bmp,
            Self::Tiff,
            Self::Gif,
            Self::Qoi,
            Self::Tga,
            Self::Ico,
            Self::Pnm,
            Self::Hdr,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
            Self::Bmp => "BMP",
            Self::Tiff => "TIFF",
            Self::Gif => "GIF",
            Self::Qoi => "QOI",
            Self::Tga => "TGA",
            Self::Ico => "ICO",
            Self::Pnm => "PPM",
            Self::Hdr => "HDR",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Gif => "gif",
            Self::Qoi => "qoi",
            Self::Tga => "tga",
            Self::Ico => "ico",
            Self::Pnm => "ppm",
            Self::Hdr => "hdr",
        }
    }
}

pub fn export_image(src: &Path, dst: &Path, kind: ExportKind) -> Result<(), String> {
    let master = open_master(src)?;
    match master {
        Master::Still { image, .. } => save_dynamic(&image, dst, kind),
        Master::Frames(built) => {
            if kind == ExportKind::Gif && built.frames.len() > 1 {
                save_gif(&built.frames, dst)
            } else {
                let frame = built.frames.first().ok_or_else(|| "imagem vazia".to_owned())?;
                let image = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba.clone())
                    .ok_or_else(|| "quadro inválido".to_owned())?;
                save_dynamic(&DynamicImage::ImageRgba8(image), dst, kind)
            }
        }
    }
}

pub fn export_many(files: &[PathBuf], dest_dir: &Path, kind: ExportKind) -> Vec<(PathBuf, Result<(), String>)> {
    files
        .iter()
        .map(|path| {
            let name = path
                .file_stem()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "imagem".to_owned());
            let dst = dest_dir.join(format!("{name}.{}", kind.extension()));
            let result = export_image(path, &dst, kind);
            (dst, result)
        })
        .collect()
}

fn save_dynamic(image: &DynamicImage, dst: &Path, kind: ExportKind) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| format!("não criou a pasta: {err}"))?;
        }
    }
    let file = File::create(dst).map_err(|err| format!("não criou o arquivo: {err}"))?;
    let mut writer = BufWriter::new(file);
    match kind {
        ExportKind::Jpeg => {
            let rgb = image.to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, 95);
            encoder
                .encode_image(&DynamicImage::ImageRgb8(rgb))
                .map_err(|err| format!("JPEG: {err}"))?;
        }
        ExportKind::Webp => {
            let rgba = image.to_rgba8();
            let encoder = WebPEncoder::new_lossless(&mut writer);
            encoder
                .encode(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|err| format!("WebP: {err}"))?;
        }
        ExportKind::Hdr => {
            let rgb = image.to_rgb32f();
            DynamicImage::ImageRgb32F(rgb)
                .write_to(&mut writer, ImageFormat::Hdr)
                .map_err(|err| format!("HDR: {err}"))?;
        }
        ExportKind::Ico => {
            if image.width() > 256 || image.height() > 256 {
                return Err("ICO só aceita até 256 px".to_owned());
            }
            image
                .write_to(&mut writer, ImageFormat::Ico)
                .map_err(|err| format!("ICO: {err}"))?;
        }
        ExportKind::Png | ExportKind::Tiff => {
            let format = if kind == ExportKind::Png {
                ImageFormat::Png
            } else {
                ImageFormat::Tiff
            };
            image
                .write_to(&mut writer, format)
                .map_err(|err| format!("{err}"))?;
        }
        ExportKind::Bmp => write_8(image, &mut writer, ImageFormat::Bmp)?,
        ExportKind::Gif => write_8(image, &mut writer, ImageFormat::Gif)?,
        ExportKind::Qoi => write_8(image, &mut writer, ImageFormat::Qoi)?,
        ExportKind::Tga => write_8(image, &mut writer, ImageFormat::Tga)?,
        ExportKind::Pnm => {
            image
                .to_rgb8()
                .write_to(&mut writer, ImageFormat::Pnm)
                .or_else(|_| {
                    DynamicImage::ImageRgb8(image.to_rgb8()).write_to(&mut writer, ImageFormat::Pnm)
                })
                .map_err(|err| format!("PNM: {err}"))?;
        }
    }
    Ok(())
}

fn write_8(image: &DynamicImage, writer: &mut BufWriter<File>, format: ImageFormat) -> Result<(), String> {
    DynamicImage::ImageRgba8(image.to_rgba8())
        .write_to(writer, format)
        .map_err(|err| format!("{err}"))
}

fn save_gif(frames: &[crate::model::Frame], dst: &Path) -> Result<(), String> {
    let file = File::create(dst).map_err(|err| format!("não criou o arquivo: {err}"))?;
    let mut encoder = GifEncoder::new(BufWriter::new(file));
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|err| format!("GIF: {err}"))?;
    for frame in frames {
        let buffer = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba.clone())
            .ok_or_else(|| "quadro inválido".to_owned())?;
        let delay = frame.delay_ms.max(1);
        encoder
            .encode_frame(Frame::from_parts(
                buffer,
                0,
                0,
                Delay::from_numer_denom_ms(delay, 1),
            ))
            .map_err(|err| format!("GIF: {err}"))?;
    }
    Ok(())
}
