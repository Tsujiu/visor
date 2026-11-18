//! Decodificador de desenhos CAD DXF (AutoCAD Drawing Exchange Format).
//! Converte entidades vetoriais para SVG e rasteriza em Rust puro via resvg.

use crate::model::Built;

pub fn decode_dxf_bytes(data: &[u8]) -> Result<Built, String> {
    let mut reader = std::io::Cursor::new(data);
    let drawing = dxf::Drawing::load(&mut reader)
        .map_err(|err| format!("Erro ao carregar entidades DXF: {err}"))?;

    let entities: Vec<&dxf::entities::Entity> = drawing.entities().collect();
    let svg_string = dxf_to_svg::dxf_to_svg(entities, None);
    if svg_string.trim().is_empty() {
        return Err("DXF não contém geometria vetorial visível".to_owned());
    }

    let mut built = crate::svg::decode_svg(svg_string.as_bytes())
        .map_err(|err| format!("Erro ao rasterizar SVG do DXF: {err}"))?;

    built.id = "dxf";
    built.notes = vec!["Desenho CAD DXF convertido e renderizado via resvg".to_owned()];
    Ok(built)
}
