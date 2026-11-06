//! Abre um arquivo: valida tamanho, identifica pelo conteúdo e escolhe o codec.

use std::fs::File;
use std::io::Read;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use crate::catalog::{self, find_id};
use crate::meta::{self, jpeg_is_cmyk};
use crate::model::{
    check_dims, downscale, has_alpha, Built, Decoded, Frame, ImageInfo, Master, Support, MAX_FILE,
    MAX_SIDE,
};
use crate::sniff::{self, Kind};

pub fn open_image(path: &Path) -> Result<Decoded, String> {
    let opened = catch_unwind(AssertUnwindSafe(|| open_inner(path)));
    match opened {
        Ok(result) => result,
        Err(_) => Err("arquivo corrompido ou malformado; a leitura foi interrompida".to_owned()),
    }
}

pub fn thumbnail(path: &Path, max_side: u32) -> Result<Frame, String> {
    let decoded = open_image(path)?;
    let frame = decoded
        .frames
        .into_iter()
        .next()
        .ok_or_else(|| "imagem vazia".to_owned())?;
    Ok(downscale(frame, max_side.max(1)).0)
}

fn open_inner(path: &Path) -> Result<Decoded, String> {
    let (master, exif) = load_master(path)?;
    present(master, exif)
}

pub(crate) fn open_master(path: &Path) -> Result<Master, String> {
    Ok(load_master(path)?.0)
}

fn load_master(path: &Path) -> Result<(Master, Vec<(String, String)>), String> {
    let meta = std::fs::metadata(path).map_err(|_| "arquivo não encontrado".to_owned())?;
    if !meta.is_file() {
        return Err("isso não é um arquivo".to_owned());
    }
    if meta.len() > MAX_FILE {
        return Err("arquivo acima de 512 MB".to_owned());
    }
    if meta.len() == 0 {
        return Err("arquivo vazio".to_owned());
    }
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let (head, tail) = read_edges(path, meta.len())?;
    let kind = sniff::sniff(&head, &tail, ext.as_deref());
    let mut master = decode_kind(path, &head, kind)?;
    if jpeg_is_cmyk(&head) {
        notes_mut(&mut master).push(
            "JPEG CMYK detectado no marcador SOF; os pixels exibidos estão em RGB".to_owned(),
        );
    }
    if let Some(icc) = meta::read_icc_file(path) {
        notes_mut(&mut master).push(icc);
    }
    Ok((master, meta::read_exif(path)))
}

fn decode_kind(path: &Path, head: &[u8], kind: Kind) -> Result<Master, String> {
    let decoded = match kind {
        Kind::Png | Kind::Jpeg | Kind::Gif | Kind::Bmp | Kind::Tiff | Kind::Webp | Kind::Ico
        | Kind::Tga | Kind::Qoi | Kind::Pnm | Kind::Hdr | Kind::Exr => {
            crate::raster::decode_path(path, kind)
        }
        Kind::Svg => read_capped(path)
            .and_then(|bytes| crate::svg::decode_svg(&bytes))
            .map(Master::Frames),
        Kind::Jxl => crate::jxl::decode_jxl(path),
        Kind::Dds => read_capped(path)
            .and_then(|bytes| crate::dds::decode_dds(&bytes))
            .map(Master::Frames),
        Kind::Ktx => read_capped(path)
            .and_then(|bytes| crate::ktx::decode_ktx1(&bytes))
            .map(Master::Frames),
        Kind::Ktx2 => read_capped(path)
            .and_then(|bytes| crate::ktx::decode_ktx2(&bytes))
            .map(Master::Frames),
        Kind::Pcx => read_capped(path)
            .and_then(|bytes| crate::simple::decode_pcx(&bytes))
            .map(Master::Frames),
        Kind::Wbmp => read_capped(path)
            .and_then(|bytes| crate::simple::decode_wbmp(&bytes))
            .map(Master::Frames),
        Kind::Ras => read_capped(path)
            .and_then(|bytes| crate::simple::decode_ras(&bytes))
            .map(Master::Frames),
        Kind::Sgi => read_capped(path)
            .and_then(|bytes| crate::simple::decode_sgi(&bytes))
            .map(Master::Frames),
        Kind::Wal => read_capped(path)
            .and_then(|bytes| crate::simple::decode_wal(&bytes))
            .map(Master::Frames),
        Kind::Dib => read_capped(path)
            .and_then(|bytes| crate::simple::decode_dib(&bytes))
            .map(Master::Frames),
        Kind::Cur => read_capped(path)
            .and_then(|bytes| crate::simple::decode_cur(&bytes))
            .map(Master::Frames),
        Kind::Emf => read_capped(path)
            .and_then(|bytes| crate::emf::decode_emf(&bytes))
            .map(Master::Frames),
        Kind::Raw => crate::rawimg::decode_raw(path).or_else(|err| {
            if is_tiff(head) {
                crate::raster::decode_path(path, Kind::Tiff).map_err(|_| err)
            } else {
                Err(err)
            }
        }),
        Kind::Unsupported(id) => Err(unsupported_message(id)),
        Kind::Unknown => Err(
            "formato não reconhecido pelo conteúdo. A extensão sozinha não torna o arquivo suportado."
                .to_owned(),
        ),
    };
    match decoded {
        Ok(master) => Ok(wrap_bytes(master)),
        Err(err) => {
            if let Some(id) = external_id(kind) {
                if crate::magick::available() {
                    return crate::magick::rasterize(path, id).map(wrap_bytes);
                }
            }
            Err(err)
        }
    }
}

fn wrap_bytes(master: Master) -> Master {
    match master {
        Master::Frames(built) => Master::Frames(built),
        other => other,
    }
}

fn external_id(kind: Kind) -> Option<&'static str> {
    match kind {
        Kind::Unsupported(id)
            if matches!(id, "pdf" | "eps" | "ps" | "ai" | "jp2" | "cdr") =>
        {
            Some(id)
        }
        _ => None,
    }
}

fn unsupported_message(id: &'static str) -> String {
    let spec = find_id(id);
    match spec {
        Some(spec) => format!(
            "{} não suportado. {}",
            spec.name, spec.note
        ),
        None => format!("{id} não suportado"),
    }
}

fn notes_mut(master: &mut Master) -> &mut Vec<String> {
    match master {
        Master::Still { notes, .. } | Master::Frames(Built { notes, .. }) => notes,
    }
}

fn present(master: Master, exif: Vec<(String, String)>) -> Result<Decoded, String> {
    let (frames, bit_depth, color, mipmaps, mut notes, id, loop_forever) = match master {
        Master::Still {
            image,
            bit_depth,
            color,
            mipmaps,
            notes,
            id,
        } => {
            check_dims(image.width(), image.height())?;
            let width = image.width();
            let height = image.height();
            let rgba = image.to_rgba8().into_raw();
            (
                vec![Frame {
                    rgba,
                    width,
                    height,
                    delay_ms: 0,
                }],
                bit_depth,
                color,
                mipmaps,
                notes,
                id,
                false,
            )
        }
        Master::Frames(built) => (
            built.frames,
            built.bit_depth,
            built.color,
            built.mipmaps,
            built.notes,
            built.id,
            built.loop_forever,
        ),
    };
    if frames.is_empty() {
        return Err("a imagem não tem pixels".to_owned());
    }
    let original_w = frames[0].width;
    let original_h = frames[0].height;
    let mut reduced = false;
    let frames: Vec<Frame> = frames
        .into_iter()
        .map(|frame| {
            let (frame, did) = downscale(frame, MAX_SIDE);
            reduced |= did;
            frame
        })
        .collect();
    if reduced {
        notes.push(format!(
            "exibição reduzida para caber em {MAX_SIDE} px; a exportação relê o arquivo"
        ));
    }
    let spec = find_id(id);
    let alpha = has_alpha(&frames[0].rgba);
    Ok(Decoded {
        info: ImageInfo {
            id,
            name: spec.map(|spec| spec.name.to_owned()).unwrap_or_else(|| id.to_owned()),
            category: spec.map(|spec| spec.category).unwrap_or(crate::model::Category::Raster),
            support: spec.map(|spec| spec.support).unwrap_or(Support::Partial),
            bit_depth,
            color,
            original_w,
            original_h,
            display_w: frames[0].width,
            display_h: frames[0].height,
            reduced,
            has_alpha: alpha,
            frame_count: frames.len(),
            loop_forever,
            mipmaps,
            exif,
            icc: notes.iter().find(|note| note.contains("ICC")).cloned(),
            notes,
        },
        frames,
    })
}

fn read_edges(path: &Path, len: u64) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut file = File::open(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    let head_len = len.min(8192) as usize;
    let mut head = vec![0u8; head_len];
    file.read_exact(&mut head)
        .map_err(|_| "não leu o começo do arquivo".to_owned())?;
    if len <= 8192 {
        return Ok((head.clone(), head));
    }
    use std::io::{Seek, SeekFrom};
    let tail_at = len.saturating_sub(64);
    file.seek(SeekFrom::Start(tail_at))
        .map_err(|_| "não alcançou o fim do arquivo".to_owned())?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)
        .map_err(|_| "não leu o fim do arquivo".to_owned())?;
    Ok((head, tail))
}

fn read_capped(path: &Path) -> Result<Vec<u8>, String> {
    let meta = std::fs::metadata(path).map_err(|err| format!("não abriu o arquivo: {err}"))?;
    if meta.len() > MAX_FILE {
        return Err("arquivo acima de 512 MB".to_owned());
    }
    std::fs::read(path).map_err(|err| format!("não leu o arquivo: {err}"))
}

fn is_tiff(head: &[u8]) -> bool {
    head.starts_with(b"II*\0") || head.starts_with(b"MM\0*")
}

pub fn browsable(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(catalog::browsable_extension)
}
