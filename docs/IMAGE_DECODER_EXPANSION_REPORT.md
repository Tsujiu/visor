# Relatório de Expansão de Decodificadores Nativos — VISOR v0.5.0

**Data:** 15 de Novembro de 2026  
**Responsável:** Engenharia de Sistemas e Codecs do VISOR  
**Classificação:** Técnico / Arquitetural

---

## 1. Contexto e Objetivos

Para a versão v0.5.0 comercial de edição única, o VISOR expandiu sua suíte de decodificação interna para cobrir os principais formatos profissionais de criação, edição gráfica, formatos científicos, de arquivo e cursores do Windows.

A restrição primordial foi:
1. **Nenhum vínculo a bibliotecas dinâmicas externas** (C/C++ runtime DLLs).
2. **Decodificação 100% autossuficiente e em Rust puro**.
3. **Resiliência a arquivos corrompidos ou maliciosos**.
4. **Alinhamento completo com o pipeline `open_image` e sistema de cache LRU do VISOR**.

---

## 2. Novos Decodificadores Implementados

### 2.1. Adobe Photoshop (`src/psd.rs` — `.psd`, `.psb`)
- **Assinatura Binária:** `8BPS` (`[0x38, 0x42, 0x50, 0x53]`).
- **Suporte de Versão:** Versão 1 (PSD regular) e Versão 2 (PSB — *Large Document Format* de até 300.000 x 300.000 pixels).
- **Modos de Cor Suportados:**
  - Grayscale (8-bit e 16-bit).
  - RGB (8-bit e 16-bit com canal alfa opcional).
  - CMYK (conversão fotométrica precisa $RGB = (1-C)(1-K)$, etc.).
  - Indexed Color (lookup table de 256 entradas).
- **Algoritmo de Descompressão:**
  - Raw descompactado (Tipo 0).
  - PackBits RLE (Tipo 1) com tabela de varredura pré-calculada por canal e linha de varredura (*scanline*).
- **Resolução de Imagem Composta:** O decodificador extrai o plano compósito achatado (*flattened composite*) gerado pelo Photoshop, garantindo compatibilidade instantânea com qualquer arquivo de design sem exigir a execução completa de blend modes em tempo real.

### 2.2. JPEG 2000 (`src/jp2.rs` — `.jp2`, `.j2k`, `.j2c`, `.jpx`)
- **Padrão:** ISO/IEC 15444-1.
- **Motor:** `openjp2` / `jpeg2k` nativo em Rust puro.
- **Contêineres de Caixa JP2:**
  - `jP  ` (Assinatura JP2 `0x0000000C6A502020`).
  - `ftyp` (Compatibilidade de marca).
  - `jp2h` e `ihdr` (Cabeçalho de dimensões e canais).
  - `colr` (Espaço de cor sRGB, Grayscale e sYCC).
  - `jp2c` (Codestream primário de wavelets discretas).
- **Suporte Avançado ao Espaço sYCC (YCbCr):**
  - O crate base `openjp2` rejeita nativamente codestreams sYCC em sua rotina `TryFrom<&Image> for DynamicImage`.
  - O VISOR implementou uma matriz de conversão vetorial direta no espaço BT.601:
    $$R = Y + 1.402 \times (Cr - 128)$$
    $$G = Y - 0.344136 \times (Cb - 128) - 0.714136 \times (Cr - 128)$$
    $$B = Y + 1.772 \times (Cb - 128)$$
  - Essa conversão viabiliza a abertura de amostras de conformidade ISO (como `file2.jp2`) sem falhas.

### 2.3. GIMP XCF (`src/xcf.rs` — `.xcf`)
- **Assinatura:** `gimp xcf ` (`[0x67, 0x69, 0x6D, 0x70, 0x20, 0x78, 0x63, 0x66]`).
- **Navegação de Estrutura:**
  - Varredura da lista de ponteiros de propriedades de canvas.
  - Varredura de ponteiros de camadas (*layer offsets*).
  - Decodificação de hierarchy e níveis de tiles (64x64 pixels).
  - Descompressão RLE GIMP por azulejo (*tile*).
- **Composição de Camadas:**
  - Processamento de baixo para cima (*bottom-up*) das camadas visíveis com canal alfa e opacidade.
  - Blending Porter-Duff com interpolação de canal transparente.

### 2.4. OpenRaster (`src/ora.rs` — `.ora`)
- **Assinatura:** Arquivo PK ZIP contendo tipo MIME `image/openraster`.
- **Estratégia de Decodificação:**
  - Leitura do arquivo ZIP via `zip::ZipArchive`.
  - Inspeção e busca por `mergedimage.png`.
  - Fallback estrutural: caso `mergedimage.png` não esteja presente, localização do primeiro arquivo raster válido em `data/` referenciado pelo `stack.xml`.
  - Decodificação do buffer PNG resultante via pipeline de alta performance do VISOR.

### 2.5. Animated Cursor (`src/ani.rs` — `.ani`)
- **Assinatura:** RIFF com identificador de tipo `ACON` (`[0x41, 0x43, 0x4F, 0x4E]`).
- **Processamento de Chunks RIFF:**
  - `anih`: Cabeçalho com contagem de frames, passos, dimensões, bits por pixel e taxa base em jiffies (1 jiffy = 1/60s).
  - `rate`: Tabela de durações customizadas por frame.
  - `seq `: Sequência de exibição dos frames.
  - `LIST/fram`: Frames individuais encapsulados no formato `icon` / `cur`.
- **Integração de Animação:**
  - Retorna uma estrutura `AnimationFrame` com durações temporais milimétricas exatas para reprodução contínua no canvas interativo do VISOR.

### 2.6. Farbfeld (`src/farbfeld.rs` — `.farbfeld`, `.ff`)
- **Assinatura:** `farbfeld` (`[0x66, 0x61, 0x72, 0x62, 0x66, 0x65, 0x6C, 0x64]`).
- **Especificação:** Formato gráfico sem perdas criado pelo projeto suckless.
  - Cabeçalho de 16 bytes: 8 bytes mágicos + 4 bytes largura (Big-Endian u32) + 4 bytes altura (Big-Endian u32).
  - Pixels: 16-bit Big-Endian RGBA (8 bytes por pixel).
- **Decodificação:** Conversão de alta velocidade de 16-bit para 8-bit RGBA com clamp de segurança e mapeamento direto para `RgbaImage`.

### 2.7. X11 Bitmap (`src/xbm.rs` — `.xbm`)
- **Estrutura:** Código-fonte C ASCII representando um array de bytes hexadecimais:
  - Diretivas `#define <name>_width <W>` e `#define <name>_height <H>`.
  - Declaração de array `static unsigned char <name>_bits[] = { 0x.., 0x.. };`.
- **Parser de Tokens:**
  - Parser léxico em Rust puro sem regex.
  - Extração de largura e altura.
  - Tokenização de literais hexadecimais (`0x..` ou decimais).
  - Expansão de bits: No formato XBM, o bit menos significativo (LSB) corresponde ao pixel mais à esquerda. O parser expande cada bit para pixel preto (0) ou branco (1), gerando imagem RGBA utilizável diretamente na viewport.

---

## 3. Integração com a Arquitetura Geral

1. **`src/sniff.rs`:** Todos os magic bytes correspondentes foram adicionados na enumeração `Kind`, permitindo identificação instantânea mesmo que o arquivo tenha extensão alterada.
2. **`src/decode.rs`:** O roteador `open_image` e `decode_path` despacha de forma assíncrona para as novas funções modulares.
3. **`src/catalog.rs`:** Inclusão de todas as extensões e tipos MIME correspondentes para navegação por diretório (filmstrip) e associação com o Explorer.
4. **`src/lib.rs`:** Exportação pública dos novos decodificadores e registros em `SUPPORTED_FORMATS`.

---

## 4. Conclusão

A adição desses 7 novos motores eleva o VISOR para um patamar de compatibilidade de nível profissional, mantendo integridade estática, alta velocidade de inicialização e ausência de qualquer dependência externa C-Runtime no Windows 11.
