//! Decodificador de texturas Valve VTF (Valve Texture Format - Source Engine).
//! Decodificação de contêiner via source-vtf e blocos DXT/RGBA via texture2ddecoder.

use source_vtf::{ImageFormat, Vtf};

use crate::model::{Built, Frame, bgra_to_rgba, check_dims};

pub fn decode_vtf_bytes(data: &[u8]) -> Result<Built, String> {
    let vtf = Vtf::from_bytes(data).map_err(|err| format!("VTF inválido: {err:?}"))?;

    let base = vtf
        .base_image()
        .map_err(|err| format!("Falha ao extrair imagem base do VTF: {err:?}"))?;

    let width = base.width as u32;
    let height = base.height as u32;
    check_dims(width, height)?;

    let w = width as usize;
    let h = height as usize;
    let src = base.data;

    let (rgba, format_name) = match base.format {
        ImageFormat::Dxt1 | ImageFormat::Dxt1OneBitAlpha => {
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc1(src, w, h, &mut pixels)
                .map_err(|err| format!("VTF DXT1: {err}"))?;
            (bgra_to_rgba(&pixels), "DXT1")
        }
        ImageFormat::Dxt3 => {
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc2(src, w, h, &mut pixels)
                .map_err(|err| format!("VTF DXT3: {err}"))?;
            (bgra_to_rgba(&pixels), "DXT3")
        }
        ImageFormat::Dxt5 => {
            let mut pixels = vec![0u32; w * h];
            texture2ddecoder::decode_bc3(src, w, h, &mut pixels)
                .map_err(|err| format!("VTF DXT5: {err}"))?;
            (bgra_to_rgba(&pixels), "DXT5")
        }
        ImageFormat::Rgba8888 => {
            let need = w * h * 4;
            let bytes = src
                .get(..need)
                .ok_or_else(|| "VTF RGBA8888 truncado".to_owned())?;
            (bytes.to_vec(), "RGBA8888")
        }
        ImageFormat::Bgra8888 | ImageFormat::Bgrx8888 => {
            let need = w * h * 4;
            let bytes = src
                .get(..need)
                .ok_or_else(|| "VTF BGRA truncado".to_owned())?;
            let mut out = Vec::with_capacity(need);
            for chunk in bytes.as_chunks::<4>().0 {
                out.push(chunk[2]); // R
                out.push(chunk[1]); // G
                out.push(chunk[0]); // B
                out.push(chunk[3]); // A
            }
            (out, "BGRA8888")
        }
        ImageFormat::Rgb888 => {
            let need = w * h * 3;
            let bytes = src
                .get(..need)
                .ok_or_else(|| "VTF RGB888 truncado".to_owned())?;
            let mut out = Vec::with_capacity(w * h * 4);
            for chunk in bytes.as_chunks::<3>().0 {
                out.push(chunk[0]);
                out.push(chunk[1]);
                out.push(chunk[2]);
                out.push(255);
            }
            (out, "RGB888")
        }
        ImageFormat::Bgr888 => {
            let need = w * h * 3;
            let bytes = src
                .get(..need)
                .ok_or_else(|| "VTF BGR888 truncado".to_owned())?;
            let mut out = Vec::with_capacity(w * h * 4);
            for chunk in bytes.as_chunks::<3>().0 {
                out.push(chunk[2]);
                out.push(chunk[1]);
                out.push(chunk[0]);
                out.push(255);
            }
            (out, "BGR888")
        }
        other => return Err(format!("Formato de textura VTF ({other:?}) não suportado")),
    };

    Ok(Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: format!("RGBA ({format_name})"),
        mipmaps: 1,
        notes: vec![format!("Valve VTF {format_name} ({width}×{height})")],
        id: "vtf",
        loop_forever: false,
    })
}
