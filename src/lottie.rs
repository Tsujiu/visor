//! Decodificador de animações vetoriais Lottie (.json, .lottie) e Telegram TGS (.tgs).
//! Renderização em Rust puro via rasterlottie com suporte a descompressão GZIP.

use flate2::read::GzDecoder;
use rasterlottie::{Animation, RenderConfig, Renderer, SupportProfile};
use std::io::Read;

use crate::model::{Built, Frame, MAX_FRAMES, check_dims};

pub fn decode_lottie_bytes(data: &[u8]) -> Result<Built, String> {
    let animation = if data.starts_with(&[0x1F, 0x8B]) {
        // Telegram Sticker: GZIP contendo Lottie JSON
        let mut decoder = GzDecoder::new(data);
        let mut json_str = String::new();
        decoder
            .read_to_string(&mut json_str)
            .map_err(|err| format!("Falha ao descompactar TGS gzip: {err}"))?;
        Animation::from_json_str(&json_str)
            .map_err(|err| format!("Lottie JSON inválido em TGS: {err}"))?
    } else if data.starts_with(b"PK\x03\x04") {
        // Arquivo .lottie (contêiner ZIP dotlottie)
        Animation::from_dotlottie_bytes(data)
            .map_err(|err| format!("Contêiner .lottie inválido: {err}"))?
    } else {
        // Lottie JSON padrão
        let json_str =
            std::str::from_utf8(data).map_err(|_| "Lottie JSON não é UTF-8 válido".to_owned())?;
        Animation::from_json_str(json_str).map_err(|err| format!("Lottie JSON inválido: {err}"))?
    };

    let w = animation.width.max(1);
    let h = animation.height.max(1);
    check_dims(w, h)?;

    let renderer = Renderer::new(SupportProfile::default());
    let config = RenderConfig::default();

    let fps = if animation.frame_rate > 0.0 {
        animation.frame_rate
    } else {
        30.0
    };
    let delay_ms = (1000.0 / fps).round().max(16.0) as u32;

    let in_frame = animation.in_point.round() as i32;
    let out_frame = animation.out_point.round() as i32;
    let total_frames = (out_frame - in_frame).max(1) as usize;

    let frames_to_render = total_frames.min(MAX_FRAMES);
    let mut frames = Vec::with_capacity(frames_to_render);

    for i in 0..frames_to_render {
        let frame_num = in_frame as f32 + i as f32;
        let raster = renderer
            .render_frame(&animation, frame_num, config)
            .map_err(|err| format!("Falha ao rasterizar quadro Lottie {i}: {err}"))?;

        frames.push(Frame {
            rgba: raster.pixels,
            width: raster.width,
            height: raster.height,
            delay_ms,
        });
    }

    if frames.is_empty() {
        return Err("Nenhum quadro rasterizado do arquivo Lottie".to_owned());
    }

    let is_animated = frames.len() > 1;
    let count = frames.len();

    Ok(Built {
        frames,
        bit_depth: 8,
        color: "RGBA (Lottie)".to_owned(),
        mipmaps: 1,
        notes: vec![format!(
            "Lottie / TGS ({w}×{h}), {count} quadro(s), {fps:.1} fps"
        )],
        id: "lottie",
        loop_forever: is_animated,
    })
}
