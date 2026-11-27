//! Decodificador de texturas compactadas ASTC (Adaptive Scalable Texture Compression).
//! Suporta o contêiner padrão .astc de 16 bytes via texture2ddecoder.

use crate::model::{Built, Frame, bgra_to_rgba, check_dims};

const ASTC_MAGIC: &[u8] = &[0x13, 0xAB, 0xA1, 0x5C];

pub fn decode_astc_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 16 || !data.starts_with(ASTC_MAGIC) {
        return Err("Arquivo ASTC inválido ou cabeçalho truncado".to_owned());
    }

    let block_w = data[4] as usize;
    let block_h = data[5] as usize;
    let block_d = data[6] as usize;

    if !(4..=12).contains(&block_w) || !(4..=12).contains(&block_h) || block_d != 1 {
        return Err(format!(
            "Dimensões de bloco ASTC não suportadas: {block_w}×{block_h}×{block_d}"
        ));
    }

    let width = (data[7] as u32 | ((data[8] as u32) << 8) | ((data[9] as u32) << 16)) as usize;
    let height = (data[10] as u32 | ((data[11] as u32) << 8) | ((data[12] as u32) << 16)) as usize;
    let depth = (data[13] as u32 | ((data[14] as u32) << 8) | ((data[15] as u32) << 16)) as usize;

    if depth != 1 {
        return Err("Textura volumétrica ASTC (3D) não é suportada".to_owned());
    }

    check_dims(width as u32, height as u32)?;

    let blocks_x = width.div_ceil(block_w);
    let blocks_y = height.div_ceil(block_h);
    let expected_bytes = blocks_x * blocks_y * 16;

    let payload = data
        .get(16..16 + expected_bytes)
        .ok_or_else(|| "Arquivo ASTC truncado nos blocos de textura".to_owned())?;

    let mut pixels = vec![0u32; width * height];
    texture2ddecoder::decode_astc(payload, width, height, block_w, block_h, &mut pixels)
        .map_err(|err| format!("Falha ao decodificar blocos ASTC: {err}"))?;

    let rgba = bgra_to_rgba(&pixels);

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width: width as u32,
            height: height as u32,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA (ASTC)".to_owned(),
        mipmaps: 1,
        notes: vec![format!("ASTC bloco {block_w}×{block_h} ({width}×{height})")],
        id: "astc",
        loop_forever: false,
    })
}
