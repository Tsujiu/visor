//! Decodificador de texturas PowerVR PVR (PVR v2 e PVR v3).
//! Decodificação de contêiner e blocos PVRTC, ETC, DXT e RGBA raw via texture2ddecoder.

use crate::binio::u32_le;
use crate::model::{Built, Frame, bgra_to_rgba, check_dims};

fn u64_le(data: &[u8], offset: usize) -> Result<u64, String> {
    let slice = data
        .get(offset..offset + 8)
        .ok_or("offset u64 fora dos limites")?;
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}

const PVR3_MAGIC: u32 = 0x0352_5650; // b"PVR\x03"
const PVR2_MAGIC: u32 = 0x2152_5650; // b"PVR!"

pub fn decode_pvr_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 52 {
        return Err("Arquivo PVR truncado".to_owned());
    }

    if u32_le(data, 0).ok() == Some(PVR3_MAGIC) {
        decode_pvr3(data)
    } else if data.len() >= 52 && u32_le(data, 44).ok() == Some(PVR2_MAGIC) {
        decode_pvr2(data)
    } else {
        Err("Assinatura mágica PVR v2/v3 não reconhecida".to_owned())
    }
}

fn decode_pvr3(data: &[u8]) -> Result<Built, String> {
    let pixel_format = u64_le(data, 8)?;
    let height = u32_le(data, 24)?;
    let width = u32_le(data, 28)?;
    let meta_size = u32_le(data, 48)? as usize;

    check_dims(width, height)?;

    let header_len = 52 + meta_size;
    let payload = data
        .get(header_len..)
        .ok_or_else(|| "PVR v3 sem dados de textura".to_owned())?;

    let w = width as usize;
    let h = height as usize;
    let mut pixels = vec![0u32; w * h];

    let format_name = match pixel_format {
        0 | 1 => {
            // PVRTC 2bpp RGB / RGBA
            texture2ddecoder::decode_pvrtc_2bpp(payload, w, h, &mut pixels)
                .map_err(|err| format!("PVRTC 2bpp: {err}"))?;
            "PVRTC 2bpp"
        }
        2 | 3 => {
            // PVRTC 4bpp RGB / RGBA
            texture2ddecoder::decode_pvrtc_4bpp(payload, w, h, &mut pixels)
                .map_err(|err| format!("PVRTC 4bpp: {err}"))?;
            "PVRTC 4bpp"
        }
        6 => {
            // ETC1
            texture2ddecoder::decode_etc1(payload, w, h, &mut pixels)
                .map_err(|err| format!("ETC1: {err}"))?;
            "ETC1"
        }
        7 => {
            // DXT1 / BC1
            texture2ddecoder::decode_bc1(payload, w, h, &mut pixels)
                .map_err(|err| format!("DXT1: {err}"))?;
            "DXT1"
        }
        9 => {
            // DXT3 / BC2
            texture2ddecoder::decode_bc2(payload, w, h, &mut pixels)
                .map_err(|err| format!("DXT3: {err}"))?;
            "DXT3"
        }
        11 => {
            // DXT5 / BC3
            texture2ddecoder::decode_bc3(payload, w, h, &mut pixels)
                .map_err(|err| format!("DXT5: {err}"))?;
            "DXT5"
        }
        _ => {
            // Uncompressed RGBA8888 fallback
            let need = w * h * 4;
            if payload.len() >= need {
                return Ok(Built {
                    frames: vec![Frame {
                        rgba: payload[..need].to_vec(),
                        width,
                        height,
                        delay_ms: 0,
                    }],
                    bit_depth: 8,
                    color: "RGBA".to_owned(),
                    mipmaps: 1,
                    notes: vec![format!("PowerVR PVR v3 RGBA ({width}×{height})")],
                    id: "pvr",
                    loop_forever: false,
                });
            }
            return Err(format!("Formato PVR v3 ({pixel_format}) não suportado"));
        }
    };

    let rgba = bgra_to_rgba(&pixels);

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
        notes: vec![format!("PowerVR PVR v3 {format_name} ({width}×{height})")],
        id: "pvr",
        loop_forever: false,
    })
}

fn decode_pvr2(data: &[u8]) -> Result<Built, String> {
    let height = u32_le(data, 4)?;
    let width = u32_le(data, 8)?;
    let flags = u32_le(data, 16)?;

    check_dims(width, height)?;

    let payload = data
        .get(52..)
        .ok_or_else(|| "PVR v2 sem dados de textura".to_owned())?;

    let format_code = flags & 0xFF;
    let w = width as usize;
    let h = height as usize;
    let mut pixels = vec![0u32; w * h];

    let format_name = match format_code {
        0x18 => {
            texture2ddecoder::decode_pvrtc_2bpp(payload, w, h, &mut pixels)
                .map_err(|err| format!("PVRTC 2bpp: {err}"))?;
            "PVRTC 2bpp"
        }
        0x19 => {
            texture2ddecoder::decode_pvrtc_4bpp(payload, w, h, &mut pixels)
                .map_err(|err| format!("PVRTC 4bpp: {err}"))?;
            "PVRTC 4bpp"
        }
        0x16 | 0x17 => {
            // RGBA8888
            let need = w * h * 4;
            if payload.len() >= need {
                return Ok(Built {
                    frames: vec![Frame {
                        rgba: payload[..need].to_vec(),
                        width,
                        height,
                        delay_ms: 0,
                    }],
                    bit_depth: 8,
                    color: "RGBA".to_owned(),
                    mipmaps: 1,
                    notes: vec![format!("PowerVR PVR v2 RGBA ({width}×{height})")],
                    id: "pvr",
                    loop_forever: false,
                });
            }
            return Err("PVR v2 RGBA truncado".to_owned());
        }
        _ => return Err(format!("Formato PVR v2 (0x{format_code:X}) não suportado")),
    };

    let rgba = bgra_to_rgba(&pixels);

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
        notes: vec![format!("PowerVR PVR v2 {format_name} ({width}×{height})")],
        id: "pvr",
        loop_forever: false,
    })
}
