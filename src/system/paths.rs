//! Operações de caminhos e arquivos no sistema de arquivos local.

use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    a.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
}

pub fn local_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Visor")
}

pub fn kind_from_path(path: &Path) -> visor::ExportKind {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "jpe" | "jfif" => visor::ExportKind::Jpeg,
        "webp" => visor::ExportKind::Webp,
        "bmp" => visor::ExportKind::Bmp,
        "tif" | "tiff" => visor::ExportKind::Tiff,
        "gif" => visor::ExportKind::Gif,
        "qoi" => visor::ExportKind::Qoi,
        "tga" => visor::ExportKind::Tga,
        "ico" => visor::ExportKind::Ico,
        "ppm" | "pgm" | "pbm" | "pnm" => visor::ExportKind::Pnm,
        "hdr" => visor::ExportKind::Hdr,
        _ => visor::ExportKind::Png,
    }
}
