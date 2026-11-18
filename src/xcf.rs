//! Decodificador de arquivos GIMP XCF.
//!
//! Utiliza a biblioteca pure-Rust `xcf` para carregar o arquivo nativo do GIMP,
//! extrair as camadas e compor a imagem final na tela em formato RGBA8.

use std::io::Cursor;
use std::path::Path;

use xcf::PropertyIdentifier;

use crate::model::{Built, Frame, Master, check_dims};

pub fn decode_xcf_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 14 || !bytes.starts_with(b"gimp xcf ") {
        return Err("XCF inválido: magic 'gimp xcf ' ausente".to_owned());
    }

    let cursor = Cursor::new(bytes);
    let xcf_doc = xcf::Xcf::load(cursor).map_err(|err| format!("XCF: {err:?}"))?;

    let width = xcf_doc.width();
    let height = xcf_doc.height();
    check_dims(width, height)?;

    let mut canvas = vec![0u8; (width as usize) * (height as usize) * 4];

    // Layers são armazenadas do topo para o fundo; compor do fundo para o topo (reverse)
    for layer in xcf_doc.layers.iter().rev() {
        // Verificar se camada está visível
        let is_visible = layer.properties.iter().all(|p| {
            if let PropertyIdentifier::PropVisible = p.kind {
                // se existir PropVisible com payload 0, está invisível
                !matches!(&p.payload, xcf::PropertyPayload::Unknown(bytes) if bytes == &[0, 0, 0, 0])
            } else {
                true
            }
        });

        if !is_visible {
            continue;
        }

        // Determinar offsets (posição da camada no canvas)
        let (offset_x, offset_y) = layer
            .properties
            .iter()
            .find_map(|p| {
                if let PropertyIdentifier::PropOffsets = p.kind {
                    if let xcf::PropertyPayload::Unknown(bytes) = &p.payload {
                        if bytes.len() >= 8 {
                            let x = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                            let y = i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                            return Some((x, y));
                        }
                    }
                }
                None
            })
            .unwrap_or((0, 0));

        let lw = layer.width;
        let lh = layer.height;

        for ly in 0..lh {
            let cy = offset_y + (ly as i32);
            if cy < 0 || cy >= (height as i32) {
                continue;
            }
            let cy = cy as usize;

            for lx in 0..lw {
                let cx = offset_x + (lx as i32);
                if cx < 0 || cx >= (width as i32) {
                    continue;
                }
                let cx = cx as usize;

                if let Some(pixel) = layer.pixel(lx, ly) {
                    let [sr, sg, sb, sa] = pixel.0;
                    if sa == 0 {
                        continue;
                    }

                    let dst_idx = (cy * (width as usize) + cx) * 4;
                    if sa == 255 {
                        canvas[dst_idx] = sr;
                        canvas[dst_idx + 1] = sg;
                        canvas[dst_idx + 2] = sb;
                        canvas[dst_idx + 3] = 255;
                    } else {
                        let dr = canvas[dst_idx] as u32;
                        let dg = canvas[dst_idx + 1] as u32;
                        let db = canvas[dst_idx + 2] as u32;
                        let da = canvas[dst_idx + 3] as u32;

                        let sa_u = sa as u32;
                        let inv_sa = 255 - sa_u;

                        let out_a = sa_u + (da * inv_sa) / 255;
                        if out_a > 0 {
                            canvas[dst_idx] =
                                ((sr as u32 * sa_u + dr * da * inv_sa / 255) / out_a) as u8;
                            canvas[dst_idx + 1] =
                                ((sg as u32 * sa_u + dg * da * inv_sa / 255) / out_a) as u8;
                            canvas[dst_idx + 2] =
                                ((sb as u32 * sa_u + db * da * inv_sa / 255) / out_a) as u8;
                            canvas[dst_idx + 3] = out_a.min(255) as u8;
                        }
                    }
                }
            }
        }
    }

    let note = format!(
        "GIMP XCF: composição achatada de {} camadas ({}×{})",
        xcf_doc.layers.len(),
        width,
        height
    );

    Ok(Master::Frames(Built {
        frames: vec![Frame {
            rgba: canvas,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA (GIMP XCF)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "xcf",
        loop_forever: false,
    }))
}

#[allow(dead_code)]
pub fn decode_xcf_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo XCF: {err}"))?;
    decode_xcf_bytes(&bytes)
}
