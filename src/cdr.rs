//! Decodificador de arquivos CorelDRAW (.cdr).
//!
//! Suporta duas arquiteturas de contêineres do CorelDRAW:
//! 1. Moderno (CorelDRAW X4 a 2024+): Contêiner ZIP padrão contendo previews raster em
//!    `metadata/thumbnails/thumbnail.png` ou `metadata/thumbnails/thumbnail.bmp`.
//! 2. Legado (CorelDRAW v3 a X3): Contêiner RIFF contendo chunks `DISP` (DIB - Device Independent
//!    Bitmap) ou `bmp ` com imagens de pré-visualização.

use std::io::{Cursor, Read};

use image::ImageFormat;

use crate::model::{Master, check_dims};

/// Decodifica os bytes de um arquivo CorelDRAW (.cdr).
pub fn decode_cdr_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 12 {
        return Err("Arquivo CorelDRAW (.cdr) truncado ou inválido".to_owned());
    }

    // 1. CorelDRAW Moderno (X4+) baseado em contêiner ZIP (PK\x03\x04)
    if bytes.starts_with(b"PK\x03\x04")
        && let Ok(master) = decode_modern_cdr_zip(bytes) {
            return Ok(master);
        }

    // 2. CorelDRAW Legado (v3 a X3) baseado em contêiner RIFF
    if bytes.starts_with(b"RIFF")
        && let Ok(master) = decode_legacy_cdr_riff(bytes) {
            return Ok(master);
        }

    // 3. Fallback: tentar ZIP mesmo sem magic inicial estrito (ou ZIP variante)
    if let Ok(master) = decode_modern_cdr_zip(bytes) {
        return Ok(master);
    }

    // 4. Fallback: tentar varredura profunda por pré-visualizações embutidas
    if let Some(master) = scan_for_embedded_preview(bytes) {
        return Ok(master);
    }

    Err("Arquivo CorelDRAW (.cdr) sem pré-visualização raster embutida. Salve o arquivo no CorelDRAW com a opção de miniatura/preview ativada.".to_owned())
}

/// Decodifica preview raster em arquivos modernos do CorelDRAW (contêiner ZIP).
fn decode_modern_cdr_zip(bytes: &[u8]) -> Result<Master, String> {
    let cursor = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("CorelDRAW ZIP inválido: {e}"))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i)
            && let Ok(name) = file.name() {
                entries.push((i, name.to_string()));
            }
    }

    let preferred_paths = [
        "metadata/thumbnails/thumbnail.png",
        "metadata/thumbnails/thumbnail.bmp",
        "metadata/thumbnails/page1.png",
        "metadata/thumbnails/page1.bmp",
        "previews/thumbnail.png",
        "previews/thumbnail.bmp",
        "thumbnails/thumbnail.png",
        "thumbnails/thumbnail.bmp",
    ];

    let mut target_index = None;
    for pref in &preferred_paths {
        if let Some(&(idx, _)) = entries
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(pref))
        {
            target_index = Some(idx);
            break;
        }
    }

    if target_index.is_none() {
        for &(idx, ref name) in &entries {
            let lower = name.to_ascii_lowercase();
            if (lower.contains("thumb") || lower.contains("preview") || lower.contains("page1"))
                && (lower.ends_with(".png")
                    || lower.ends_with(".bmp")
                    || lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".webp"))
            {
                target_index = Some(idx);
                break;
            }
        }
    }

    let idx = target_index.ok_or_else(|| {
        "CorelDRAW (.cdr): pacote ZIP não contém pré-visualização raster".to_owned()
    })?;

    let mut file = archive
        .by_index(idx)
        .map_err(|e| format!("erro lendo preview do CorelDRAW: {e}"))?;

    let mut img_bytes = Vec::new();
    file.read_to_end(&mut img_bytes)
        .map_err(|e| format!("erro extraindo preview do CorelDRAW: {e}"))?;

    let dyn_img = image::load_from_memory(&img_bytes)
        .map_err(|e| format!("não foi possível decodificar preview raster do CorelDRAW: {e}"))?;

    let width = dyn_img.width();
    let height = dyn_img.height();
    check_dims(width, height)?;

    let note = format!(
        "CorelDRAW (.cdr): pré-visualização embutida extraída do contêiner ZIP ({width}×{height})"
    );

    Ok(Master::Still {
        image: dyn_img,
        bit_depth: 8,
        color: "RGBA (CorelDRAW)".to_owned(),
        mipmaps: 1,
        notes: vec![note],
        id: "cdr",
    })
}

/// Decodifica preview em arquivos legados do CorelDRAW (contêiner RIFF).
fn decode_legacy_cdr_riff(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 12 || !bytes.starts_with(b"RIFF") {
        return Err("CorelDRAW RIFF inválido: cabeçalho RIFF ausente".to_owned());
    }

    // Busca o chunk DISP ou bmp na árvore RIFF
    if let Some(img_data) = find_riff_preview(bytes)
        && let Ok(master) = decode_raw_preview_data(&img_data) {
            return Ok(master);
        }

    // Varredura de fallback
    if let Some(master) = scan_for_embedded_preview(bytes) {
        return Ok(master);
    }

    Err("CorelDRAW (.cdr): nenhum preview embutido encontrado no contêiner RIFF".to_owned())
}

/// Navega pela árvore de chunks do contêiner RIFF procurando DISP ou bmp.
fn find_riff_preview(bytes: &[u8]) -> Option<Vec<u8>> {
    find_in_riff_range(bytes, 12, bytes.len())
}

fn find_in_riff_range(bytes: &[u8], start: usize, end: usize) -> Option<Vec<u8>> {
    let mut pos = start;
    while pos + 8 <= end {
        let chunk_id = &bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let data_start = pos + 8;
        let data_end = (data_start + chunk_size).min(end);
        let chunk_data = &bytes[data_start..data_end];

        if chunk_id == b"DISP" || chunk_id == b"bmp " || chunk_id == b"BMP " || chunk_id == b"bmpt"
        {
            return Some(chunk_data.to_vec());
        } else if (chunk_id == b"LIST" || chunk_id == b"RIFF")
            && chunk_data.len() > 4
            && let Some(found) = find_in_riff_range(bytes, data_start + 4, data_end)
        {
            return Some(found);
        }

        let padded_size = (chunk_size + 1) & !1;
        pos = data_start.saturating_add(padded_size);
    }
    None
}

/// Decodifica os bytes brutos do preview (BMP completo, DIB isolado, PNG, WMF ou EMF).
fn decode_raw_preview_data(data: &[u8]) -> Result<Master, String> {
    if data.is_empty() {
        return Err("preview vazio".to_owned());
    }

    // 1. Arquivo BMP completo
    if data.starts_with(b"BM")
        && let Ok(dyn_img) = image::load_from_memory(data) {
            let width = dyn_img.width();
            let height = dyn_img.height();
            check_dims(width, height)?;
            return Ok(Master::Still {
                image: dyn_img,
                bit_depth: 8,
                color: "RGB (CorelDRAW)".to_owned(),
                mipmaps: 1,
                notes: vec![format!(
                    "CorelDRAW (.cdr): preview BMP extraído do contêiner ({width}×{height})"
                )],
                id: "cdr",
            });
        }

    // 2. Arquivo PNG completo
    if data.starts_with(b"\x89PNG\r\n\x1a\n")
        && let Ok(dyn_img) = image::load_from_memory(data) {
            let width = dyn_img.width();
            let height = dyn_img.height();
            check_dims(width, height)?;
            return Ok(Master::Still {
                image: dyn_img,
                bit_depth: 8,
                color: "RGBA (CorelDRAW)".to_owned(),
                mipmaps: 1,
                notes: vec![format!(
                    "CorelDRAW (.cdr): preview PNG extraído do contêiner ({width}×{height})"
                )],
                id: "cdr",
            });
        }

    // 3. DIB puro iniciando em offset 0 (biSize >= 40)
    if data.len() >= 40 {
        let bi_size = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if bi_size >= 40 && (bi_size as usize) <= data.len()
            && let Some(bmp_bytes) = dib_to_bmp(data)
                && let Ok(dyn_img) =
                    image::load_from_memory_with_format(&bmp_bytes, ImageFormat::Bmp)
                {
                    let width = dyn_img.width();
                    let height = dyn_img.height();
                    check_dims(width, height)?;
                    return Ok(Master::Still {
                        image: dyn_img,
                        bit_depth: 8,
                        color: "RGB (CorelDRAW DIB)".to_owned(),
                        mipmaps: 1,
                        notes: vec![format!(
                            "CorelDRAW (.cdr): preview DIB/DISP renderizado ({width}×{height})"
                        )],
                        id: "cdr",
                    });
                }
    }

    // 4. DIB com prefixo de 4 bytes (Clipboard format type DWORD, ex: CF_DIB=8)
    if data.len() >= 44 {
        let bi_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if bi_size >= 40 && (bi_size as usize) <= data.len() - 4
            && let Some(bmp_bytes) = dib_to_bmp(&data[4..])
                && let Ok(dyn_img) =
                    image::load_from_memory_with_format(&bmp_bytes, ImageFormat::Bmp)
                {
                    let width = dyn_img.width();
                    let height = dyn_img.height();
                    check_dims(width, height)?;
                    return Ok(Master::Still {
                        image: dyn_img,
                        bit_depth: 8,
                        color: "RGB (CorelDRAW DIB)".to_owned(),
                        mipmaps: 1,
                        notes: vec![format!(
                            "CorelDRAW (.cdr): preview DIB/DISP renderizado ({width}×{height})"
                        )],
                        id: "cdr",
                    });
                }
    }

    // 5. BMP completo após 4 bytes de cabeçalho clipboard
    if data.len() >= 6 && data[4..].starts_with(b"BM")
        && let Ok(dyn_img) = image::load_from_memory(&data[4..]) {
            let width = dyn_img.width();
            let height = dyn_img.height();
            check_dims(width, height)?;
            return Ok(Master::Still {
                image: dyn_img,
                bit_depth: 8,
                color: "RGB (CorelDRAW)".to_owned(),
                mipmaps: 1,
                notes: vec![format!(
                    "CorelDRAW (.cdr): preview BMP extraído ({width}×{height})"
                )],
                id: "cdr",
            });
        }

    // 6. WMF (Windows Metafile)
    if data.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A]) || data.starts_with(&[0x01, 0x00, 0x09, 0x00]) {
        if let Ok(built) = crate::emf::decode_wmf(data) {
            return Ok(Master::Frames(built));
        }
    } else if data.len() >= 8
        && (data[4..].starts_with(&[0xD7, 0xCD, 0xC6, 0x9A])
            || data[4..].starts_with(&[0x01, 0x00, 0x09, 0x00]))
        && let Ok(built) = crate::emf::decode_wmf(&data[4..]) {
            return Ok(Master::Frames(built));
        }

    // 7. EMF (Enhanced Metafile)
    if data.len() >= 44 && &data[40..44] == b" EMF"
        && let Ok(built) = crate::emf::decode_emf(data) {
            return Ok(Master::Frames(built));
        }

    Err("formato de preview não reconhecido nos dados".to_owned())
}

/// Converte uma estrutura DIB (BITMAPINFOHEADER + paleta opcional + pixels) em um arquivo BMP completo.
fn dib_to_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    if dib.len() < 40 {
        return None;
    }
    let bi_size = u32::from_le_bytes(dib[0..4].try_into().ok()?);
    if bi_size < 40 || (bi_size as usize) > dib.len() {
        return None;
    }
    let bi_bit_count = u16::from_le_bytes(dib[14..16].try_into().ok()?);
    let bi_compression = u32::from_le_bytes(dib[16..20].try_into().ok()?);
    let bi_clr_used = u32::from_le_bytes(dib[32..36].try_into().ok()?);

    let palette_size = if bi_clr_used != 0 {
        (bi_clr_used as usize) * 4
    } else if bi_bit_count <= 8 {
        (1usize << bi_bit_count) * 4
    } else if bi_compression == 3 {
        12 // 3 máscaras DWORD (R, G, B)
    } else {
        0
    };

    let off_bits = 14 + (bi_size as usize) + palette_size;
    let total_size = (14 + dib.len()) as u32;

    let mut bmp = Vec::with_capacity(14 + dib.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&total_size.to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); // reservados
    bmp.extend_from_slice(&(off_bits as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}

/// Varredura por assinaturas de pré-visualização embutidas em arquivos malformados ou incomuns.
fn scan_for_embedded_preview(bytes: &[u8]) -> Option<Master> {
    // 1. Procura chunk DISP
    if let Some(pos) = bytes.windows(4).position(|w| w == b"DISP")
        && pos + 8 <= bytes.len() {
            let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let end = (pos + 8 + size).min(bytes.len());
            if let Ok(master) = decode_raw_preview_data(&bytes[pos + 8..end]) {
                return Some(master);
            }
        }

    // 2. Procura imagem PNG embutida
    if let Some(pos) = bytes.windows(8).position(|w| w == b"\x89PNG\r\n\x1a\n")
        && let Ok(dyn_img) = image::load_from_memory(&bytes[pos..]) {
            let width = dyn_img.width();
            let height = dyn_img.height();
            if check_dims(width, height).is_ok() {
                return Some(Master::Still {
                    image: dyn_img,
                    bit_depth: 8,
                    color: "RGBA (CorelDRAW)".to_owned(),
                    mipmaps: 1,
                    notes: vec![format!(
                        "CorelDRAW (.cdr): preview PNG extraído ({width}×{height})"
                    )],
                    id: "cdr",
                });
            }
        }

    // 3. Procura imagem BMP embutida
    let mut search_pos = 0;
    while let Some(pos) = bytes[search_pos..].windows(2).position(|w| w == b"BM") {
        let abs_pos = search_pos + pos;
        if abs_pos + 14 <= bytes.len() {
            let file_size =
                u32::from_le_bytes(bytes[abs_pos + 2..abs_pos + 6].try_into().unwrap()) as usize;
            if file_size > 14 && file_size <= bytes.len() - abs_pos
                && let Ok(dyn_img) = image::load_from_memory(&bytes[abs_pos..abs_pos + file_size]) {
                    let width = dyn_img.width();
                    let height = dyn_img.height();
                    if check_dims(width, height).is_ok() {
                        return Some(Master::Still {
                            image: dyn_img,
                            bit_depth: 8,
                            color: "RGB (CorelDRAW)".to_owned(),
                            mipmaps: 1,
                            notes: vec![format!(
                                "CorelDRAW (.cdr): preview BMP extraído ({width}×{height})"
                            )],
                            id: "cdr",
                        });
                    }
                }
        }
        search_pos = abs_pos + 2;
        if search_pos + 14 > bytes.len() {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_modern_cdr_zip_png_preview() {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("metadata/thumbnails/thumbnail.png", options)
                .unwrap();
            let mut png_bytes = Vec::new();
            {
                let img = image::RgbaImage::new(10, 10);
                let mut c = Cursor::new(&mut png_bytes);
                img.write_to(&mut c, image::ImageFormat::Png).unwrap();
            }
            zip.write_all(&png_bytes).unwrap();
            zip.finish().unwrap();
        }

        let master = decode_cdr_bytes(&buf).expect("deve decodificar modern CDR");
        match master {
            Master::Still {
                image, id, notes, ..
            } => {
                assert_eq!(id, "cdr");
                assert_eq!(image.width(), 10);
                assert_eq!(image.height(), 10);
                assert!(notes[0].contains("ZIP"));
            }
            _ => panic!("esperava Master::Still"),
        }
    }

    #[test]
    fn test_legacy_cdr_riff_disp_dib() {
        // Gera um bitmap de 2x2 pixels 24bpp (DIB)
        // Largura: 2 px, Altura: 2 px (Bottom-Up)
        // Row pitch: (2 * 3 + 3) & !3 = 8 bytes
        let mut dib = Vec::new();
        // BITMAPINFOHEADER (40 bytes)
        dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
        dib.extend_from_slice(&2i32.to_le_bytes()); // biWidth
        dib.extend_from_slice(&2i32.to_le_bytes()); // biHeight
        dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
        dib.extend_from_slice(&24u16.to_le_bytes()); // biBitCount
        dib.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
        dib.extend_from_slice(&16u32.to_le_bytes()); // biSizeImage (2 rows * 8 bytes)
        dib.extend_from_slice(&2835i32.to_le_bytes()); // biXPelsPerMeter
        dib.extend_from_slice(&2835i32.to_le_bytes()); // biYPelsPerMeter
        dib.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
        dib.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant
        // Pixel data (2 rows of 8 bytes = 16 bytes)
        dib.extend_from_slice(&[255, 0, 0, 0, 255, 0, 0, 0]);
        dib.extend_from_slice(&[0, 0, 255, 255, 255, 255, 0, 0]);

        // Monta contêiner RIFF com chunk DISP e prefixo de 4 bytes (CF_DIB = 8)
        let mut disp_payload = Vec::new();
        disp_payload.extend_from_slice(&8u32.to_le_bytes()); // CF_DIB
        disp_payload.extend_from_slice(&dib);

        let chunk_size = disp_payload.len() as u32;
        let total_size = 4 + 8 + chunk_size; // "CDRA" (4) + "DISP" (4) + size (4) + payload

        let mut riff = Vec::new();
        riff.extend_from_slice(b"RIFF");
        riff.extend_from_slice(&total_size.to_le_bytes());
        riff.extend_from_slice(b"CDRA");
        riff.extend_from_slice(b"DISP");
        riff.extend_from_slice(&chunk_size.to_le_bytes());
        riff.extend_from_slice(&disp_payload);

        let master = decode_cdr_bytes(&riff).expect("deve decodificar legacy RIFF CDR");
        match master {
            Master::Still {
                image, id, notes, ..
            } => {
                assert_eq!(id, "cdr");
                assert_eq!(image.width(), 2);
                assert_eq!(image.height(), 2);
                assert!(notes[0].contains("DIB/DISP"));
            }
            _ => panic!("esperava Master::Still"),
        }
    }

    #[test]
    fn test_real_guildmaster_cdr() {
        let p = std::path::Path::new(r"C:\Users\bruno\Downloads\CAMISETA-GUILDMASTER-BRUNO.cdr");
        if p.exists() {
            let bytes = std::fs::read(p).unwrap();
            let master =
                decode_cdr_bytes(&bytes).expect("deve decodificar CAMISETA-GUILDMASTER-BRUNO.cdr");
            match master {
                Master::Still { image, id, .. } => {
                    assert_eq!(id, "cdr");
                    assert!(image.width() > 0 && image.height() > 0);
                }
                _ => panic!("esperava Master::Still"),
            }
        }
    }

    #[test]
    fn test_cdr_without_preview_fails_gracefully() {
        let mut riff = Vec::new();
        riff.extend_from_slice(b"RIFF");
        riff.extend_from_slice(&4u32.to_le_bytes());
        riff.extend_from_slice(b"CDRX");

        let res = decode_cdr_bytes(&riff);
        match res {
            Err(err) => assert!(err.contains("sem pré-visualização")),
            Ok(_) => panic!("deve falhar sem preview"),
        }
    }
}
