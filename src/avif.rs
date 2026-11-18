//! Decodificador AVIF (AV1 Image File Format) e AVIS (animado) em Rust puro via oxideav-avif.

use crate::model::{Built, Frame, check_dims};

pub fn decode_avif_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 12 {
        return Err("AVIF truncado".to_owned());
    }

    // Tenta decodificar sequência de quadros (se for animação AVIS ou burst)
    if let Ok(seq) = oxideav_avif::decode_all(data) {
        if seq.len() > 1 {
            let mut frames = Vec::with_capacity(seq.len());
            for f in seq {
                let w = f.image.width;
                let h = f.image.height;
                check_dims(w, h)?;
                let rgba = f
                    .image
                    .try_to_rgba8()
                    .map_err(|err| format!("AVIS: {err}"))?;
                let delay_ms = f
                    .delay
                    .map(|d| (d.as_secs_f64() * 1000.0).round().max(10.0) as u32)
                    .unwrap_or(100);
                frames.push(Frame {
                    rgba,
                    width: w,
                    height: h,
                    delay_ms,
                });
            }
            if !frames.is_empty() {
                let w = frames[0].width;
                let h = frames[0].height;
                return Ok(Built {
                    frames,
                    bit_depth: 8,
                    color: "RGBA".to_owned(),
                    mipmaps: 1,
                    notes: vec![format!("AVIS animado ({w}×{h})")],
                    id: "avif",
                    loop_forever: true,
                });
            }
        }
    }

    // Imagem estática AVIF
    let rgba_img =
        oxideav_avif::decode_rgba8(data).map_err(|err| format!("AVIF recusado: {err}"))?;
    let width = rgba_img.width;
    let height = rgba_img.height;
    check_dims(width, height)?;

    Ok(Built {
        frames: vec![Frame {
            rgba: rgba_img.data,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: 1,
        notes: vec![format!("AVIF estático ({width}×{height})")],
        id: "avif",
        loop_forever: false,
    })
}
