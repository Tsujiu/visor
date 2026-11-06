//! EXIF e ICC são só informação. Nada embutido é executado nem aplicado como código.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use exif::{Reader, Tag};

pub fn jpeg_is_cmyk(head: &[u8]) -> bool {
    let mut index = 0;
    while index + 1 < head.len() {
        if head[index] != 0xFF {
            index += 1;
            continue;
        }
        let marker = head[index + 1];
        if matches!(marker, 0xC0 | 0xC1 | 0xC2) {
            return head.get(index + 9).copied() == Some(4);
        }
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            index += 2;
            continue;
        }
        if index + 4 > head.len() {
            break;
        }
        let len = u16::from_be_bytes([head[index + 2], head[index + 3]]) as usize;
        if len < 2 {
            break;
        }
        index += 2 + len;
    }
    false
}

pub fn icc_note(data: &[u8]) -> Option<String> {
    let pos = data.windows(4).position(|window| window == b"acsp")?;
    if pos >= 36 {
        let size = u32::from_be_bytes(data[pos - 36..pos - 32].try_into().ok()?);
        if (128..20_000_000).contains(&size) {
            return Some(format!(
                "perfil ICC presente ({size} bytes); não aplicado"
            ));
        }
    }
    Some(format!(
        "perfil ICC presente (assinatura acsp no byte {pos}); não aplicado"
    ))
}

pub fn read_icc_file(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = vec![0u8; 8 * 1024 * 1024];
    let read = file.read(&mut buf).ok()?;
    icc_note(&buf[..read])
}

pub fn read_exif(path: &Path) -> Vec<(String, String)> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let Ok(meta) = file.metadata() else {
        return Vec::new();
    };
    if meta.len() > 48 * 1024 * 1024 {
        return vec![(
            "EXIF".to_owned(),
            "não lido: arquivo acima de 48 MB".to_owned(),
        )];
    }
    let mut reader = BufReader::new(file);
    let Ok(exif) = Reader::new().read_from_container(&mut reader) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for field in exif.fields() {
        if field.tag == Tag::MakerNote {
            continue;
        }
        let mut value = field.display_value().to_string();
        if value.len() > 160 {
            value.truncate(160);
            value.push('…');
        }
        rows.push((field.tag.to_string(), value));
        if rows.len() == 40 {
            break;
        }
    }
    rows
}
