//! Decodificador Canon CR3 RAW (ISOBMFF / CRX).
//! Extrai a imagem de prévia de alta resolução incorporada pela câmera no contêiner CRX/HEIF.

use crate::model::{Built, Frame, check_dims};

const JPEG_SOI: &[u8] = &[0xFF, 0xD8, 0xFF];
const JPEG_EOI: &[u8] = &[0xFF, 0xD9];

pub fn decode_cr3_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 16 {
        return Err("Arquivo CR3 truncado".to_owned());
    }

    // Valida marcação ISOBMFF ftyp crx
    let is_crx = data.len() >= 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"crx "
            || data[8..data.len().min(32)].windows(4).any(|w| w == b"crx "));

    if !is_crx && !data.starts_with(b"\x00\x00\x00\x18ftypcrx ") {
        return Err("Assinatura Canon CR3 (ftyp crx) não encontrada".to_owned());
    }

    // Localiza os marcadores de prévia JPEG embutida no contêiner
    // As câmeras Canon inserem pre-views JPEG (incluindo a de resolução total no box PRV)
    let mut best_jpeg: Option<&[u8]> = None;
    let mut max_len = 0usize;

    let mut cursor = 0;
    while cursor + 4 < data.len() {
        if data[cursor..].starts_with(JPEG_SOI) {
            let start = cursor;
            // Busca o final do JPEG (EOI: FF D9)
            if let Some(pos) = data[start + 2..].windows(2).position(|w| w == JPEG_EOI) {
                let end = start + 2 + pos + 2;
                let len = end - start;
                // Prefere a maior prévia disponível (normalmente > 500 KB)
                if len > max_len && len > 1024 {
                    max_len = len;
                    best_jpeg = Some(&data[start..end]);
                }
                cursor = end;
                continue;
            }
        }
        cursor += 1;
    }

    let jpeg_bytes = best_jpeg.ok_or_else(|| {
        "Não foi possível localizar o fluxo de imagem de prévia no contêiner CR3".to_owned()
    })?;

    let img = image::load_from_memory(jpeg_bytes)
        .map_err(|err| format!("Falha ao decodificar prévia JPEG do Canon CR3: {err}"))?;

    let width = img.width();
    let height = img.height();
    check_dims(width, height)?;

    let rgba = img.into_rgba8().into_raw();

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "sRGB".to_owned(),
        mipmaps: 1,
        notes: vec![format!(
            "Canon CR3 RAW (Prévia de alta resolução {width}×{height})"
        )],
        id: "cr3",
        loop_forever: false,
    })
}
