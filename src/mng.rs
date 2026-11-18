//! Decodificador MNG (Multiple-image Network Graphics).
//! Extrai fluxos PNG incorporados no contêiner com framing de chunks.

use crate::binio::u32_be;
use crate::model::{Built, Frame, check_dims};

const MNG_MAGIC: &[u8] = &[0x8A, b'M', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub fn decode_mng_bytes(data: &[u8]) -> Result<Built, String> {
    if !data.starts_with(MNG_MAGIC) {
        return Err("MNG inválido: assinatura mágica ausente".to_owned());
    }

    let mut cursor = 8;
    let mut nominal_w = 0u32;
    let mut nominal_h = 0u32;
    let mut ticks_per_sec = 1000u32;

    let mut frames = Vec::new();
    let mut current_png_chunks: Option<Vec<u8>> = None;

    while cursor + 12 <= data.len() {
        let length = u32_be(data, cursor)? as usize;
        let chunk_type = &data[cursor + 4..cursor + 8];
        let total_chunk_len = 12 + length;

        if cursor + total_chunk_len > data.len() {
            break;
        }

        let chunk_bytes = &data[cursor..cursor + total_chunk_len];

        match chunk_type {
            b"MHDR" => {
                if length >= 28 {
                    nominal_w = u32_be(data, cursor + 8)?;
                    nominal_h = u32_be(data, cursor + 12)?;
                    let ticks = u32_be(data, cursor + 16)?;
                    if ticks > 0 {
                        ticks_per_sec = ticks;
                    }
                }
            }
            b"IHDR" => {
                let mut stream = Vec::with_capacity(chunk_bytes.len() + 64);
                stream.extend_from_slice(PNG_MAGIC);
                stream.extend_from_slice(chunk_bytes);
                current_png_chunks = Some(stream);
            }
            b"PLTE" | b"tRNS" | b"IDAT" => {
                if let Some(ref mut stream) = current_png_chunks {
                    stream.extend_from_slice(chunk_bytes);
                }
            }
            b"IEND" => {
                if let Some(mut stream) = current_png_chunks.take() {
                    stream.extend_from_slice(chunk_bytes);
                    if let Ok(dyn_img) = image::load_from_memory(&stream) {
                        let w = dyn_img.width();
                        let h = dyn_img.height();
                        if check_dims(w, h).is_ok() {
                            let rgba = dyn_img.into_rgba8().into_raw();
                            let delay_ms = 1000 / ticks_per_sec.max(1);
                            frames.push(Frame {
                                rgba,
                                width: w,
                                height: h,
                                delay_ms: delay_ms.max(10),
                            });
                        }
                    }
                }
            }
            b"MEND" => {
                break;
            }
            _ => {
                // Outros chunks do MNG (DEFI, FRAM, etc.) ignorados para renderização
            }
        }

        cursor += total_chunk_len;
    }

    if frames.is_empty() {
        return Err("MNG sem quadros PNG decodificáveis".to_owned());
    }

    let w = if nominal_w > 0 {
        nominal_w
    } else {
        frames[0].width
    };
    let h = if nominal_h > 0 {
        nominal_h
    } else {
        frames[0].height
    };
    check_dims(w, h)?;

    let is_animated = frames.len() > 1;
    let count = frames.len();

    Ok(Built {
        frames,
        bit_depth: 8,
        color: "RGBA (MNG)".to_owned(),
        mipmaps: 1,
        notes: vec![format!(
            "MNG {} ({count} quadros)",
            if is_animated { "animado" } else { "estático" }
        )],
        id: "mng",
        loop_forever: is_animated,
    })
}
