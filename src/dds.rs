//! DDS: o contêiner passa pelo ddsfile e os blocos BC pelo texture2ddecoder.
//! DirectXTex é C++/MSVC e não entra neste executável GNU.

use std::io::Cursor;

use ddsfile::{Dds, DxgiFormat};

use crate::model::{bgra_to_rgba, check_dims, Built, Frame};

pub fn decode_dds(data: &[u8]) -> Result<Built, String> {
    let dds = Dds::read(Cursor::new(data)).map_err(|err| format!("DDS inválido: {err}"))?;
    let width = dds.get_width();
    let height = dds.get_height();
    check_dims(width, height)?;
    let mipmaps = dds.get_num_mipmap_levels().max(1);
    let format = dds.get_dxgi_format();
    let code = format.map(dxgi_code).unwrap_or_else(|| legacy_fourcc(data));
    let layer = dds
        .get_data(0)
        .map_err(|err| format!("DDS sem texels: {err}"))?;
    let (rgba, color, note) = decode_texels(code, width, height, layer)?;
    let mut built = Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color,
        mipmaps,
        notes: Vec::new(),
        id: "dds",
        loop_forever: false,
    };
    if let Some(note) = note {
        built.notes.push(note);
    }
    if mipmaps > 1 {
        built.notes.push(format!(
            "{mipmaps} mipmaps no arquivo; a tela mostra o mip 0"
        ));
    }
    Ok(built)
}

fn dxgi_code(format: DxgiFormat) -> u32 {
    format as u32
}

fn legacy_fourcc(data: &[u8]) -> u32 {
    match data.get(84..88) {
        Some(b"DXT1") => 71,
        Some(b"DXT2") | Some(b"DXT3") => 74,
        Some(b"DXT4") | Some(b"DXT5") => 77,
        Some(b"ATI1") | Some(b"BC4U") => 80,
        Some(b"ATI2") | Some(b"BC5U") => 83,
        _ => 0,
    }
}

fn decode_texels(code: u32, width: u32, height: u32, data: &[u8]) -> Result<(Vec<u8>, String, Option<String>), String> {
    let pixels = width as usize * height as usize;
    match code {
        28 | 29 => {
            let need = pixels * 4;
            let bytes = data.get(..need).ok_or_else(|| "DDS RGBA truncado".to_owned())?;
            Ok((bytes.to_vec(), "RGBA".to_owned(), None))
        }
        87 | 88 | 91 => {
            let need = pixels * 4;
            let bytes = data.get(..need).ok_or_else(|| "DDS BGRA truncado".to_owned())?;
            let mut rgba = Vec::with_capacity(need);
            for px in bytes.chunks_exact(4) {
                let alpha = if code == 91 { 255 } else { px[3] };
                rgba.extend_from_slice(&[px[2], px[1], px[0], alpha]);
            }
            Ok((rgba, "RGBA".to_owned(), None))
        }
        71 | 72 => decode_blocks(width, height, data, 8, "BC1", |src, dst| {
            texture2ddecoder::decode_bc1(src, width as usize, height as usize, dst)
        }),
        74 | 75 => decode_blocks(width, height, data, 16, "BC2", |src, dst| {
            texture2ddecoder::decode_bc2(src, width as usize, height as usize, dst)
        }),
        77 | 78 => decode_blocks(width, height, data, 16, "BC3", |src, dst| {
            texture2ddecoder::decode_bc3(src, width as usize, height as usize, dst)
        }),
        80 | 81 => decode_blocks(width, height, data, 8, "BC4", |src, dst| {
            texture2ddecoder::decode_bc4(src, width as usize, height as usize, dst)
        }),
        83 | 84 => decode_blocks(width, height, data, 16, "BC5", |src, dst| {
            texture2ddecoder::decode_bc5(src, width as usize, height as usize, dst)
        }),
        95 => decode_blocks(width, height, data, 16, "BC6H", |src, dst| {
            texture2ddecoder::decode_bc6(src, width as usize, height as usize, dst, false)
        }),
        96 => decode_blocks(width, height, data, 16, "BC6H", |src, dst| {
            texture2ddecoder::decode_bc6(src, width as usize, height as usize, dst, true)
        }),
        98 | 99 => decode_blocks(width, height, data, 16, "BC7", |src, dst| {
            texture2ddecoder::decode_bc7(src, width as usize, height as usize, dst)
        }),
        other => Err(format!(
            "DDS com formato DXGI {other} não é lido. RGBA8 e BC1 a BC7 estão no decodificador."
        )),
    }
}

fn decode_blocks(
    width: u32,
    height: u32,
    data: &[u8],
    block_bytes: usize,
    name: &str,
    decode: impl FnOnce(&[u8], &mut [u32]) -> Result<(), &'static str>,
) -> Result<(Vec<u8>, String, Option<String>), String> {
    let blocks_x = (width as usize + 3) / 4;
    let blocks_y = (height as usize + 3) / 4;
    let need = blocks_x * blocks_y * block_bytes;
    let src = data.get(..need).ok_or_else(|| format!("DDS {name} truncado"))?;
    let mut pixels = vec![0u32; width as usize * height as usize];
    decode(src, &mut pixels).map_err(|err| format!("DDS {name}: {err}"))?;
    let note = if name == "BC1" {
        None
    } else {
        Some(format!(
            "{name} passou pelo texture2ddecoder; só BC1 e RGBA8 têm teste de cor nesta build"
        ))
    };
    Ok((bgra_to_rgba(&pixels), name.to_owned(), note))
}
