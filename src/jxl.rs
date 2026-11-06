//! JPEG XL via jxl-oxide. Não há codificador e o ICC não é aplicado.

use std::fs::File;
use std::path::Path;

use image::ImageDecoder as _;

use crate::model::Master;
use crate::raster::{describe_color, limits_of};

pub fn decode_jxl(path: &Path) -> Result<Master, String> {
    let file = File::open(path).map_err(|err| format!("não abriu o JXL: {err}"))?;
    let mut decoder = jxl_oxide::integration::JxlDecoder::new(file)
        .map_err(|err| format!("JPEG XL: {err}"))?;
    decoder
        .set_limits(limits_of())
        .map_err(|err| format!("JPEG XL grande demais: {err}"))?;
    let (bit_depth, color) = describe_color(decoder.color_type());
    let mut notes = vec!["leitura apenas; este programa não grava JPEG XL".to_owned()];
    if let Ok(Some(profile)) = decoder.icc_profile() {
        notes.push(format!(
            "perfil ICC do JPEG XL ({} bytes); não aplicado",
            profile.len()
        ));
    }
    let image = image::DynamicImage::from_decoder(decoder)
        .map_err(|err| format!("não decodificou o JPEG XL: {err}"))?;
    Ok(Master::Still {
        image,
        bit_depth,
        color,
        mipmaps: 1,
        notes,
        id: "jxl",
    })
}
