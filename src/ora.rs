//! Decodificador de arquivos OpenRaster (.ora).
//!
//! O formato OpenRaster é uma especificação aberta para imagens raster multicamadas
//! (utilizado por Krita, GIMP e MyPaint). Conforme a especificação oficial ORA,
//! o contêiner ZIP armazena a composição final completa do documento em `mergedimage.png`.

use std::io::{Cursor, Read};
use std::path::Path;

use image::ImageFormat;

use crate::model::{Built, Frame, Master, check_dims};

pub fn decode_ora_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 30 || !bytes.starts_with(b"PK\x03\x04") {
        return Err("OpenRaster (.ora) inválido: cabeçalho ZIP ausente".to_owned());
    }

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|err| format!("ORA ZIP: {err}"))?;

    let mut names = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i)
            && let Ok(name) = file.name() {
                names.push(name.to_string());
            }
    }

    let target_name = if names.iter().any(|n| n == "mergedimage.png") {
        "mergedimage.png"
    } else if names.iter().any(|n| n == "Thumbnails/thumbnail.png") {
        "Thumbnails/thumbnail.png"
    } else {
        return Err("OpenRaster (.ora) sem mergedimage.png no arquivo".to_owned());
    };

    let mut merged_file = archive
        .by_name(target_name)
        .map_err(|err| format!("erro abrindo {target_name} do ORA: {err}"))?;

    let mut png_bytes = Vec::new();
    merged_file
        .read_to_end(&mut png_bytes)
        .map_err(|err| format!("erro lendo mergedimage.png do ORA: {err}"))?;

    let dyn_img = image::load_from_memory_with_format(&png_bytes, ImageFormat::Png)
        .map_err(|err| format!("não decodificou mergedimage.png do ORA: {err}"))?;

    let width = dyn_img.width();
    let height = dyn_img.height();
    check_dims(width, height)?;

    let rgba = dyn_img.to_rgba8().into_raw();
    let note = format!(
        "OpenRaster (.ora): composição achatada mergedimage.png ({}×{})",
        width, height
    );

    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA (OpenRaster)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "ora",
        loop_forever: false,
    }))
}

#[allow(dead_code)]
pub fn decode_ora_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo ORA: {err}"))?;
    decode_ora_bytes(&bytes)
}
