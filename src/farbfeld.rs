//! Decodificador de imagens Farbfeld (.ff, .farbfeld).
//!
//! Farbfeld é um formato de imagem sem perdas de 16 bits por canal,
//! criado pelo projeto suckless.org. O formato consiste em 8 bytes magic `farbfeld`,
//! largura e altura em inteiros big-endian de 32 bits, seguidos por pixels RGBA de 16 bits BE.

use std::path::Path;

use crate::binio::u32_be;
use crate::model::{Built, Frame, Master, check_dims};

pub fn decode_farbfeld_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 16 {
        return Err("Farbfeld inválido ou truncado".to_owned());
    }
    if !bytes.starts_with(b"farbfeld") {
        return Err("Farbfeld: magic 'farbfeld' ausente".to_owned());
    }

    let width = u32_be(bytes, 8)?;
    let height = u32_be(bytes, 12)?;
    check_dims(width, height)?;

    let total_pixels = (width as usize) * (height as usize);
    let expected_bytes = 16 + total_pixels * 8; // 8 bytes por pixel (16 bits x 4 canais)
    if bytes.len() < expected_bytes {
        return Err(format!(
            "Farbfeld truncado: esperados {} bytes, encontrados {}",
            expected_bytes,
            bytes.len()
        ));
    }

    let mut rgba = vec![0u8; total_pixels * 4];
    let pixel_data = &bytes[16..expected_bytes];

    for (i, chunk) in pixel_data.chunks_exact(8).enumerate() {
        // Cada canal tem 2 bytes (Big Endian u16). Converte para u8 reduzindo os 8 bits menos significativos
        let r = chunk[0]; // mais significativo
        let g = chunk[2];
        let b = chunk[4];
        let a = chunk[6];

        let dst = i * 4;
        rgba[dst] = r;
        rgba[dst + 1] = g;
        rgba[dst + 2] = b;
        rgba[dst + 3] = a;
    }

    let note = format!(
        "Farbfeld: 16 bits/canal RGBA sem perdas ({}×{})",
        width, height
    );

    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 16,
        color: "RGBA (16-bit Farbfeld)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "farbfeld",
        loop_forever: false,
    }))
}

#[allow(dead_code)]
pub fn decode_farbfeld_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo Farbfeld: {err}"))?;
    decode_farbfeld_bytes(&bytes)
}
