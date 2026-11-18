//! O formato real sai dos bytes. A extensão só desempata magics fracos.

use crate::binio::{u16_le, u32_be, u32_le};
use crate::catalog;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Tiff,
    Webp,
    Ico,
    Cur,
    Ani,
    Tga,
    Qoi,
    Pnm,
    Hdr,
    Exr,
    Svg,
    Jxl,
    Dds,
    Ktx,
    Ktx2,
    Pcx,
    Wbmp,
    Ras,
    Sgi,
    Wal,
    Dib,
    Emf,
    Raw,
    Psd,
    Jp2,
    Xcf,
    Ora,
    Farbfeld,
    Xbm,
    Avif,
    Wmf,
    Mng,
    Lottie,
    Tgs,
    Dxf,
    Cr3,
    Astc,
    Basis,
    Pvr,
    Atf,
    Vtf,
    Blp,
    Spr,
    Txd,
    Tex,
    Cdr,
    Unsupported(&'static str),
    Unknown,
}

const RAW_EXT: &[&str] = &[
    "raw", "dng", "cr2", "crw", "nef", "nrw", "arw", "srf", "sr2", "raf", "orf", "rw2", "pef",
    "3fr", "iiq", "erf", "kdc", "dcr", "mos", "mrw", "x3f", "bay", "cap",
];

pub fn sniff(head: &[u8], tail: &[u8], ext: Option<&str>) -> Kind {
    let ext = ext.map(|ext| ext.to_ascii_lowercase());
    let ext = ext.as_deref();

    if head.starts_with(b"8BPS") {
        return Kind::Psd;
    }
    if head.starts_with(b"gimp xcf ") {
        return Kind::Xcf;
    }
    if head.starts_with(b"farbfeld") {
        return Kind::Farbfeld;
    }
    if head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"ACON" {
        return Kind::Ani;
    }
    if head.starts_with(&[0x00, 0x00, 0x00, 0x0C, b'j', b'P', b' ', b' '])
        || head.starts_with(&[0xFF, 0x4F, 0xFF, 0x51])
    {
        return Kind::Jp2;
    }
    if head.starts_with(&[0x00, b'M', b'R', b'M']) {
        return Kind::Raw;
    }
    if head.starts_with(&[0x50, 0x4B, 0x03, 0x04])
        && (ext == Some("ora") || head.windows(17).any(|w| w == b"image/openraster"))
    {
        return Kind::Ora;
    }
    if head.starts_with(&[0x50, 0x4B, 0x03, 0x04]) && ext == Some("cdr") {
        return Kind::Cdr;
    }
    if is_xbm_text(head) {
        return Kind::Xbm;
    }

    if head.starts_with(&[
        0xAB, b'K', b'T', b'X', b' ', b'2', b'0', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]) {
        return Kind::Ktx2;
    }
    if head.starts_with(&[
        0xAB, b'K', b'T', b'X', b' ', b'1', b'1', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]) {
        return Kind::Ktx;
    }
    if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Kind::Png;
    }
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Kind::Jpeg;
    }
    if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        return Kind::Gif;
    }
    if head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"WEBP" {
        return Kind::Webp;
    }
    if head.starts_with(b"BM") {
        return Kind::Bmp;
    }
    if head.starts_with(b"DDS ") {
        return Kind::Dds;
    }
    if head.starts_with(&[0x76, 0x2F, 0x31, 0x01]) {
        return Kind::Exr;
    }
    if head.starts_with(b"#?RADIANCE") || head.starts_with(b"#?RGBE") {
        return Kind::Hdr;
    }
    if head.starts_with(&[0xFF, 0x0A])
        || head.starts_with(&[
            0x00, 0x00, 0x00, 0x0C, b'J', b'X', b'L', b' ', 0x0D, 0x0A, 0x87, 0x0A,
        ])
    {
        return Kind::Jxl;
    }
    if head.starts_with(b"qoif") {
        return Kind::Qoi;
    }
    if head.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return Kind::Ico;
    }
    if head.starts_with(&[0x00, 0x00, 0x02, 0x00]) {
        return Kind::Cur;
    }
    if u32_be(head, 0).ok() == Some(0x59A6_6A95) {
        return Kind::Ras;
    }
    if u16_be_ok(head) == Some(474) {
        return Kind::Sgi;
    }
    if head.starts_with(&[0x13, 0xAB, 0xA1, 0x5C]) {
        return Kind::Astc;
    }
    if head.starts_with(b"VTF\0") {
        return Kind::Vtf;
    }
    if head.starts_with(b"BLP0") || head.starts_with(b"BLP1") || head.starts_with(b"BLP2") {
        return Kind::Blp;
    }
    if head.starts_with(b"\x8AMNG\r\n\x1A\n") {
        return Kind::Mng;
    }
    if head.starts_with(b"sB") {
        return Kind::Basis;
    }
    if head.starts_with(b"PVR\x03") || head.starts_with(&[0x34, 0x00, 0x00, 0x00]) {
        return Kind::Pvr;
    }
    if head.starts_with(b"ATF") {
        return Kind::Atf;
    }
    if head.starts_with(b"IDSP") {
        return Kind::Spr;
    }
    if head.starts_with(b"KTEX") {
        return Kind::Tex;
    }
    if head.len() >= 4 && u32_le(head, 0).ok() == Some(0x16) && ext == Some("txd") {
        return Kind::Txd;
    }
    if head.starts_with(b"%PDF") {
        return Kind::Unsupported("pdf");
    }
    if head.len() >= 12 && &head[4..8] == b"ftyp" {
        let brand = &head[8..head.len().min(32)];
        if brand.windows(4).any(|b| b == b"avif" || b == b"avis") {
            return Kind::Avif;
        }
        if brand.windows(3).any(|b| b == b"crx") {
            return Kind::Cr3;
        }
    }
    if is_emf(head) {
        return Kind::Emf;
    }
    if head.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A]) || head.starts_with(&[0x01, 0x00, 0x09, 0x00]) {
        return Kind::Wmf;
    }
    if head.starts_with(b"%!PS") || head.starts_with(b"%!ps") {
        return match ext {
            Some("eps") | Some("epsf") => Kind::Unsupported("eps"),
            Some("ai") => Kind::Unsupported("ai"),
            _ => Kind::Unsupported("ps"),
        };
    }
    if head.len() >= 12
        && head.starts_with(b"RIFF")
        && (&head[8..11] == b"CDR" || &head[8..12] == b"CDRF")
    {
        return Kind::Cdr;
    }
    if is_pnm(head) {
        return Kind::Pnm;
    }
    if is_svg_text(head) {
        return Kind::Svg;
    }
    if head.starts_with(&[0x1F, 0x8B]) {
        return match ext {
            Some("svgz") | Some("svg") => Kind::Svg,
            Some("tgs") => Kind::Tgs,
            _ => Kind::Unknown,
        };
    }
    if looks_like_dxf(head) {
        return Kind::Dxf;
    }
    if tga_footer(tail) || (ext == Some("tga") && looks_like_tga(head)) {
        return Kind::Tga;
    }
    if looks_like_pcx(head) && (ext == Some("pcx") || strict_pcx(head)) {
        return Kind::Pcx;
    }
    if ext == Some("wbmp") && head.first() == Some(&0) {
        return Kind::Wbmp;
    }
    if ext == Some("dib") && looks_like_dib(head) {
        return Kind::Dib;
    }
    if ext == Some("wal") && looks_like_wal(head) {
        return Kind::Wal;
    }
    if is_tiff(head) {
        if ext.is_some_and(|ext| RAW_EXT.contains(&ext)) {
            return Kind::Raw;
        }
        return Kind::Tiff;
    }
    if ext.is_some_and(|ext| RAW_EXT.contains(&ext)) {
        return Kind::Raw;
    }
    if ext == Some("cr3") {
        return Kind::Cr3;
    }
    if ext == Some("lottie") || (ext == Some("json") && looks_like_lottie(head)) {
        return Kind::Lottie;
    }
    if let Some(ext) = ext {
        if let Some(spec) = catalog::find_extension(ext) {
            return kind_from_id(spec.id);
        }
    }
    Kind::Unknown
}

fn kind_from_id(id: &str) -> Kind {
    match id {
        "png" => Kind::Png,
        "jpeg" => Kind::Jpeg,
        "gif" => Kind::Gif,
        "bmp" => Kind::Bmp,
        "dib" => Kind::Dib,
        "tiff" => Kind::Tiff,
        "webp" => Kind::Webp,
        "jxl" => Kind::Jxl,
        "ico" => Kind::Ico,
        "cur" => Kind::Cur,
        "tga" => Kind::Tga,
        "pcx" => Kind::Pcx,
        "qoi" => Kind::Qoi,
        "hdr" => Kind::Hdr,
        "exr" => Kind::Exr,
        "pnm" => Kind::Pnm,
        "sgi" => Kind::Sgi,
        "ras" => Kind::Ras,
        "wbmp" => Kind::Wbmp,
        "dds" => Kind::Dds,
        "ktx" => Kind::Ktx,
        "ktx2" => Kind::Ktx2,
        "wal" => Kind::Wal,
        "svg" => Kind::Svg,
        "emf" => Kind::Emf,
        "raw" => Kind::Raw,
        "psd" | "psb" => Kind::Psd,
        "jp2" => Kind::Jp2,
        "xcf" => Kind::Xcf,
        "ora" => Kind::Ora,
        "ani" => Kind::Ani,
        "farbfeld" | "ff" => Kind::Farbfeld,
        "xbm" => Kind::Xbm,
        "avif" => Kind::Avif,
        "wmf" => Kind::Wmf,
        "mng" => Kind::Mng,
        "lottie" => Kind::Lottie,
        "tgs" => Kind::Tgs,
        "dxf" => Kind::Dxf,
        "cr3" => Kind::Cr3,
        "astc" => Kind::Astc,
        "basis" => Kind::Basis,
        "pvr" => Kind::Pvr,
        "atf" => Kind::Atf,
        "vtf" => Kind::Vtf,
        "blp" => Kind::Blp,
        "spr" => Kind::Spr,
        "txd" => Kind::Txd,
        "tex" => Kind::Tex,
        "cdr" => Kind::Cdr,
        other => Kind::Unsupported(
            catalog::find_id(other)
                .map(|spec| spec.id)
                .unwrap_or("desconhecido"),
        ),
    }
}

fn is_xbm_text(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(&head[..head.len().min(512)]);
    text.contains("#define")
        && (text.contains("_width") || text.contains("width"))
        && (text.contains("_bits") || text.contains("char") || text.contains("static"))
}

fn u16_be_ok(data: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(..2)?.try_into().ok()?))
}

fn is_tiff(head: &[u8]) -> bool {
    head.starts_with(b"II*\0") || head.starts_with(b"MM\0*")
}

fn is_emf(head: &[u8]) -> bool {
    u32_le(head, 0).ok() == Some(1) && u32_le(head, 40).ok() == Some(0x464D_4520)
}

fn is_pnm(head: &[u8]) -> bool {
    matches!(head, [b'P', b'1'..=b'6', b' ' | b'\n' | b'\r' | b'\t', ..])
}

fn is_svg_text(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(&head[..head.len().min(512)]).to_ascii_lowercase();
    let trimmed = text.trim_start();
    trimmed.starts_with("<svg")
        || trimmed.starts_with("<?xml") && trimmed.contains("<svg")
        || trimmed.starts_with("<!doctype svg")
}

fn looks_like_tga(head: &[u8]) -> bool {
    head.len() >= 18 && matches!(head[2], 1 | 2 | 3 | 9 | 10 | 11) && head[1] <= 1
}

fn tga_footer(tail: &[u8]) -> bool {
    tail.windows(16).any(|window| window == b"TRUEVISION-XFILE")
}

fn looks_like_pcx(head: &[u8]) -> bool {
    head.len() >= 16 && head[0] == 0x0A && head[1] <= 5 && head[2] <= 1
}

fn strict_pcx(head: &[u8]) -> bool {
    looks_like_pcx(head) && head[2] == 1 && matches!(head[3], 1 | 2 | 4 | 8)
}

fn looks_like_dib(head: &[u8]) -> bool {
    matches!(u32_le(head, 0).ok(), Some(40 | 108 | 124))
        && matches!(u16_le(head, 14).ok(), Some(1 | 8 | 16 | 24 | 32))
}

fn looks_like_wal(head: &[u8]) -> bool {
    let Ok(width) = u32_le(head, 32) else {
        return false;
    };
    let Ok(height) = u32_le(head, 36) else {
        return false;
    };
    width > 0 && height > 0 && width <= 4096 && height <= 4096
}

fn looks_like_dxf(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(&head[..head.len().min(128)]);
    let flat: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    flat.to_ascii_uppercase().starts_with("0SECTION")
}

fn looks_like_lottie(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(&head[..head.len().min(1024)]);
    text.contains("\"layers\"") && text.contains("\"v\"")
}
