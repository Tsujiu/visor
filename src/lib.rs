//! Visor: leitura, identificação e conversão de imagens.

mod ani;
mod assoc;
mod astc;
mod atf;
mod avif;
mod basis;
mod binio;
mod blp;
mod catalog;
mod cdr;
mod cr3;
mod dds;
mod decode;
mod dxf;
mod emf;
mod export;
mod farbfeld;
mod jp2;
mod jxl;
mod ktex;
mod ktx;
mod lottie;
mod magick;
mod meta;
mod mng;
mod model;
mod ora;
mod psd;
mod pvr;
mod raster;
mod rawimg;
mod simple;
mod sniff;
mod spr;
mod svg;
mod txd;
mod vtf;
mod xbm;
mod xcf;

pub use assoc::{AssocReport, current_png_progid, register_as_default};
pub use catalog::{
    FormatSpec, browsable_extension, external_extensions, formats, libraries, partial_extensions,
    verified_extensions,
};
pub use decode::{browsable, open_image, thumbnail};
pub use export::{ExportKind, export_image, export_many};
pub use magick::available as magick_available;
pub use model::{Category, Decoded, Frame, ImageInfo, Support};

#[cfg(test)]
mod tests;
