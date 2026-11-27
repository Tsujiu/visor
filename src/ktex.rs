//! Klei Entertainment KTEX texture decoder (.tex).
//! Usado em jogos como Don't Starve, Oxygen Not Included, Mark of the Ninja.

use crate::binio::{u16_le, u32_le};
use crate::model::{Built, Frame, bgra_to_rgba, check_dims};

pub fn decode_ktex_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 18 {
        return Err("arquivo KTEX muito curto".to_owned());
    }
    if !data.starts_with(b"KTEX") {
        return Err("magic inválido para KTEX (esperado KTEX)".to_owned());
    }

    let specs = u32_le(data, 4).map_err(|e| e.to_string())?;
    let _platform = specs & 0x07;
    let pixel_format = (specs >> 3) & 0x07;
    let _texture_type = (specs >> 6) & 0x07;
    let mipmap_count = ((specs >> 9) & 0x0F).max(1) as usize;

    let meta_offset = 8;
    if data.len() < meta_offset + 10 {
        return Err("metadados de mipmap truncados no KTEX".to_owned());
    }

    let width = u16_le(data, meta_offset).map_err(|e| e.to_string())? as u32;
    let height = u16_le(data, meta_offset + 2).map_err(|e| e.to_string())? as u32;
    let _pitch = u16_le(data, meta_offset + 4).map_err(|e| e.to_string())?;
    let data_size = u32_le(data, meta_offset + 6).map_err(|e| e.to_string())? as usize;

    check_dims(width, height)?;

    let data_start = meta_offset + mipmap_count * 10;
    if data_start > data.len() {
        return Err("tabela de mipmaps excede o tamanho do arquivo KTEX".to_owned());
    }

    let src = if data_start + data_size <= data.len() {
        &data[data_start..data_start + data_size]
    } else {
        &data[data_start..]
    };

    let w = width as usize;
    let h = height as usize;

    let (rgba, format_name) = match pixel_format {
        0 => {
            // DXT1 / BC1
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc1(src, w, h, &mut pixels)
                .map_err(|e| format!("KTEX DXT1: {e}"))?;
            (bgra_to_rgba(&pixels), "DXT1")
        }
        1 => {
            // DXT3 / BC2
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc2(src, w, h, &mut pixels)
                .map_err(|e| format!("KTEX DXT3: {e}"))?;
            (bgra_to_rgba(&pixels), "DXT3")
        }
        2 => {
            // DXT5 / BC3
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc3(src, w, h, &mut pixels)
                .map_err(|e| format!("KTEX DXT5: {e}"))?;
            (bgra_to_rgba(&pixels), "DXT5")
        }
        3 => {
            // RGB888
            let need = w * h * 3;
            let slice = src
                .get(..need)
                .ok_or_else(|| "KTEX RGB truncado".to_owned())?;
            let mut out = Vec::with_capacity(w * h * 4);
            for chunk in slice.as_chunks::<3>().0 {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(chunk[2]);
                out.push(255);
            }
            (out, "RGB888")
        }
        4 => {
            // RGBA8888
            let need = w * h * 4;
            let slice = src
                .get(..need)
                .ok_or_else(|| "KTEX RGBA truncado".to_owned())?;
            (slice.to_vec(), "RGBA8888")
        }
        5 => {
            // ARGB8888
            let need = w * h * 4;
            let slice = src
                .get(..need)
                .ok_or_else(|| "KTEX ARGB truncado".to_owned())?;
            let mut out = Vec::with_capacity(w * h * 4);
            for chunk in slice.as_chunks::<4>().0 {
                out.push(chunk[1]); // R
                out.push(chunk[2]); // G
                out.push(chunk[3]); // B
                out.push(chunk[0]); // A
            }
            (out, "ARGB8888")
        }
        _ => {
            // Tenta DXT5 por padrão caso formato desconhecido
            let mut pixels = vec![0u32; w * h];
            if texture2ddecoder::decode_bc3(src, w, h, &mut pixels).is_ok() {
                (bgra_to_rgba(&pixels), "DXT5 (fallback)")
            } else {
                return Err(format!("formato KTEX não suportado: {pixel_format}"));
            }
        }
    };

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: mipmap_count as u32,
        notes: vec![format!(
            "Klei KTEX {format_name}, {width}x{height}, {mipmap_count} mipmaps"
        )],
        id: "tex",
        loop_forever: false,
    })
}
