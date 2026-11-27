//! Decodificador de bitmaps X11 (.xbm).
//!
//! XBM é um formato clássico da era X11 codificado em texto ASCII como sintaxe C.
//! Cada byte armazena 8 pixels monocromáticos (LSB primeiro).

use std::path::Path;

use crate::model::{Built, Frame, Master, check_dims};

pub fn decode_xbm_bytes(bytes: &[u8]) -> Result<Master, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("#define") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[1].to_ascii_lowercase();
                if name.ends_with("_width") || name.ends_with("width") {
                    if let Ok(w) = parts[2].parse::<u32>() {
                        width = Some(w);
                    }
                } else if (name.ends_with("_height") || name.ends_with("height"))
                    && let Ok(h) = parts[2].parse::<u32>() {
                        height = Some(h);
                    }
            }
        }
    }

    let width = width.ok_or_else(|| "XBM sem definição de largura".to_owned())?;
    let height = height.ok_or_else(|| "XBM sem definição de altura".to_owned())?;
    check_dims(width, height)?;

    // Localizar a chave de abertura do array de bytes `{`
    let brace_pos = text
        .find('{')
        .ok_or_else(|| "XBM sem bloco de dados '{'".to_owned())?;
    let data_str = &text[brace_pos + 1..];
    let end_brace = data_str.find('}').unwrap_or(data_str.len());
    let hex_part = &data_str[..end_brace];

    let mut raw_bytes = Vec::new();
    for token in hex_part.split(|c: char| c == ',' || c.is_whitespace() || c == ';') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let clean = token
            .strip_prefix("0x")
            .or_else(|| token.strip_prefix("0X"))
            .unwrap_or(token);
        if let Ok(byte_val) = u8::from_str_radix(clean, 16) {
            raw_bytes.push(byte_val);
        }
    }

    let bytes_per_row = (width as usize).div_ceil(8);
    let expected = bytes_per_row * (height as usize);
    if raw_bytes.len() < expected {
        return Err(format!(
            "XBM com dados incompletos: esperados {} bytes, encontrados {}",
            expected,
            raw_bytes.len()
        ));
    }

    let mut rgba = vec![0u8; (width as usize) * (height as usize) * 4];

    for y in 0..height as usize {
        let row_offset = y * bytes_per_row;
        for x in 0..width as usize {
            let byte_idx = row_offset + (x / 8);
            let bit_idx = x % 8;
            let bit = (raw_bytes[byte_idx] >> bit_idx) & 1;

            let dst_idx = (y * (width as usize) + x) * 4;
            // No padrão XBM, bit 1 é preto (tinta), bit 0 é branco
            let val = if bit == 1 { 0u8 } else { 255u8 };
            rgba[dst_idx] = val;
            rgba[dst_idx + 1] = val;
            rgba[dst_idx + 2] = val;
            rgba[dst_idx + 3] = 255;
        }
    }

    let note = format!(
        "X11 Bitmap (.xbm): monocromático 1-bit ({}×{})",
        width, height
    );

    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 1,
        color: "Monocromático (1-bit XBM)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "xbm",
        loop_forever: false,
    }))
}

#[allow(dead_code)]
pub fn decode_xbm_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo XBM: {err}"))?;
    decode_xbm_bytes(&bytes)
}
