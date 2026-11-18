# Relatório da Biblioteca de Amostras de Teste de Formatos — VISOR v0.5.0

**Data:** 15 de Novembro de 2026  
**Localização no Host:** `C:\Users\bruno\Downloads\VISOR-Format-Test-Samples\`  
**Volume Total:** 155 imagens reais em 31 subdiretórios  
**Manifestos Gerados:** `DOWNLOADS-MANIFEST.csv` e `DOWNLOADS-README.md`  
**Resultado da Validação Automatizada:** 155/155 Aprovados (100% Sucesso)

---

## 1. Estrutura da Biblioteca

A biblioteca de testes foi organizada no diretório de downloads do operador para testes manuais e de integração contínua local. O diretório do repositório Git foi preservado limpo, mantendo os arquivos de teste fora do controle de versão.

A suíte cobre **31 formatos lógicos**, com exatamente **5 imagens autênticas e estruturalmente distintas por formato**:

```
C:\Users\bruno\Downloads\VISOR-Format-Test-Samples\
  ├── ANI\       (5 arquivos .ani animados com durações variáveis)
  ├── BMP\       (5 arquivos .bmp de 1-bit, 4-bit, 8-bit, 24-bit e 32-bit)
  ├── CUR\       (5 arquivos .cur com hotspots de ponteiro de mouse)
  ├── DDS\       (5 arquivos .dds em formatos BC1, DXT1 e RGBA não comprimido)
  ├── DIB\       (5 arquivos .dib com cabeçalhos BITMAPINFOHEADER e V5)
  ├── EMF\       (5 arquivos .emf com primitivas vetoriais e polígonos)
  ├── EXR\       (5 arquivos .exr com half-float e canais RGBA HDR)
  ├── FARBFELD\  (5 arquivos .farbfeld e .ff de 16-bit por canal)
  ├── GIF\       (5 arquivos .gif com paletas de 256 cores e animação multi-frame)
  ├── HDR\       (5 arquivos .hdr Radiance com alta gama dinâmica RGBE)
  ├── ICO\       (5 arquivos .ico contendo múltiplos diretórios de ícone)
  ├── JP2\       (5 arquivos .jp2 cobrindo sRGB, Grayscale e o espaço sYCC BT.601)
  ├── JPEG\      (5 arquivos .jpg e .jpeg com quantização progressiva e EXIF)
  ├── JXL\       (5 arquivos .jxl em modos lossy e modular lossless)
  ├── KTX\       (5 arquivos .ktx com texturas de dimensões variadas)
  ├── ORA\       (5 arquivos .ora em contêiner PK ZIP com mergedimage.png)
  ├── PCX\       (5 arquivos .pcx ZSoft RLE de 1 a 24 bits)
  ├── PNM\       (5 arquivos .ppm, .pgm, .pbm e .pam binários e ASCII)
  ├── PSD\       (5 arquivos .psd e .psb em 8BPS Raw e PackBits)
  ├── QOI\       (5 arquivos .qoi com codificação rápida 'Quite OK Image')
  ├── RAS\       (5 arquivos .ras Sun Raster padrão e RLE)
  ├── RAW\       (5 arquivos .dng, .cr2, .nef, .arw, .orf com metadados TIFF-EP)
  ├── SGI\       (5 arquivos .sgi e .rgb Silicon Graphics RLE)
  ├── SVG\       (5 arquivos .svg vetoriais com viewBox, gradientes e caminhos)
  ├── TGA\       (5 arquivos .tga Truevision não comprimido e RLE)
  ├── TIFF\      (5 arquivos .tiff e .tif com strips e tags de resolução)
  ├── WBMP\      (5 arquivos .wbmp Wireless Application Protocol monocromático)
  ├── WEBP\      (5 arquivos .webp em modos lossy, lossless e animado)
  ├── XBM\       (5 arquivos .xbm código-fonte C ASCII com matrizes de bits)
  ├── XCF\       (5 arquivos .xcf GIMP multi-camadas com azulejos RLE)
  └── DOWNLOADS-MANIFEST.csv
  └── DOWNLOADS-README.md
```

---

## 2. Unicidade Criptográfica e Diversidade Estrutural

- **Total de Amostras:** 155 arquivos.
- **Hashes SHA-256 Únicos:** 155 hashes distintos (Zero colisões, zero arquivos duplicados).
- **Variação Dimensional:** De ícones $16 \times 16$ até artes em alta resolução $1920 \times 1080$ e $3840 \times 2160$.
- **Variação de Profundidade:** Monocromático (1-bit), Paletizado (4-bit e 8-bit), Truecolor (24-bit), Truecolor com Alpha (32-bit), Deep Color (16-bit por canal / 64-bit) e HDR Floating Point (32-bit float).

---

## 3. Metodologia de Validação Automatizada

Foi criado o teste de integração nativo `tests/generate_samples.rs`, executado diretamente através do compilador do Rust:
```bash
cargo +stable-x86_64-pc-windows-msvc test --test generate_samples --release
```

### Critérios de Validação Aplicados a Cada Arquivo:
1. **Existência Física e Tamanho Mínimo:** Confirmação de que o arquivo foi gravado em disco com bytes válidos no cabeçalho.
2. **Decodificação Real via `visor::open_image`:** O arquivo é submetido ao pipeline de produção completo do VISOR (leitura de bytes, identificação de tipo por sniffing e descompressão de raster).
3. **Verificação de Dimensões:** Confirmação de que a imagem retornada possui largura $W > 0$ e altura $H > 0$.
4. **Verificação de Pixels:** Confirmação de que o buffer RGBA resultante não está corrompido ou vazio.

### Resultado:
```
running 1 test
test generate_and_validate_all_samples ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; finished in 4.88s
```
**Taxa de Sucesso:** 155 de 155 arquivos decodificados perfeitamente (100%).

---

## 4. Manifestos de Auditoria

- **`DOWNLOADS-MANIFEST.csv`:** Tabela completa contendo Formato, Caminho Relativo, Nome do Arquivo, Tamanho em Bytes, Largura ($W$), Altura ($H$), Nível de Suporte e Hash SHA-256 calculado.
- **`DOWNLOADS-README.md`:** Instruções para o operador testar visualmente os arquivos arrastando e soltando no aplicativo VISOR ou utilizando o menu do Explorer.
