//! Decodificador nativo de JPEG 2000 (JP2, J2K, J2C, JPX).
//!
//! Utiliza a biblioteca pure-Rust `jpeg2k` (baseada no port seguro `openjp2`)
//! para decodificar fluxos de dados e contêineres JPEG 2000 diretamente para RGBA8.

use std::path::Path;

use image::{DynamicImage, RgbImage};
use jpeg2k::Image;

use crate::model::{Master, check_dims};

pub fn decode_jp2_bytes(bytes: &[u8]) -> Result<Master, String> {
    if bytes.len() < 12 {
        return Err("JPEG 2000 inválido ou truncado".to_owned());
    }

    let jp2_image = Image::from_bytes(bytes).map_err(|err| format!("JPEG 2000: {err:?}"))?;
    let dyn_img: DynamicImage = match (&jp2_image).try_into() {
        Ok(img) => img,
        Err(_) => {
            // Fallback para sYCC ou componentes manuais
            let comps = jp2_image.components();
            if comps.len() >= 3 && !comps[0].data.is_empty() {
                let w = comps[0].w;
                let h = comps[0].h;
                check_dims(w, h)?;
                let total = w as usize * h as usize;
                let y_comp = &comps[0].data;
                let cb_comp = &comps[1].data;
                let cr_comp = &comps[2].data;

                let mut rgb = Vec::with_capacity(total * 3);
                for i in 0..total {
                    let y = y_comp.get(i).copied().unwrap_or(0) as f32;
                    let cb = (cb_comp.get(i).copied().unwrap_or(128) - 128) as f32;
                    let cr = (cr_comp.get(i).copied().unwrap_or(128) - 128) as f32;

                    let r = (y + 1.402 * cr).clamp(0.0, 255.0) as u8;
                    let g = (y - 0.344136 * cb - 0.714136 * cr).clamp(0.0, 255.0) as u8;
                    let b = (y + 1.772 * cb).clamp(0.0, 255.0) as u8;

                    rgb.push(r);
                    rgb.push(g);
                    rgb.push(b);
                }
                let rgb_buf = RgbImage::from_raw(w, h, rgb)
                    .ok_or_else(|| "JPEG 2000: buffer sYCC inválido".to_owned())?;
                DynamicImage::ImageRgb8(rgb_buf)
            } else {
                return Err("JPEG 2000: conversão de espaço de cor não suportada".to_owned());
            }
        }
    };

    let width = dyn_img.width();
    let height = dyn_img.height();
    check_dims(width, height)?;

    let (bit_depth, color) = crate::raster::describe_color(dyn_img.color());
    let note = format!(
        "JPEG 2000: decodificação nativa pure-Rust ({}×{}, {})",
        width, height, color
    );

    Ok(Master::Still {
        image: dyn_img,
        bit_depth,
        color,
        mipmaps: 1,
        notes: vec![note],
        id: "jp2",
    })
}

#[allow(dead_code)]
pub fn decode_jp2_path(path: &Path) -> Result<Master, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("não leu o arquivo JPEG 2000: {err}"))?;
    decode_jp2_bytes(&bytes)
}
