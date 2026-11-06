//! KTX 1 tem leitor próprio. KTX 2 usa o crate ktx2.
//! Só RGBA8 sem supercompressão: Basis continua fora.

use crate::binio::u32_le;
use crate::model::{check_dims, Built, Frame};

const KTX1: &[u8] = &[0xAB, b'K', b'T', b'X', b' ', b'1', b'1', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];

pub fn decode_ktx1(data: &[u8]) -> Result<Built, String> {
    if data.len() < 64 || !data.starts_with(KTX1) {
        return Err("KTX inválido".to_owned());
    }
    if u32_le(data, 12)? != 0x0403_0201 {
        return Err("KTX com endianness trocado não é lido".to_owned());
    }
    let gl_type = u32_le(data, 16)?;
    let type_size = u32_le(data, 20)?;
    let format = u32_le(data, 24)?;
    let width = u32_le(data, 36)?;
    let height = u32_le(data, 40)?.max(1);
    let depth = u32_le(data, 44)?;
    let array = u32_le(data, 48)?;
    let faces = u32_le(data, 52)?;
    let levels = u32_le(data, 56)?.max(1);
    let kv = u32_le(data, 60)? as usize;
    if depth > 1 || array > 0 || faces != 1 {
        return Err("KTX de volume, array ou cubemap não é lido".to_owned());
    }
    // GL_UNSIGNED_BYTE, GL_RGBA, GL_RGBA8
    if gl_type != 0x1401 || type_size != 1 || format != 0x1908 {
        return Err("KTX que não é RGBA8 não é lido".to_owned());
    }
    check_dims(width, height)?;
    let mut cursor = 64 + kv;
    let mut mipmaps = 0u32;
    let mut base = None;
    for level in 0..levels {
        let side = 1u32 << level;
        let lw = (width / side).max(1);
        let lh = (height / side).max(1);
        let image_size = u32_le(data, cursor)? as usize;
        cursor += 4;
        let bytes = data
            .get(cursor..cursor + image_size)
            .ok_or_else(|| "KTX truncado".to_owned())?;
        cursor += image_size;
        cursor = (cursor + 3) & !3;
        mipmaps += 1;
        if level == 0 {
            let need = lw as usize * lh as usize * 4;
            if bytes.len() < need {
                return Err("KTX com mip 0 incompleto".to_owned());
            }
            base = Some((bytes[..need].to_vec(), lw, lh));
        }
    }
    let (rgba, width, height) = base.ok_or_else(|| "KTX sem mip 0".to_owned())?;
    let mut built = still_ktx("ktx", width, height, rgba, mipmaps);
    if mipmaps > 1 {
        built.notes.push(format!(
            "{mipmaps} mipmaps; a tela mostra o nível 0"
        ));
    }
    Ok(built)
}

pub fn decode_ktx2(data: &[u8]) -> Result<Built, String> {
    let reader = ktx2::Reader::new(data).map_err(|err| format!("KTX2 inválido: {err}"))?;
    let header = reader.header();
    if header.supercompression_scheme.is_some() {
        return Err("KTX2 com supercompressão (Basis ou Zstd) não é lido".to_owned());
    }
    let width = header.pixel_width;
    let height = header.pixel_height.max(1);
    if header.pixel_depth > 1 || header.layer_count > 1 || header.face_count > 1 {
        return Err("KTX2 de volume, array ou cubemap não é lido".to_owned());
    }
    let format = header
        .format
        .ok_or_else(|| "KTX2 sem VkFormat (formato universal) não é lido".to_owned())?;
    let code = format.value();
    if !matches!(code, 37 | 43) {
        return Err(format!(
            "KTX2 VkFormat {code} não é RGBA8. Só UNORM e SRGB são lidos."
        ));
    }
    check_dims(width, height)?;
    let level = reader
        .levels()
        .next()
        .ok_or_else(|| "KTX2 sem mip".to_owned())?;
    // O índice do KTX2 começa pelo mip menor. O nível de base é o último.
    let base = reader
        .levels()
        .last()
        .ok_or_else(|| "KTX2 sem mip de base".to_owned())?;
    let _ = level;
    let need = width as usize * height as usize * 4;
    let bytes = base
        .data
        .get(..need)
        .ok_or_else(|| "KTX2 com texels incompletos".to_owned())?;
    let mipmaps = header.level_count.max(1);
    let mut built = still_ktx("ktx2", width, height, bytes.to_vec(), mipmaps);
    if mipmaps > 1 {
        built.notes.push(format!(
            "{mipmaps} mipmaps; a tela mostra o nível de base"
        ));
    }
    Ok(built)
}

fn still_ktx(id: &'static str, width: u32, height: u32, rgba: Vec<u8>, mipmaps: u32) -> Built {
    Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps,
        notes: Vec::new(),
        id,
        loop_forever: false,
    }
}
