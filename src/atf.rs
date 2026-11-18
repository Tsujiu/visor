//! Decodificador de texturas Adobe ATF (Adobe Texture Format).
//! Suporta formatos descompactados RGB888/RGBA8888 e compactados DXT1/DXT5 via texture2ddecoder.

use crate::model::{Built, Frame, bgra_to_rgba, check_dims};

pub fn decode_atf_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 12 || !data.starts_with(b"ATF") {
        return Err("Arquivo ATF inválido ou truncado".to_owned());
    }

    // Estrutura do cabeçalho ATF padrão:
    // 0..3: "ATF"
    // 3: Versão / Assinatura (ex: 0x00, 0xFF, 0x01..0x04)
    // 4..7 ou 4..8: Comprimento do payload
    // Seguido por: formato (1 byte), log2(largura) (1 byte), log2(altura) (1 byte), contagem (1 byte)
    let header_offset = if data[3] == 0xFF {
        // ATF v2+ com extensão
        12
    } else {
        8
    };

    if data.len() < header_offset + 4 {
        return Err("Cabeçalho ATF incompleto".to_owned());
    }

    let format_byte = data[header_offset];
    let log2_w = data[header_offset + 1];
    let log2_h = data[header_offset + 2];
    let _mip_count = data[header_offset + 3];

    if log2_w > 14 || log2_h > 14 {
        return Err("Dimensões ATF excedem o limite de 16384 px".to_owned());
    }

    let width = 1u32 << log2_w;
    let height = 1u32 << log2_h;
    check_dims(width, height)?;

    let payload = &data[header_offset + 4..];
    let w = width as usize;
    let h = height as usize;

    let (rgba, format_name) = match format_byte {
        0 => {
            // RGB888
            let need = w * h * 3;
            let src = payload
                .get(..need)
                .ok_or_else(|| "ATF RGB888 truncado".to_owned())?;
            let mut out = Vec::with_capacity(w * h * 4);
            for chunk in src.chunks_exact(3) {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(chunk[2]);
                out.push(255);
            }
            (out, "RGB888")
        }
        1 => {
            // RGBA8888
            let need = w * h * 4;
            let src = payload
                .get(..need)
                .ok_or_else(|| "ATF RGBA8888 truncado".to_owned())?;
            (src.to_vec(), "RGBA8888")
        }
        2 => {
            // DXT1
            let need = ((w + 3) / 4) * ((h + 3) / 4) * 8;
            let src = payload
                .get(..need)
                .ok_or_else(|| "ATF DXT1 truncado".to_owned())?;
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc1(src, w, h, &mut pixels)
                .map_err(|err| format!("ATF DXT1: {err}"))?;
            (bgra_to_rgba(&pixels), "DXT1")
        }
        4 => {
            // DXT5
            let need = ((w + 3) / 4) * ((h + 3) / 4) * 16;
            let src = payload
                .get(..need)
                .ok_or_else(|| "ATF DXT5 truncado".to_owned())?;
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc3(src, w, h, &mut pixels)
                .map_err(|err| format!("ATF DXT5: {err}"))?;
            (bgra_to_rgba(&pixels), "DXT5")
        }
        other => return Err(format!("Formato de textura ATF ({other}) não suportado")),
    };

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: format!("RGBA ({format_name})"),
        mipmaps: 1,
        notes: vec![format!("Adobe ATF {format_name} ({width}×{height})")],
        id: "atf",
        loop_forever: false,
    })
}
