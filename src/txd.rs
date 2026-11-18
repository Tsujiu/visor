//! RenderWare TXD texture dictionary decoder (.txd).

use crate::model::{Built, Frame};
use libtxd::dictionary::TextureDictionary;
use std::io::Cursor;

pub fn decode_txd_bytes(data: &[u8]) -> Result<Built, String> {
    let mut cursor = Cursor::new(data);
    let dict = TextureDictionary::from_reader(&mut cursor)
        .map_err(|e| format!("erro ao analisar TXD: {e:?}"))?;

    if dict.textures.is_empty() {
        return Err("o arquivo TXD não contém texturas".to_owned());
    }

    let mut frames = Vec::new();
    let mut notes = Vec::new();
    let mut total_mipmaps = 0;

    for (i, tex) in dict.textures.iter().enumerate() {
        let rgba_img = match tex.to_rgba8(0) {
            Ok(img) => img,
            Err(e) => {
                if i == 0 {
                    return Err(format!(
                        "falha ao converter textura inicial '{}': {e:?}",
                        tex.name
                    ));
                } else {
                    continue;
                }
            }
        };

        let width = rgba_img.width();
        let height = rgba_img.height();
        let rgba = rgba_img.into_raw();

        if i == 0 {
            total_mipmaps = tex.mipmaps.len();
        }

        frames.push(Frame {
            rgba,
            width,
            height,
            delay_ms: 1000,
        });
    }

    if frames.is_empty() {
        return Err("nenhuma textura válida pôde ser decodificada do TXD".to_owned());
    }

    let count = frames.len();
    let first = &dict.textures[0];
    notes.push(format!(
        "RenderWare TXD: {count} textura(s), primeira='{}', compressão={:?}, {total_mipmaps} mipmaps",
        first.name, first.compression
    ));

    Ok(Built {
        frames,
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: total_mipmaps as u32,
        notes,
        id: "txd",
        loop_forever: false,
    })
}
