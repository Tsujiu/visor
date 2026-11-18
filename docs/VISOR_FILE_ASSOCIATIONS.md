# VISOR — Inventário Oficial de Associações de Arquivos Windows
## Fonte de Verdade: `src/catalog.rs`

Este documento estabelece o catálogo exato e auditado de extensões de arquivo reconhecidas pelo decodificador nativo do VISOR para fins de registro no sistema operacional Windows (10 e 11) através de ProgID e capacidades de aplicação (`Capabilities\FileAssociations` e `OpenWithProgids`).

---

## 1. Princípios de Associação

1. **Apenas Formatos Decodificados Nativamente:** Somente são registradas extensões cujos codecs estejam compilados no executável e confirmados por testes automatizados de decodificação.
2. **Exclusão de Formatos Externos:** Formatos que dependem de utilitários externos no PATH (como ImageMagick para PDF, EPS, AI) **não** são registrados no registro do Windows, evitando falhas silenciosas em máquinas onde tais ferramentas não estejam instaladas.
3. **Exclusão Estrita de Formatos Não Suportados:** Extensões sem suporte nativo comprovado não entram nas chaves de associação.
4. **Respeito às Escolhas do Usuário:** O VISOR registra-se unicamente como **candidato** no catálogo do Windows (`OpenWithProgids` e `Capabilities`). A definição como padrão ocorre exclusivamente por escolha explícita do usuário na interface oficial do Windows Settings.

---

## 2. Inventário de Extensões Registradas

### Grupo A: Formatos com Leitura Completa (`Support::Full`)

| Formato | Extensões | Biblioteca / Codec | Capacidade de Exportação | Observações |
| :--- | :--- | :--- | :--- | :--- |
| **PNG / APNG** | `.png`, `.apng` | `png` crate (Rust puro) | Sim (PNG 8/16-bit) | Suporte completo a canal alfa e animações APNG. |
| **JPEG** | `.jpg`, `.jpeg`, `.jpe`, `.jfif` | `zune-jpeg` | Sim (JPEG Q95) | Decodificador de alta performance tipo libjpeg-turbo. Conversão CMYK para RGB. |
| **GIF** | `.gif` | `image` (GIF) | Sim (GIF) | Animação com intervalos de quadro e sobreposição alfa. |
| **BMP** | `.bmp` | `image` (BMP) | Sim (BMP) | BMP padrão com cabeçalho de arquivo. |
| **TIFF** | `.tif`, `.tiff` | `tiff` crate | Sim (TIFF 8/16-bit) | Suporta 8 e 16 bits por canal, sem perda. |
| **ICO** | `.ico` | `image` (ICO) | Sim (ICO até 256px) | Ícone Windows com PNG ou BMP embutido. |
| **TGA** | `.tga` | `image` (TGA) | Sim (TGA) | Truevision TGA com rodapé de arquivo. |
| **QOI** | `.qoi` | `image` (QOI) | Sim (QOI) | Quite OK Image format com compressão sem perdas rápida. |
| **HDR** | `.hdr`, `.pic` | `image` (HDR) | Sim (HDR float) | Radiance RGBE para imagens de alta faixa dinâmica. |
| **PNM** | `.ppm`, `.pgm`, `.pbm`, `.pnm` | `image` (PNM) | Sim (PNM binário) | PBM (1-bit), PGM (escala de cinza), PPM (RGB). |

### Grupo B: Formatos de Leitura Verificada (`Support::ReadOnly`)

| Formato | Extensões | Biblioteca / Codec | Capacidade de Exportação | Observações |
| :--- | :--- | :--- | :--- | :--- |
| **DIB** | `.dib` | `image` (BMP) | Não | BITMAPINFOHEADER solto sem cabeçalho de arquivo. |
| **JPEG XL** | `.jxl` | `jxl-oxide` | Não | Decodificador puro Rust para JPEG XL. Perfil de cor preservado. |
| **CUR** | `.cur` | `image` (ICO) | Não | Cursor do Windows (tipo 2) com DIB ou PNG embutido. |
| **PCX** | `.pcx` | `image` (PCX) | Não | ZSoft PCX com RLE 8 bits paletizado ou 24 bits. |
| **OpenEXR** | `.exr` | `exr` crate | Não | Leitura RGB de ponto flutuante em Rust puro. |
| **SGI** | `.sgi`, `.rgb`, `.rgba`, `.bw` | Leitor interno | Não | Imagens Silicon Graphics uncompressed e RLE. |
| **WBMP** | `.wbmp` | Leitor interno | Não | Wireless Bitmap preto e branco (1-bit). |
| **KTX** | `.ktx` | Leitor interno | Não | Khronos Texture 1 RGBA8 com mipmaps. |
| **KTX2** | `.ktx2` | `ktx2` crate | Não | Contêiner KTX2 para texturas sem supercompressão. |
| **SVG / SVGZ** | `.svg`, `.svgz` | `resvg` / `usvg` | Não | Renderização vetorial segura sem scripts nem dependências externas. |
| **EMF** | `.emf` | `GDI+` (Win32) | Não | Enhanced Metafile do Windows rasterizado via GDI+. |

### Grupo C: Formatos com Suporte de Leitura Parcial (`Support::Partial`)

| Formato | Extensões | Biblioteca / Codec | Capacidade de Exportação | Observações |
| :--- | :--- | :--- | :--- | :--- |
| **WebP** | `.webp` | `image-webp` | Não | WebP estático com e sem perdas e transparência alfa. |
| **Sun Raster** | `.ras` | `image` | Não | Sun Raster standard 24/32 bits. |
| **DDS** | `.dds` | `ddsfile` + `texture2ddecoder` | Não | DirectDraw Surface: RGBA8, BC1 até BC7. Mipmap 0 exibido. |
| **Quake WAL** | `.wal` | Leitor interno | Não | Textura WAL com contagem de mips. |
| **Camera RAW** | `.raw`, `.dng`, `.cr2`, `.nef`, `.nrw`, `.arw`, `.srf`, `.sr2`, `.raf`, `.orf`, `.rw2`, `.pef`, `.3fr`, `.iiq`, `.erf`, `.kdc`, `.mos` | `rawloader` | Não | Prévia de sensores de câmeras fotográficas sem demosaic complexo. |

---

## 3. Formatos Não Registrados no Sistema

### Formatos Externos (`Support::External` — Requerem ImageMagick)
Estes formatos dependem do executável `magick.exe` presente no PATH. **Não são registrados no Windows** para evitar que o VISOR seja definido como manipulador de arquivos em máquinas sem ImageMagick:
- `.pdf`, `.eps`, `.epsf`, `.ps`, `.ai`, `.jp2`, `.j2k`, `.j2c`, `.jpx`, `.cdr`

### Formatos Não Suportados (`Support::Unsupported` — NUNCA Registrados)
Formatos sem decodificador implementado ou rejeitados na compilação:
- `.avif`, `.astc`, `.basis`, `.pvr`, `.atf`, `.vtf`, `.blp`, `.spr`, `.txd`, `.tex`, `.wmf`, `.dxf`, `.mng`, `.json`, `.lottie`, `.tgs`, `.cr3`

---

## 4. Totalizadores do Inventário

- **Total de Extensões Registradas no Windows:** 49 extensões.
  - *Full Support:* 17 extensões (`.png`, `.apng`, `.jpg`, `.jpeg`, `.jpe`, `.jfif`, `.gif`, `.bmp`, `.tif`, `.tiff`, `.ico`, `.tga`, `.qoi`, `.hdr`, `.pic`, `.ppm`, `.pgm`, `.pbm`, `.pnm`)
  - *ReadOnly:* 15 extensões (`.dib`, `.jxl`, `.cur`, `.pcx`, `.exr`, `.sgi`, `.rgb`, `.rgba`, `.bw`, `.wbmp`, `.ktx`, `.ktx2`, `.svg`, `.svgz`, `.emf`)
  - *Partial:* 21 extensões (`.webp`, `.ras`, `.dds`, `.wal`, `.raw`, `.dng`, `.cr2`, `.nef`, `.nrw`, `.arw`, `.srf`, `.sr2`, `.raf`, `.orf`, `.rw2`, `.pef`, `.3fr`, `.iiq`, `.erf`, `.kdc`, `.mos`)
- **ProgID Utilizado:** `Visor.Image`
- **Nome Registrado:** `VISOR`
- **Alvo:** `HKCU\Software\Classes\Visor.Image`, `HKCU\Software\Visor\Capabilities`, `HKCU\Software\RegisteredApplications`, `HKCU\Software\Classes\.ext\OpenWithProgids`.
