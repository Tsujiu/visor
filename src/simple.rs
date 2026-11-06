//! Formatos pequenos com leitor próprio e teste de cor.
//! Cada um valida o cabeçalho antes de alocar a imagem.

use crate::binio::{i32_le, u16_be, u16_le, u32_be, u32_le};
use crate::model::{check_dims, Built, Frame};

pub fn decode_pcx(data: &[u8]) -> Result<Built, String> {
    if data.len() < 128 || data[0] != 0x0A {
        return Err("PCX inválido".to_owned());
    }
    let encoding = data[2];
    if encoding > 1 {
        return Err("PCX com codificação desconhecida".to_owned());
    }
    let bpp = data[3];
    let xmin = u16_le(data, 4)?;
    let ymin = u16_le(data, 6)?;
    let xmax = u16_le(data, 8)?;
    let ymax = u16_le(data, 10)?;
    let width = u32::from(xmax.saturating_sub(xmin)) + 1;
    let height = u32::from(ymax.saturating_sub(ymin)) + 1;
    check_dims(width, height)?;
    let planes = data[65];
    let bytes_per_line = u16_le(data, 66)? as usize;
    if bytes_per_line == 0 || planes == 0 || bytes_per_line > 16_384 {
        return Err("PCX com plano inválido".to_owned());
    }
    let expected = bytes_per_line
        .checked_mul(planes as usize)
        .and_then(|row| row.checked_mul(height as usize))
        .ok_or_else(|| "PCX grande demais".to_owned())?;
    if expected > crate::model::MAX_ALLOC as usize {
        return Err("PCX grande demais".to_owned());
    }
    let body = &data[128..];
    let packed = if encoding == 1 {
        rle_exact(body, expected)?
    } else {
        body.get(..expected)
            .ok_or_else(|| "PCX truncado".to_owned())?
            .to_vec()
    };
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    if planes == 3 && bpp == 8 {
        for y in 0..height as usize {
            let row = y * bytes_per_line * 3;
            for x in 0..width as usize {
                let i = (y * width as usize + x) * 4;
                rgba[i] = packed[row + x];
                rgba[i + 1] = packed[row + bytes_per_line + x];
                rgba[i + 2] = packed[row + bytes_per_line * 2 + x];
                rgba[i + 3] = 255;
            }
        }
    } else if planes == 1 && bpp == 8 {
        let palette = pcx_palette(data);
        for y in 0..height as usize {
            for x in 0..width as usize {
                let index = packed[y * bytes_per_line + x] as usize;
                let fallback = [index as u8, index as u8, index as u8];
                let rgb = palette.get(index * 3..index * 3 + 3).unwrap_or(&fallback);
                let i = (y * width as usize + x) * 4;
                rgba[i] = rgb[0];
                rgba[i + 1] = rgb[1];
                rgba[i + 2] = rgb[2];
                rgba[i + 3] = 255;
            }
        }
    } else {
        return Err(format!("PCX {bpp} bits e {planes} planos não é lido"));
    }
    Ok(still("pcx", width, height, rgba, 8, "RGB"))
}

fn pcx_palette(data: &[u8]) -> Vec<u8> {
    if data.len() >= 769 && data[data.len() - 769] == 0x0C {
        data[data.len() - 768..].to_vec()
    } else {
        (0..256).flat_map(|i| [i as u8, i as u8, i as u8]).collect()
    }
}

fn rle_exact(src: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(expected);
    let mut index = 0;
    while out.len() < expected {
        if index >= src.len() {
            return Err("PCX truncado".to_owned());
        }
        let byte = src[index];
        index += 1;
        if byte & 0xC0 == 0xC0 {
            let count = (byte & 0x3F) as usize;
            if count == 0 || index >= src.len() || out.len() + count > expected {
                return Err("PCX RLE inválido".to_owned());
            }
            let value = src[index];
            index += 1;
            out.extend(std::iter::repeat(value).take(count));
        } else {
            out.push(byte);
        }
    }
    Ok(out)
}

pub fn decode_wbmp(data: &[u8]) -> Result<Built, String> {
    if data.len() < 4 || data[0] != 0 || data[1] != 0 {
        return Err("WBMP inválido".to_owned());
    }
    let (width, index) = wbmp_varint(data, 2)?;
    let (height, index) = wbmp_varint(data, index)?;
    check_dims(width, height)?;
    let row = (width as usize + 7) / 8;
    let need = row * height as usize;
    let bits = data
        .get(index..index + need)
        .ok_or_else(|| "WBMP truncado".to_owned())?;
    let mut rgba = vec![255u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        for x in 0..width as usize {
            let bit = bits[y * row + x / 8] & (0x80 >> (x % 8));
            if bit != 0 {
                let i = (y * width as usize + x) * 4;
                rgba[i] = 0;
                rgba[i + 1] = 0;
                rgba[i + 2] = 0;
            }
        }
    }
    Ok(still("wbmp", width, height, rgba, 1, "preto e branco"))
}

fn wbmp_varint(data: &[u8], mut index: usize) -> Result<(u32, usize), String> {
    let mut value = 0u32;
    for _ in 0..4 {
        let byte = *data.get(index).ok_or_else(|| "WBMP truncado".to_owned())?;
        index += 1;
        value = (value << 7) | u32::from(byte & 0x7F);
        if byte & 0x80 == 0 {
            return Ok((value, index));
        }
    }
    Err("WBMP com dimensão inválida".to_owned())
}

pub fn decode_ras(data: &[u8]) -> Result<Built, String> {
    if u32_be(data, 0)? != 0x59A6_6A95 {
        return Err("RAS inválido".to_owned());
    }
    let width = u32_be(data, 4)?;
    let height = u32_be(data, 8)?;
    let depth = u32_be(data, 12)?;
    let kind = u32_be(data, 20)?;
    let map_type = u32_be(data, 24)?;
    let map_len = u32_be(data, 28)? as usize;
    check_dims(width, height)?;
    if kind != 1 {
        return Err("RAS comprimido não é lido".to_owned());
    }
    if map_type != 0 || map_len != 0 {
        return Err("RAS com mapa de cor não é lido".to_owned());
    }
    let channels: usize = match depth {
        24 => 3,
        32 => 4,
        _ => return Err(format!("RAS de {depth} bits não é lido")),
    };
    let stride = if depth == 24 {
        (width as usize * 3 + 1) & !1
    } else {
        width as usize * 4
    };
    let start = 32;
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        let row = data
            .get(start + y * stride..start + y * stride + width as usize * channels)
            .ok_or_else(|| "RAS truncado".to_owned())?;
        for x in 0..width as usize {
            let s = x * channels;
            let i = (y * width as usize + x) * 4;
            rgba[i] = row[s + 2];
            rgba[i + 1] = row[s + 1];
            rgba[i + 2] = row[s];
            rgba[i + 3] = if channels == 4 { row[s + 3] } else { 255 };
        }
    }
    Ok(still(
        "ras",
        width,
        height,
        rgba,
        8,
        if channels == 4 { "RGBA" } else { "RGB" },
    ))
}

pub fn decode_sgi(data: &[u8]) -> Result<Built, String> {
    if u16_be(data, 0)? != 474 {
        return Err("SGI inválido".to_owned());
    }
    let storage = *data.get(2).ok_or_else(|| "SGI truncado".to_owned())?;
    let bpc = *data.get(3).ok_or_else(|| "SGI truncado".to_owned())?;
    if bpc != 1 {
        return Err("SGI de 16 bits por canal não é lido".to_owned());
    }
    let xsize = u16_be(data, 6)? as u32;
    let ysize = u16_be(data, 8)? as u32;
    let zsize = u16_be(data, 10)? as u32;
    check_dims(xsize, ysize)?;
    if !matches!(zsize, 1 | 3 | 4) {
        return Err(format!("SGI com {zsize} canais não é lido"));
    }
    let mut rgba = vec![0u8; xsize as usize * ysize as usize * 4];
    if storage == 0 {
        let plane = xsize as usize * ysize as usize;
        for z in 0..zsize as usize {
            for y in 0..ysize as usize {
                let src_y = ysize as usize - 1 - y;
                let row = data
                    .get(512 + z * plane + src_y * xsize as usize..512 + z * plane + (src_y + 1) * xsize as usize)
                    .ok_or_else(|| "SGI truncado".to_owned())?;
                for x in 0..xsize as usize {
                    put_channel(&mut rgba, xsize, x, y, z, zsize as usize, row[x]);
                }
            }
        }
    } else if storage == 1 {
        let rows = zsize as usize * ysize as usize;
        for z in 0..zsize as usize {
            for y in 0..ysize as usize {
                let slot = z * ysize as usize + y;
                let offset = u32_be(data, 512 + slot * 4)? as usize;
                let line = sgi_rle(data.get(offset..).ok_or_else(|| "SGI RLE truncado".to_owned())?, xsize as usize)?;
                let dest_y = ysize as usize - 1 - y;
                for x in 0..xsize as usize {
                    put_channel(&mut rgba, xsize, x, dest_y, z, zsize as usize, line[x]);
                }
            }
        }
        let _ = rows;
    } else {
        return Err("SGI com armazenamento desconhecido".to_owned());
    }
    let color = match zsize {
        1 => "escala de cinza",
        4 => "RGBA",
        _ => "RGB",
    };
    Ok(still("sgi", xsize, ysize, rgba, 8, color))
}

fn put_channel(rgba: &mut [u8], width: u32, x: usize, y: usize, z: usize, zsize: usize, value: u8) {
    let i = (y * width as usize + x) * 4;
    if zsize == 1 {
        rgba[i] = value;
        rgba[i + 1] = value;
        rgba[i + 2] = value;
        rgba[i + 3] = 255;
    } else if z < 3 {
        rgba[i + z] = value;
        rgba[i + 3] = 255;
    } else {
        rgba[i + 3] = value;
    }
}

fn sgi_rle(src: &[u8], width: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(width);
    let mut index = 0;
    while out.len() < width {
        let byte = *src.get(index).ok_or_else(|| "SGI RLE truncado".to_owned())?;
        index += 1;
        let count = (byte & 0x7F) as usize;
        if count == 0 {
            break;
        }
        if out.len() + count > width {
            return Err("SGI RLE estoura a linha".to_owned());
        }
        if byte & 0x80 != 0 {
            let literal = src
                .get(index..index + count)
                .ok_or_else(|| "SGI RLE truncado".to_owned())?;
            out.extend_from_slice(literal);
            index += count;
        } else {
            let value = *src.get(index).ok_or_else(|| "SGI RLE truncado".to_owned())?;
            index += 1;
            out.extend(std::iter::repeat(value).take(count));
        }
    }
    if out.len() != width {
        return Err("SGI RLE incompleto".to_owned());
    }
    Ok(out)
}

pub fn decode_wal(data: &[u8]) -> Result<Built, String> {
    if data.len() < 100 {
        return Err("WAL truncado".to_owned());
    }
    let width = u32_le(data, 32)?;
    let height = u32_le(data, 36)?;
    check_dims(width, height)?;
    let mut mipmaps = 0u32;
    let mut pixels = None;
    for level in 0..4 {
        let offset = u32_le(data, 40 + level * 4)? as usize;
        if offset == 0 {
            continue;
        }
        let shifted_w = width >> level;
        let shifted_h = height >> level;
        if shifted_w == 0 && shifted_h == 0 {
            continue;
        }
        let lw = shifted_w.max(1);
        let lh = shifted_h.max(1);
        let need = lw as usize * lh as usize;
        let Some(slice) = data.get(offset..offset + need) else {
            continue;
        };
        mipmaps += 1;
        if level == 0 {
            pixels = Some(slice);
        }
    }
    let pixels = pixels.ok_or_else(|| "WAL sem o primeiro mip".to_owned())?;
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for index in pixels {
        rgba.extend_from_slice(&[*index, *index, *index, 255]);
    }
    let mut built = still("wal", width, height, rgba, 8, "índice em cinza");
    built.mipmaps = mipmaps.max(1);
    built.notes.push(
        "WAL não traz paleta; cada índice virou cinza. Isso é leitura parcial.".to_owned(),
    );
    Ok(built)
}

pub fn decode_dib(data: &[u8]) -> Result<Built, String> {
    let header = u32_le(data, 0)?;
    if !matches!(header, 40 | 108 | 124) {
        return Err("DIB inválido".to_owned());
    }
    let width = i32_le(data, 4)?;
    let height_raw = i32_le(data, 8)?;
    let top_down = height_raw < 0;
    let height = height_raw.unsigned_abs();
    let planes = u16_le(data, 12)?;
    let bits = u16_le(data, 14)?;
    let compression = u32_le(data, 16)?;
    if planes != 1 || compression != 0 {
        return Err("DIB comprimido não é lido".to_owned());
    }
    if !matches!(bits, 24 | 32) {
        return Err(format!("DIB de {bits} bits não é lido"));
    }
    let width = u32::try_from(width).map_err(|_| "DIB com largura inválida".to_owned())?;
    check_dims(width, height)?;
    let channels = (bits / 8) as usize;
    let stride = (width as usize * channels + 3) & !3;
    let pixels = data
        .get(header as usize..)
        .ok_or_else(|| "DIB truncado".to_owned())?;
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        let src_y = if top_down { y } else { height as usize - 1 - y };
        let row = pixels
            .get(src_y * stride..src_y * stride + width as usize * channels)
            .ok_or_else(|| "DIB truncado".to_owned())?;
        for x in 0..width as usize {
            let s = x * channels;
            let i = (y * width as usize + x) * 4;
            rgba[i] = row[s + 2];
            rgba[i + 1] = row[s + 1];
            rgba[i + 2] = row[s];
            rgba[i + 3] = if channels == 4 { row[s + 3] } else { 255 };
        }
    }
    Ok(still(
        "dib",
        width,
        height,
        rgba,
        8,
        if channels == 4 { "RGBA" } else { "RGB" },
    ))
}

pub fn decode_ico(data: &[u8]) -> Result<Built, String> {
    decode_icon(data, 1, "ico")
}

pub fn decode_cur(data: &[u8]) -> Result<Built, String> {
    decode_icon(data, 2, "cur")
}

fn decode_icon(data: &[u8], kind: u16, id: &'static str) -> Result<Built, String> {
    if data.len() < 22 || u16_le(data, 0)? != 0 || u16_le(data, 2)? != kind {
        return Err(format!("{id} inválido"));
    }
    let count = u16_le(data, 4)? as usize;
    if count == 0 || count > 64 {
        return Err(format!("{id} sem imagens"));
    }
    let mut best: Option<(u32, usize, usize)> = None;
    for index in 0..count {
        let at = 6 + index * 16;
        let entry = crate::binio::take(data, at, 16)?;
        let mut width = u32::from(entry[0]);
        let mut height = u32::from(entry[1]);
        if width == 0 {
            width = 256;
        }
        if height == 0 {
            height = 256;
        }
        let bytes = u32_le(entry, 8)? as usize;
        let offset = u32_le(entry, 12)? as usize;
        if best.as_ref().is_none_or(|item| width * height > item.0) {
            best = Some((width * height, offset, bytes));
        }
    }
    let Some((_, offset, bytes)) = best else {
        return Err(format!("{id} vazio"));
    };
    let image = data
        .get(offset..offset + bytes)
        .ok_or_else(|| format!("{id} truncado"))?;
    if image.starts_with(&[0x89, b'P', b'N', b'G']) {
        let decoded = image::load_from_memory(image).map_err(|err| format!("PNG do {id}: {err}"))?;
        let width = decoded.width();
        let height = decoded.height();
        check_dims(width, height)?;
        return Ok(still(
            id,
            width,
            height,
            decoded.into_rgba8().into_raw(),
            8,
            "RGBA",
        ));
    }
    let mut dib = decode_dib(image)?;
    dib.id = id;
    if id == "cur" {
        dib.notes.push("cursor com bitmap embutido".to_owned());
    }
    Ok(dib)
}

pub fn still(
    id: &'static str,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    bit_depth: u8,
    color: &str,
) -> Built {
    Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth,
        color: color.to_owned(),
        mipmaps: 1,
        notes: Vec::new(),
        id,
        loop_forever: false,
    }
}
