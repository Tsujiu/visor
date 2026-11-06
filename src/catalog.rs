//! Catálogo honesto. Uma extensão só entra no filtro de abertura se o nível
//! correspondente foi exercitado por um teste de decodificação.

use crate::model::{Category, Support};

#[derive(Clone, Copy, Debug)]
pub struct FormatSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub extensions: &'static [&'static str],
    pub support: Support,
    pub note: &'static str,
}

const FORMATS: &[FormatSpec] = &[
    FormatSpec {
        id: "png",
        name: "PNG / APNG",
        category: Category::Raster,
        extensions: &["png", "apng"],
        support: Support::Full,
        note: "libpng via crate png. Alfa, 16 bits na exportação PNG/TIFF e APNG com duração dos quadros.",
    },
    FormatSpec {
        id: "jpeg",
        name: "JPEG",
        category: Category::Raster,
        extensions: &["jpg", "jpeg", "jpe", "jfif"],
        support: Support::Full,
        note: "zune-jpeg (classe libjpeg-turbo). CMYK, quando o marcador SOF tem 4 componentes, vira RGB. Qualidade 95 na exportação.",
    },
    FormatSpec {
        id: "gif",
        name: "GIF",
        category: Category::Animation,
        extensions: &["gif"],
        support: Support::Full,
        note: "Animação com duração e ordem. Descarte de quadro é sobreposição com alfa.",
    },
    FormatSpec {
        id: "bmp",
        name: "BMP",
        category: Category::Raster,
        extensions: &["bmp"],
        support: Support::Full,
        note: "BMP com cabeçalho de arquivo.",
    },
    FormatSpec {
        id: "dib",
        name: "DIB",
        category: Category::Raster,
        extensions: &["dib"],
        support: Support::ReadOnly,
        note: "BITMAPINFOHEADER solto, 24 e 32 bits, BI_RGB.",
    },
    FormatSpec {
        id: "tiff",
        name: "TIFF",
        category: Category::Raster,
        extensions: &["tif", "tiff"],
        support: Support::Full,
        note: "crate tiff. A exportação TIFF preserva 16 bits quando a origem é 16 bits.",
    },
    FormatSpec {
        id: "webp",
        name: "WebP",
        category: Category::Raster,
        extensions: &["webp"],
        support: Support::Partial,
        note: "image-webp: estático com alfa, leitura e escrita sem perdas. WebP animado não tem vetor de teste nesta build.",
    },
    FormatSpec {
        id: "avif",
        name: "AVIF",
        category: Category::Raster,
        extensions: &["avif"],
        support: Support::Unsupported,
        note: "libavif/dav1d não entra nesta build GNU portátil. Estático e animado ficam de fora.",
    },
    FormatSpec {
        id: "jxl",
        name: "JPEG XL",
        category: Category::Raster,
        extensions: &["jxl"],
        support: Support::ReadOnly,
        note: "jxl-oxide. Sem codificador. Perfil de cor é informado e não aplicado.",
    },
    FormatSpec {
        id: "ico",
        name: "ICO",
        category: Category::Raster,
        extensions: &["ico"],
        support: Support::Full,
        note: "Ícone com BMP ou PNG embutido. A exportação ICO exige até 256 px.",
    },
    FormatSpec {
        id: "cur",
        name: "CUR",
        category: Category::Raster,
        extensions: &["cur"],
        support: Support::ReadOnly,
        note: "Cursor tipo 2 com PNG ou DIB embutido. Não é tratado como ICO.",
    },
    FormatSpec {
        id: "tga",
        name: "TGA",
        category: Category::Raster,
        extensions: &["tga"],
        support: Support::Full,
        note: "Truevision TGA, inclusive com rodapé TRUEVISION-XFILE.",
    },
    FormatSpec {
        id: "pcx",
        name: "PCX",
        category: Category::Raster,
        extensions: &["pcx"],
        support: Support::ReadOnly,
        note: "RLE de 8 bits com paleta e 24 bits planares.",
    },
    FormatSpec {
        id: "qoi",
        name: "QOI",
        category: Category::Raster,
        extensions: &["qoi"],
        support: Support::Full,
        note: "Quite OK Image.",
    },
    FormatSpec {
        id: "hdr",
        name: "HDR",
        category: Category::Raster,
        extensions: &["hdr", "pic"],
        support: Support::Full,
        note: "Radiance RGBE. A exibição é 8 bits; a exportação HDR guarda ponto flutuante.",
    },
    FormatSpec {
        id: "exr",
        name: "OpenEXR",
        category: Category::Raster,
        extensions: &["exr"],
        support: Support::ReadOnly,
        note: "crate exr, não a biblioteca C OpenEXR. Leitura RGB. Sem gravador nesta build.",
    },
    FormatSpec {
        id: "jp2",
        name: "JPEG 2000",
        category: Category::Raster,
        extensions: &["jp2", "j2k", "j2c", "jpx"],
        support: Support::External,
        note: "Sem OpenJPEG embutido. Rasteriza se o comando magick estiver no PATH.",
    },
    FormatSpec {
        id: "pnm",
        name: "PNM",
        category: Category::Raster,
        extensions: &["ppm", "pgm", "pbm", "pnm"],
        support: Support::Full,
        note: "PBM, PGM e PPM binários.",
    },
    FormatSpec {
        id: "sgi",
        name: "SGI",
        category: Category::Raster,
        extensions: &["sgi", "rgb", "rgba", "bw"],
        support: Support::ReadOnly,
        note: "SGI sem compressão, com teste de cor. O RLE está no leitor, sem vetor automático. Origem no canto inferior esquerdo.",
    },
    FormatSpec {
        id: "ras",
        name: "Sun RAS",
        category: Category::Raster,
        extensions: &["ras"],
        support: Support::Partial,
        note: "Sun Raster tipo standard, 24 e 32 bits. RLE não é lido.",
    },
    FormatSpec {
        id: "wbmp",
        name: "WBMP",
        category: Category::Raster,
        extensions: &["wbmp"],
        support: Support::ReadOnly,
        note: "Wireless Bitmap. 0 é branco e 1 é preto.",
    },
    FormatSpec {
        id: "dds",
        name: "DDS",
        category: Category::Texture,
        extensions: &["dds"],
        support: Support::Partial,
        note: "ddsfile + texture2ddecoder. RGBA8 e BC1 verificados por cor. BC2, BC3, BC4, BC5, BC6H e BC7 usam o mesmo decodificador, sem vetor de cor próprio. Mip 0 é exibido; a contagem de mips aparece nos dados técnicos.",
    },
    FormatSpec {
        id: "ktx",
        name: "KTX",
        category: Category::Texture,
        extensions: &["ktx"],
        support: Support::ReadOnly,
        note: "KTX 1 não comprimido, RGBA8, com mipmaps.",
    },
    FormatSpec {
        id: "ktx2",
        name: "KTX2",
        category: Category::Texture,
        extensions: &["ktx2"],
        support: Support::ReadOnly,
        note: "Parser ktx2. RGBA8 sem supercompressão. BasisLZ/UASTC não são transcodificados.",
    },
    FormatSpec {
        id: "astc",
        name: "ASTC",
        category: Category::Texture,
        extensions: &["astc"],
        support: Support::Unsupported,
        note: "texture2ddecoder decodifica blocos ASTC, mas não há um contêiner .astc com cor verificada. A extensão não entra no filtro.",
    },
    FormatSpec {
        id: "basis",
        name: "Basis",
        category: Category::Texture,
        extensions: &["basis"],
        support: Support::Unsupported,
        note: "Basis Universal não é transcodificado.",
    },
    FormatSpec {
        id: "pvr",
        name: "PVR",
        category: Category::Texture,
        extensions: &["pvr"],
        support: Support::Unsupported,
        note: "Contêiner PowerVR sem decodificador próprio. PVRTC no meio de um DDS não é assumido.",
    },
    FormatSpec {
        id: "atf",
        name: "ATF",
        category: Category::Texture,
        extensions: &["atf"],
        support: Support::Unsupported,
        note: "Adobe ATF, estrutura própria, não implementada.",
    },
    FormatSpec {
        id: "vtf",
        name: "VTF",
        category: Category::Texture,
        extensions: &["vtf"],
        support: Support::Unsupported,
        note: "Valve Texture Format reconhecido pelo magic VTF, sem decodificador.",
    },
    FormatSpec {
        id: "blp",
        name: "BLP",
        category: Category::Texture,
        extensions: &["blp"],
        support: Support::Unsupported,
        note: "Blizzard BLP1/BLP2 reconhecido pelo magic, sem decodificador.",
    },
    FormatSpec {
        id: "wal",
        name: "WAL",
        category: Category::Texture,
        extensions: &["wal"],
        support: Support::Partial,
        note: "Quake WAL. O arquivo não traz paleta; os índices aparecem em cinza. Mips são contados.",
    },
    FormatSpec {
        id: "spr",
        name: "SPR",
        category: Category::Texture,
        extensions: &["spr"],
        support: Support::Unsupported,
        note: "A extensão SPR é de vários jogos e não compartilha uma estrutura. Nenhum contêiner é assumido.",
    },
    FormatSpec {
        id: "txd",
        name: "TXD",
        category: Category::Texture,
        extensions: &["txd"],
        support: Support::Unsupported,
        note: "RenderWare TXD e outros TXD não são o mesmo arquivo. Sem decodificador.",
    },
    FormatSpec {
        id: "tex",
        name: "TEX",
        category: Category::Texture,
        extensions: &["tex"],
        support: Support::Unsupported,
        note: "TEX é ambíguo (vários motores). Sem um magic conhecido, o arquivo é recusado.",
    },
    FormatSpec {
        id: "svg",
        name: "SVG / SVGZ",
        category: Category::Vector,
        extensions: &["svg", "svgz"],
        support: Support::ReadOnly,
        note: "resvg/usvg. Sem scripts, sem imagens externas e sem SMIL: SVG animado sai estático. Alfa é desmultiplicado.",
    },
    FormatSpec {
        id: "emf",
        name: "EMF",
        category: Category::Vector,
        extensions: &["emf"],
        support: Support::ReadOnly,
        note: "Metafile desenhado pelo GDI+ do Windows, sem executar conteúdo ativo.",
    },
    FormatSpec {
        id: "wmf",
        name: "WMF",
        category: Category::Vector,
        extensions: &["wmf"],
        support: Support::Unsupported,
        note: "WMF não tem vetor de teste. EMF, que é outro contêiner, passa pelo GDI+.",
    },
    FormatSpec {
        id: "pdf",
        name: "PDF",
        category: Category::Vector,
        extensions: &["pdf"],
        support: Support::External,
        note: "Sem motor PDF embutido e sem JavaScript. A primeira página rasteriza se magick estiver no PATH.",
    },
    FormatSpec {
        id: "eps",
        name: "EPS",
        category: Category::Vector,
        extensions: &["eps", "epsf"],
        support: Support::External,
        note: "PostScript encapsulado via ImageMagick, se o comando magick existir.",
    },
    FormatSpec {
        id: "ps",
        name: "PostScript",
        category: Category::Vector,
        extensions: &["ps"],
        support: Support::External,
        note: "PostScript via ImageMagick. O arquivo não é interpretado por este programa.",
    },
    FormatSpec {
        id: "ai",
        name: "Adobe Illustrator",
        category: Category::Vector,
        extensions: &["ai"],
        support: Support::External,
        note: "AI costuma ser PDF ou PostScript. Só abre por conversor externo.",
    },
    FormatSpec {
        id: "dxf",
        name: "DXF",
        category: Category::Vector,
        extensions: &["dxf"],
        support: Support::Unsupported,
        note: "Desenho CAD, não um raster. Não é executado nem desenhado.",
    },
    FormatSpec {
        id: "cdr",
        name: "CorelDRAW",
        category: Category::Vector,
        extensions: &["cdr"],
        support: Support::External,
        note: "RIFF CDR. Só por conversor externo, quando o magick conhece o arquivo.",
    },
    FormatSpec {
        id: "mng",
        name: "MNG",
        category: Category::Animation,
        extensions: &["mng"],
        support: Support::Unsupported,
        note: "Multiple-image Network Graphics, sem decodificador.",
    },
    FormatSpec {
        id: "lottie",
        name: "Lottie",
        category: Category::Animation,
        extensions: &["json", "lottie"],
        support: Support::Unsupported,
        note: "JSON Lottie não é executado. A extensão .json genérica também não entra no filtro.",
    },
    FormatSpec {
        id: "tgs",
        name: "TGS",
        category: Category::Animation,
        extensions: &["tgs"],
        support: Support::Unsupported,
        note: "Lottie compactado do Telegram. O JSON não é executado.",
    },
    FormatSpec {
        id: "raw",
        name: "RAW de câmera",
        category: Category::Raw,
        extensions: &[
            "raw", "dng", "cr2", "nef", "nrw", "arw", "srf", "sr2", "raf", "orf", "rw2", "pef",
            "3fr", "iiq", "erf", "kdc", "mos",
        ],
        support: Support::Partial,
        note: "rawloader, no lugar de LibRaw. Sem amostra de câmera neste ambiente: a abertura é tentada e a prévia é cinza ou RGB cru, sem demosaic. Não é suporte completo.",
    },
    FormatSpec {
        id: "cr3",
        name: "Canon CR3",
        category: Category::Raw,
        extensions: &["cr3"],
        support: Support::Unsupported,
        note: "CR3 é HEIF/CRX. O rawloader não lê esse contêiner.",
    },
];

pub fn formats() -> &'static [FormatSpec] {
    FORMATS
}

pub fn find_id(id: &str) -> Option<&'static FormatSpec> {
    FORMATS.iter().find(|spec| spec.id == id)
}

pub fn find_extension(ext: &str) -> Option<&'static FormatSpec> {
    let ext = ext.trim_start_matches('.').to_ascii_lowercase();
    FORMATS
        .iter()
        .find(|spec| spec.extensions.iter().any(|candidate| *candidate == ext))
}

pub fn extensions_with(levels: &[Support]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for spec in FORMATS {
        if levels.contains(&spec.support) && spec.id != "lottie" {
            for ext in spec.extensions {
                if !out.contains(ext) {
                    out.push(*ext);
                }
            }
        }
    }
    out
}

pub fn verified_extensions() -> Vec<&'static str> {
    extensions_with(&[Support::Full, Support::ReadOnly])
}

pub fn partial_extensions() -> Vec<&'static str> {
    extensions_with(&[Support::Partial])
}

pub fn external_extensions() -> Vec<&'static str> {
    extensions_with(&[Support::External])
}

pub fn browsable_extension(ext: &str) -> bool {
    find_extension(ext).is_some_and(|spec| {
        matches!(
            spec.support,
            Support::Full | Support::ReadOnly | Support::Partial
        )
    })
}

/// Bibliotecas realmente ligadas, e o que ficou de fora de propósito.
pub fn libraries() -> &'static [(&'static str, &'static str)] {
    &[
        ("png / image", "PNG, APNG, BMP, TIFF, GIF, QOI, TGA, PNM, ICO, HDR"),
        ("zune-jpeg", "JPEG, no lugar de libjpeg-turbo"),
        ("image-webp", "WebP estático, no lugar de libwebp C"),
        ("exr", "OpenEXR em Rust, no lugar da biblioteca C"),
        ("jxl-oxide", "JPEG XL, no lugar de libjxl"),
        ("resvg / usvg", "SVG e SVGZ"),
        ("ddsfile + texture2ddecoder", "DDS e BC, no lugar de DirectXTex"),
        ("ktx2", "contêiner KTX2; KTX 1 tem leitor próprio"),
        ("rawloader", "prévia RAW, no lugar de LibRaw"),
        ("kamadak-exif", "EXIF em JPEG, TIFF, PNG e WebP"),
        ("GDI+", "EMF no Windows"),
        ("ImageMagick (opcional, no PATH)", "PDF, EPS, PS, AI, JP2, CDR"),
    ]
}
