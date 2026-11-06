//! SVG estático com resvg. Scripts, imagens externas e SMIL não rodam.

use crate::model::{check_dims, Built, Frame};

pub fn decode_svg(data: &[u8]) -> Result<Built, String> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(data, &options).map_err(|err| format!("SVG: {err}"))?;
    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;
    check_dims(width, height)?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "não foi possível alocar o SVG".to_owned())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let rgba = unpremultiply(pixmap.data());
    let mut built = Built {
        frames: vec![Frame {
            rgba,
            width,
            height,
            delay_ms: 0,
        }],
        bit_depth: 8,
        color: "RGBA".to_owned(),
        mipmaps: 1,
        notes: vec![
            "SVG animado é desenhado estático. Nenhum script ou arquivo externo é carregado."
                .to_owned(),
        ],
        id: "svg",
        loop_forever: false,
    };
    if data.starts_with(&[0x1F, 0x8B]) {
        built.notes.push("SVGZ descompactado só para desenhar".to_owned());
    }
    Ok(built)
}

fn unpremultiply(pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len());
    for px in pixels.chunks_exact(4) {
        let alpha = u32::from(px[3]);
        if alpha == 0 || alpha == 255 {
            out.extend_from_slice(px);
        } else {
            out.push(((u32::from(px[0]) * 255) / alpha) as u8);
            out.push(((u32::from(px[1]) * 255) / alpha) as u8);
            out.push(((u32::from(px[2]) * 255) / alpha) as u8);
            out.push(px[3]);
        }
    }
    out
}
