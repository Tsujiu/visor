//! GoldSrc / Half-Life IDSP sprite decoder (.spr).

use crate::binio::{u16_le, u32_le};
use crate::model::{Built, Frame};

pub fn decode_spr_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 42 {
        return Err("arquivo SPR muito curto".to_owned());
    }
    if !data.starts_with(b"IDSP") {
        return Err("magic inválido para GoldSrc SPR (esperado IDSP)".to_owned());
    }

    let version = u32_le(data, 4).map_err(|e| e.to_string())?;
    if version != 1 && version != 2 {
        return Err(format!("versão SPR não suportada: {version}"));
    }

    let spr_type = u32_le(data, 8).map_err(|e| e.to_string())?;
    let tex_format = u32_le(data, 12).map_err(|e| e.to_string())?;
    let max_w = u32_le(data, 20).map_err(|e| e.to_string())?;
    let max_h = u32_le(data, 24).map_err(|e| e.to_string())?;
    let num_frames = u32_le(data, 28).map_err(|e| e.to_string())? as usize;

    let palette_size = u16_le(data, 40).map_err(|e| e.to_string())? as usize;
    let palette_bytes_len = palette_size * 3;
    let palette_offset = 42;
    if data.len() < palette_offset + palette_bytes_len {
        return Err("arquivo SPR truncado na paleta".to_owned());
    }

    let palette = &data[palette_offset..palette_offset + palette_bytes_len];
    let mut offset = palette_offset + palette_bytes_len;

    let mut frames = Vec::new();

    for _ in 0..num_frames {
        if offset + 4 > data.len() {
            break;
        }
        let frame_type = u32_le(data, offset).map_err(|e| e.to_string())?;
        offset += 4;

        if frame_type == 0 {
            // Single frame
            if offset + 16 > data.len() {
                break;
            }
            let _origin_x = u32_le(data, offset).map_err(|e| e.to_string())?;
            let _origin_y = u32_le(data, offset + 4).map_err(|e| e.to_string())?;
            let width = u32_le(data, offset + 8).map_err(|e| e.to_string())?;
            let height = u32_le(data, offset + 12).map_err(|e| e.to_string())?;
            offset += 16;

            let pixel_count = (width as usize) * (height as usize);
            if offset + pixel_count > data.len() {
                break;
            }
            let pixel_data = &data[offset..offset + pixel_count];
            offset += pixel_count;

            let rgba = decode_palette_indices(pixel_data, palette, palette_size, tex_format);
            frames.push(Frame {
                rgba,
                width,
                height,
                delay_ms: 100,
            });
        } else if frame_type == 1 {
            // Group frame (animated sequence)
            if offset + 4 > data.len() {
                break;
            }
            let subframe_count = u32_le(data, offset).map_err(|e| e.to_string())? as usize;
            offset += 4;

            // Interval times (f32 each)
            let intervals_len = subframe_count * 4;
            if offset + intervals_len > data.len() {
                break;
            }
            let intervals_data = &data[offset..offset + intervals_len];
            offset += intervals_len;

            for i in 0..subframe_count {
                if offset + 16 > data.len() {
                    break;
                }
                let _origin_x = u32_le(data, offset).map_err(|e| e.to_string())?;
                let _origin_y = u32_le(data, offset + 4).map_err(|e| e.to_string())?;
                let width = u32_le(data, offset + 8).map_err(|e| e.to_string())?;
                let height = u32_le(data, offset + 12).map_err(|e| e.to_string())?;
                offset += 16;

                let pixel_count = (width as usize) * (height as usize);
                if offset + pixel_count > data.len() {
                    break;
                }
                let pixel_data = &data[offset..offset + pixel_count];
                offset += pixel_count;

                let delay_f32 = if (i + 1) * 4 <= intervals_data.len() {
                    let b: [u8; 4] = intervals_data[i * 4..(i + 1) * 4].try_into().unwrap();
                    f32::from_le_bytes(b)
                } else {
                    0.1f32
                };
                let delay_ms = ((delay_f32.max(0.01) * 1000.0) as u32).clamp(10, 5000);

                let rgba = decode_palette_indices(pixel_data, palette, palette_size, tex_format);
                frames.push(Frame {
                    rgba,
                    width,
                    height,
                    delay_ms,
                });
            }
        }
    }

    if frames.is_empty() {
        return Err("nenhum frame válido encontrado no arquivo SPR".to_owned());
    }

    let format_name = match tex_format {
        0 => "SPR_NORMAL",
        1 => "SPR_ADDITIVE",
        2 => "SPR_INDEXALPHA",
        3 => "SPR_ALPHTEST",
        _ => "Desconhecido",
    };

    let count = frames.len();
    Ok(Built {
        frames,
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: 0,
        notes: vec![format!(
            "GoldSrc SPR v{version}, tipo={spr_type}, formato={format_name}, {count} quadro(s), max={max_w}x{max_h}"
        )],
        id: "spr",
        loop_forever: count > 1,
    })
}

fn decode_palette_indices(
    indices: &[u8],
    palette: &[u8],
    palette_size: usize,
    tex_format: u32,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(indices.len() * 4);
    for &idx in indices {
        let idx = (idx as usize).min(palette_size.saturating_sub(1));
        let r = palette[idx * 3];
        let g = palette[idx * 3 + 1];
        let b = palette[idx * 3 + 2];

        match tex_format {
            0 => {
                // SPR_NORMAL: standard opaque color
                out.extend_from_slice(&[r, g, b, 255]);
            }
            1 => {
                // SPR_ADDITIVE: black is transparent
                let a = if r == 0 && g == 0 && b == 0 { 0 } else { 255 };
                out.extend_from_slice(&[r, g, b, a]);
            }
            2 => {
                // SPR_INDEXALPHA: palette index is the alpha channel
                out.extend_from_slice(&[255, 255, 255, idx as u8]);
            }
            3 => {
                // SPR_ALPHTEST: index 255 is transparent
                let a = if idx == 255 { 0 } else { 255 };
                out.extend_from_slice(&[r, g, b, a]);
            }
            _ => {
                out.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }
    out
}
