//! Visor: leitura, identificação e conversão de imagens.

mod assoc;
mod binio;
mod catalog;
mod dds;
mod decode;
mod emf;
mod export;
mod jxl;
mod ktx;
mod magick;
mod meta;
mod model;
mod raster;
mod rawimg;
mod simple;
mod sniff;
mod svg;

pub use assoc::{current_png_progid, register_as_default, AssocReport};
pub use catalog::{
    browsable_extension, external_extensions, formats, libraries, partial_extensions,
    verified_extensions, FormatSpec,
};
pub use decode::{browsable, open_image, thumbnail};
pub use export::{export_image, export_many, ExportKind};
pub use magick::available as magick_available;
pub use model::{Category, Decoded, Frame, ImageInfo, Support};

#[cfg(test)]
mod tests;
