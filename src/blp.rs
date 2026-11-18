//! Blizzard BLP1 / BLP2 texture decoder.

use crate::model::{Built, Frame};

pub fn decode_blp_bytes(data: &[u8]) -> Result<Built, String> {
    let blp = image_blp::parser::load_blp_from_buf(data)
        .map_err(|e| format!("erro ao analisar BLP: {e:?}"))?;

    let dyn_img = image_blp::convert::blp_to_image(&blp, 0)
        .map_err(|e| format!("erro ao decodificar BLP mipmap 0: {e:?}"))?;

    let width = dyn_img.width();
    let height = dyn_img.height();
    let rgba = dyn_img.to_rgba8().into_raw();

    let version_str = match blp.header.version {
        image_blp::types::BlpVersion::Blp0 => "BLP0",
        image_blp::types::BlpVersion::Blp1 => "BLP1",
        image_blp::types::BlpVersion::Blp2 => "BLP2",
    };

    let mipmaps = blp.header.mipmaps_count();

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: mipmaps as u32,
        notes: vec![format!(
            "Blizzard {version_str}, {width}x{height}, {mipmaps} mipmaps"
        )],
        id: "blp",
        loop_forever: false,
    })
}
