//! Prévia RAW. Não é um revelador: sem demosaic e sem teste com arquivo de câmera.

use std::path::Path;

use crate::model::{check_dims, Built, Frame, Master};

pub fn decode_raw(path: &Path) -> Result<Master, String> {
    let raw = rawloader::decode_file(path).map_err(|err| format!("RAW: {err:?}"))?;
    let width = u32::try_from(raw.width).map_err(|_| "RAW com largura inválida".to_owned())?;
    let height = u32::try_from(raw.height).map_err(|_| "RAW com altura inválida".to_owned())?;
    check_dims(width, height)?;
    let pixels = width as usize * height as usize;
    let mut rgba = vec![0u8; pixels * 4];
    let color = match raw.data {
        rawloader::RawImageData::Integer(data) => {
            fill_integer(&mut rgba, &data, pixels, raw.cpp)?
        }
        rawloader::RawImageData::Float(data) => fill_float(&mut rgba, &data, pixels, raw.cpp)?,
    };
    let note = format!(
        "prévia RAW {} {} ({}×{}, {} canal). Sem demosaic nem balanço de câmera; sem amostra real neste ambiente.",
        raw.make, raw.model, width, height, raw.cpp
    );
    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 16,
        color,
        mipmaps: 1,
        notes: vec![note],
        id: "raw",
        loop_forever: false,
    }))
}

fn fill_integer(rgba: &mut [u8], data: &[u16], pixels: usize, cpp: usize) -> Result<String, String> {
    if data.len() < pixels.saturating_mul(cpp.max(1)) {
        return Err("RAW com buffer menor que a imagem".to_owned());
    }
    let shift = if data.iter().take(4096).any(|value| *value > 255) {
        8
    } else {
        0
    };
    match cpp {
        1 => {
            for (index, pixel) in rgba.chunks_exact_mut(4).enumerate() {
                let value = (data[index] >> shift) as u8;
                pixel.copy_from_slice(&[value, value, value, 255]);
            }
            Ok("prévia em cinza".to_owned())
        }
        3 | 4 => {
            for (index, pixel) in rgba.chunks_exact_mut(4).enumerate() {
                let base = index * cpp;
                pixel[0] = (data[base] >> shift) as u8;
                pixel[1] = (data[base + 1] >> shift) as u8;
                pixel[2] = (data[base + 2] >> shift) as u8;
                pixel[3] = if cpp == 4 {
                    (data[base + 3] >> shift) as u8
                } else {
                    255
                };
            }
            Ok(if cpp == 4 { "RGBA cru".to_owned() } else { "RGB cru".to_owned() })
        }
        other => Err(format!("RAW com {other} amostras por pixel não é lido")),
    }
}

fn fill_float(rgba: &mut [u8], data: &[f32], pixels: usize, cpp: usize) -> Result<String, String> {
    if cpp != 1 || data.len() < pixels {
        return Err("RAW em ponto flutuante não está no formato esperado".to_owned());
    }
    for (index, pixel) in rgba.chunks_exact_mut(4).enumerate() {
        let value = (data[index].clamp(0.0, 1.0) * 255.0) as u8;
        pixel.copy_from_slice(&[value, value, value, 255]);
    }
    Ok("prévia em cinza".to_owned())
}
