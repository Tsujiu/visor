//! Decodificador de arquivos Adobe Photoshop PSD e PSB (Large Document).
//!
//! Utiliza a biblioteca pure-Rust `psd` para extrair a composição achatada
//! oficial (flattened composite) em RGBA8, dimensões, canais e metadados de camadas.

use std::path::Path;

use crate::model::{Built, Frame, Master, check_dims};

pub fn decode_psd(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 26 {
        return Err("PSD inválido ou truncado".to_owned());
    }
    if !bytes.starts_with(b"8BPS") {
        return Err("magic 8BPS do Photoshop não encontrado".to_owned());
    }

    let psd_img = psd::Psd::from_bytes(bytes).map_err(|err| format!("PSD: {err}"))?;

    let width = psd_img.width();
    let height = psd_img.height();
    check_dims(width, height)?;

    let rgba = psd_img.rgba();
    if rgba.len() != (width as usize * height as usize * 4) {
        return Err("buffer de pixels do PSD com tamanho inconsistente".to_owned());
    }

    let layer_count = psd_img.layers().len();
    let depth = psd_img.depth();
    let depth_val = match depth {
        psd::PsdDepth::Eight => 8,
        psd::PsdDepth::Sixteen => 16,
        psd::PsdDepth::ThirtyTwo => 32,
        psd::PsdDepth::One => 1,
    };

    let color_mode = format!("{:?}", psd_img.color_mode());
    let note = format!(
        "Photoshop PSD/PSB: composição achatada ({} camadas, modo {}, {} bits/canal)",
        layer_count, color_mode, depth_val
    );

    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: depth_val,
        color: format!("RGBA ({color_mode})"),
        mipmaps: 1,
        notes: vec![note],
        id: "psd",
        loop_forever: false,
    }))
}

#[allow(dead_code)]
pub fn decode_psd_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo PSD: {err}"))?;
    decode_psd(&bytes)
}
