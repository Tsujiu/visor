//! Decodificador de cursores animados do Windows (.ani).
//!
//! O formato ANI é um contêiner RIFF com FourCC `ACON`. Ele encapsula múltiplos
//! quadros no formato ICO/CUR juntamente com taxas de exibição (jiffies = 1/60s).

use std::path::Path;

use crate::binio::u32_le;
use crate::model::{Built, Master, check_dims};

pub fn decode_ani_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 36 {
        return Err("ANI inválido ou truncado".to_owned());
    }
    if !bytes.starts_with(b"RIFF") || bytes.get(8..12) != Some(b"ACON") {
        return Err("ANI inválido: cabeçalho RIFF/ACON não encontrado".to_owned());
    }

    let mut pos = 12;
    let mut default_jif_rate = 10u32; // padrão 10 jiffies (~166ms)
    let mut rates = Vec::new();
    let mut raw_frames = Vec::new();

    while pos + 8 <= bytes.len() {
        let chunk_id = &bytes[pos..pos + 4];
        let chunk_len = u32_le(bytes, pos + 4)? as usize;
        let chunk_data_start = pos + 8;
        let chunk_data_end = (chunk_data_start + chunk_len).min(bytes.len());
        let chunk_data = &bytes[chunk_data_start..chunk_data_end];

        // Chunks no RIFF têm padding para 2 bytes
        let next_pos = pos + 8 + chunk_len + (chunk_len % 2);

        if chunk_id == b"anih" && chunk_data.len() >= 36 {
            // cbSize: u32, nFrames: u32, nSteps: u32, cx: u32, cy: u32, cBitCount: u32, cPlanes: u32, jifRate: u32
            if let Ok(rate) = u32_le(chunk_data, 28)
                && rate > 0 {
                    default_jif_rate = rate;
                }
        } else if chunk_id == b"rate" {
            // Tabela de taxas individuais para cada frame
            let count = chunk_data.len() / 4;
            for i in 0..count {
                if let Ok(rate) = u32_le(chunk_data, i * 4) {
                    rates.push(rate);
                }
            }
        } else if chunk_id == b"LIST" && chunk_data.len() >= 4 {
            let list_type = &chunk_data[0..4];
            if list_type == b"fram" {
                // Percorrer os chunks internos de "fram"
                let mut fram_pos = 4;
                while fram_pos + 8 <= chunk_data.len() {
                    let sub_id = &chunk_data[fram_pos..fram_pos + 4];
                    let sub_len = u32_le(chunk_data, fram_pos + 4)? as usize;
                    let sub_start = fram_pos + 8;
                    let sub_end = (sub_start + sub_len).min(chunk_data.len());
                    let sub_data = &chunk_data[sub_start..sub_end];

                    let next_fram_pos = fram_pos + 8 + sub_len + (sub_len % 2);

                    if sub_id == b"icon" {
                        raw_frames.push(sub_data);
                    }
                    fram_pos = next_fram_pos;
                }
            }
        }

        pos = next_pos;
    }

    if raw_frames.is_empty() {
        return Err("ANI sem quadros icon encontrados".to_owned());
    }

    let mut frames = Vec::new();
    let mut canvas_w = 0u32;
    let mut canvas_h = 0u32;

    for (index, icon_data) in raw_frames.iter().enumerate() {
        // Cada quadro pode ser decodificado como ICO ou CUR
        let decoded_built = crate::simple::decode_ico(icon_data)
            .or_else(|_| crate::simple::decode_cur(icon_data))
            .map_err(|err| format!("quadro {} do cursor animado: {err}", index + 1))?;

        if let Some(mut frame) = decoded_built.frames.into_iter().next() {
            if canvas_w == 0 {
                canvas_w = frame.width;
                canvas_h = frame.height;
                check_dims(canvas_w, canvas_h)?;
            }

            // Calcular duração do quadro: 1 jiffy = 1000ms / 60 = 16.666 ms
            let jif = rates.get(index).copied().unwrap_or(default_jif_rate);
            let delay_ms = ((jif as u64 * 1000) / 60).clamp(16, 5000) as u32;
            frame.delay_ms = delay_ms;

            frames.push(frame);
        }
    }

    if frames.is_empty() {
        return Err("nenhum quadro válido no cursor animado".to_owned());
    }

    let note = format!(
        "Cursor animado Windows (.ani): {} quadros ({}×{})",
        frames.len(),
        canvas_w,
        canvas_h
    );

    Ok(Master::Frames(Built {
        frames,
        bit_depth: 8,
        color: "RGBA (Animated Cursor)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "ani",
        loop_forever: true,
    }))
}

#[allow(dead_code)]
pub fn decode_ani_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo ANI: {err}"))?;
    decode_ani_bytes(&bytes)
}
