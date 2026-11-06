//! Conversor externo. O arquivo não é interpretado aqui; o magick só rasteriza.

use std::path::Path;
use std::process::Command;

use crate::model::Master;
use crate::raster::read_memory_image;
use image::ImageFormat;

pub fn available() -> bool {
    Command::new("magick")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn rasterize(path: &Path, id: &'static str) -> Result<Master, String> {
    if !available() {
        return Err(format!(
            "{id} só abre com o ImageMagick (comando magick no PATH). Nesta máquina ele não está disponível."
        ));
    }
    let output = Command::new("magick")
        .arg("-limit")
        .arg("memory")
        .arg("256MiB")
        .arg("-limit")
        .arg("width")
        .arg("16000")
        .arg("-limit")
        .arg("height")
        .arg("16000")
        .arg(format!("{}[0]", path.display()))
        .arg("png:-")
        .output()
        .map_err(|err| format!("não executou o magick: {err}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.trim();
        return Err(format!(
            "o ImageMagick não converteu este arquivo. {}",
            if detail.is_empty() { "sem detalhe" } else { detail }
        ));
    }
    let image = read_memory_image(&output.stdout, ImageFormat::Png)?;
    Ok(Master::Still {
        image,
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: 1,
        notes: vec!["rasterizado pelo ImageMagick; conteúdo ativo não é executado".to_owned()],
        id,
    })
}

