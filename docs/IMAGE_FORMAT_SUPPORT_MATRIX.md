# Matriz de Suporte a Formatos de Imagem — VISOR v0.5.0

**Versão do Documento:** 1.0  
**Data:** 15 de Novembro de 2026  
**Status:** Oficial / Homologado  
**Alvo:** Windows 10/11 x64 (MSVC Static CRT)

---

## 1. Visão Geral da Arquitetura de Codecs

O **VISOR v0.5.0** adota uma política rigorosa de decodificação de mídia:
- **100% Rust Nativo / Memory-Safe**: Todos os decodificadores operam em código Rust com garantias do borrow checker contra estouro de buffer (*buffer overflow*), uso após liberação (*use-after-free*) e corrupção de memória.
- **MSVC Static CRT (`/MT`)**: Binário autossuficiente sem dependências dinâmicas de C-Runtime (`vcruntime140.dll`, `msvcp140.dll`, `api-ms-win-crt-*`).
- **Zero DLLs Externas**: Depende exclusivamente de 19 DLLs essenciais do subsistema nativo do Windows (`ntdll.dll`, `KERNEL32.dll`, `USER32.dll`, `GDI32.dll`, `SHELL32.dll`, etc.).
- **Detecção Híbrida Inteligente**: Sniffing de magic bytes (assinatura binária de cabeçalho) com fallback para extensão de arquivo, prevenindo extensões mascaradas.

---

## 2. Matriz Completa de Formatos Suportados (31 Formatos / 70+ Extensões)

| # | Formato | Extensões | Nível de Suporte | Mecanismo / Codec | Rust Puro | CRT Estático | Animação / Camadas |
|---|---------|-----------|------------------|-------------------|:---------:|:------------:|:------------------:|
| 1 | **PNG** | `.png`, `.apng` | Leitura / Escrita | `image-png` (Rust) | Sim | Sim | APNG animado |
| 2 | **JPEG** | `.jpg`, `.jpeg`, `.jpe`, `.jfif` | Leitura / Escrita | `image-jpeg` (Rust) | Sim | Sim | EXIF / Orientação |
| 3 | **GIF** | `.gif` | Leitura / Escrita | `image-gif` (Rust) | Sim | Sim | Animado (Delays) |
| 4 | **BMP / DIB** | `.bmp`, `.dib` | Leitura / Escrita | `image-bmp` (Rust) | Sim | Sim | DIB V3/V4/V5 |
| 5 | **TIFF** | `.tiff`, `.tif` | Leitura / Escrita | `image-tiff` (Rust) | Sim | Sim | Multi-página |
| 6 | **WebP** | `.webp` | Leitura / Escrita | `image-webp` (Rust) | Sim | Sim | Lossy, Lossless, Animação |
| 7 | **ICO** | `.ico` | Leitura / Escrita | `image-ico` (Rust) | Sim | Sim | Multi-resolução |
| 8 | **CUR** | `.cur` | Leitura / Escrita | `image-ico` / `ani.rs` | Sim | Sim | Hotspot de cursor |
| 9 | **ANI** | `.ani` | Somente Leitura | `src/ani.rs` (Nativo) | Sim | Sim | RIFF/ACON animado |
| 10 | **JPEG XL** | `.jxl` | Somente Leitura | `jxl-oxide` (Rust) | Sim | Sim | HDR / Lossless |
| 11 | **HDR** | `.hdr` | Somente Leitura | `image-hdr` (Rust) | Sim | Sim | Radiance RGBE |
| 12 | **OpenEXR** | `.exr` | Somente Leitura | `exr` crate (Rust) | Sim | Sim | Half-float / Deep |
| 13 | **QOI** | `.qoi` | Leitura / Escrita | `qoi` crate (Rust) | Sim | Sim | Lossless rápido |
| 14 | **TGA** | `.tga`, `.icb`, `.vda`, `.vst` | Leitura / Escrita | `image-tga` (Rust) | Sim | Sim | RLE / Raw |
| 15 | **PSD** | `.psd` | Somente Leitura | `src/psd.rs` (Nativo) | Sim | Sim | Compósito 8BPS |
| 16 | **PSB** | `.psb` | Somente Leitura | `src/psd.rs` (Nativo) | Sim | Sim | Large Document Format |
| 17 | **JPEG 2000** | `.jp2`, `.j2k`, `.j2c`, `.jpx` | Somente Leitura | `src/jp2.rs` / `openjp2` | Sim | Sim | sRGB, Gray, sYCC |
| 18 | **GIMP XCF** | `.xcf` | Somente Leitura | `src/xcf.rs` (Nativo) | Sim | Sim | Compósito Multi-camadas |
| 19 | **OpenRaster** | `.ora` | Somente Leitura | `src/ora.rs` (Nativo) | Sim | Sim | ZIP / Compósito ORA |
| 20 | **Farbfeld** | `.farbfeld`, `.ff` | Somente Leitura | `src/farbfeld.rs` (Nativo) | Sim | Sim | 16-bit BE RGBA |
| 21 | **XBM** | `.xbm` | Somente Leitura | `src/xbm.rs` (Nativo) | Sim | Sim | C ASCII Monochrome |
| 22 | **Netpbm** | `.pnm`, `.pbm`, `.pgm`, `.ppm`, `.pam` | Leitura / Escrita | `image-pnm` (Rust) | Sim | Sim | Binário e ASCII |
| 23 | **PCX / DCX** | `.pcx`, `.dcx` | Somente Leitura | `pcx` crate (Rust) | Sim | Sim | RLE ZSoft |
| 24 | **SGI** | `.sgi`, `.rgb`, `.rgba`, `.bw` | Somente Leitura | `src/raster.rs` (Nativo) | Sim | Sim | Silicon Graphics |
| 25 | **Sun Raster** | `.ras` | Somente Leitura | `src/raster.rs` (Nativo) | Sim | Sim | Sun Microsystems |
| 26 | **Wireless BMP** | `.wbmp` | Somente Leitura | `src/raster.rs` (Nativo) | Sim | Sim | 1-bit WAP |
| 27 | **DirectDraw Surface** | `.dds` | Somente Leitura | `image-dds` (Rust) | Sim | Sim | BC1..BC7, RGBA, DXT |
| 28 | **Khronos Texture** | `.ktx`, `.ktx2` | Somente Leitura | `ktx2` / Native (Rust) | Sim | Sim | GPU Mipmaps |
| 29 | **Quake WAL** | `.wal` | Somente Leitura | `src/raster.rs` (Nativo) | Sim | Sim | Paletizado Mipmap |
| 30 | **Vetor SVG** | `.svg`, `.svgz` | Somente Leitura | `resvg` / `usvg` (Rust) | Sim | Sim | Rasterização vetorial |
| 31 | **Metafile EMF** | `.emf` | Somente Leitura | `src/emf.rs` / GDI+ | Sim | Sim | Windows Enhanced Metafile |
| 32 | **Camera RAW** | `.dng`, `.cr2`, `.nef`, `.arw`, `.orf`, `.rw2`, `.pef`, `.raf`, `.srw`, `.crw`, `.kdc`, `.dcr`, `.mrw`, `.nrw`, `.raw`, `.x3f`, `.3fr`, `.iiq`, `.erf`, `.mos`, `.bay`, `.cap` | Somente Leitura | `rawloader` + TIFF-EP | Sim | Sim | Decodificação DNG/TIFF + Metadados |

---

## 3. Resumo Estatístico

- **Formatos Lógicos Reconhecidos:** 32 categorias
- **Extensões de Arquivo Catalogadas:** 72 extensões
- **Formatos com Suporte a Gravação/Exportação:** 9 (PNG, JPEG, GIF, BMP, TIFF, WebP, ICO, QOI, PNM)
- **Formatos com Suporte a Animação Completa:** 4 (GIF, APNG, WebP animado, ANI animado)
- **Conformidade de Segurança:** 100% Rust Seguro, bounds checking de dimensões (limite máximo de 65.536 px), verificação de overflow aritmético em alocações de buffers.
- **Independência de Runtime:** 0 chamadas a bibliotecas C dinâmicas (`msvcrt`, `ucrtbase` dinâmico). Binário estático compilado via MSVC x64.
