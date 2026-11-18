//! Decodificador e transcodificador de texturas supercompactadas Basis Universal (.basis).
//! Transcodificação em tempo real para RGBA32 via transcoder Binomial Basis Universal.

use basis_universal::transcoding::{
    DecodeFlags, TranscodeParameters, Transcoder, TranscoderTextureFormat, transcoder_init,
};

use crate::model::{Built, Frame, check_dims};

pub fn decode_basis_bytes(data: &[u8]) -> Result<Built, String> {
    if data.len() < 16 {
        return Err("Arquivo Basis truncado".to_owned());
    }

    transcoder_init();

    let mut transcoder = Transcoder::new();
    if !transcoder.validate_header(data) {
        return Err("Cabeçalho Basis Universal inválido".to_owned());
    }

    if transcoder.prepare_transcoding(data).is_err() {
        return Err("Falha ao preparar transcodificação do Basis Universal".to_owned());
    }

    let description = transcoder
        .image_level_description(data, 0, 0)
        .ok_or_else(|| "Nível de mipmap 0 não encontrado no arquivo Basis".to_owned())?;

    let width = description.original_width;
    let height = description.original_height;
    check_dims(width, height)?;

    let params = TranscodeParameters {
        image_index: 0,
        level_index: 0,
        decode_flags: Some(DecodeFlags::empty()),
        output_row_pitch_in_blocks_or_pixels: None,
        output_rows_in_pixels: None,
    };

    let rgba = transcoder
        .transcode_image_level(data, TranscoderTextureFormat::RGBA32, params)
        .map_err(|err| format!("Falha na transcodificação Basis para RGBA32: {err:?}"))?;

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA (Basis)".to_owned(),
        mipmaps: 1,
        notes: vec![format!(
            "Basis Universal transcodificado ({width}×{height})"
        )],
        id: "basis",
        loop_forever: false,
    })
}
