# VISOR — Inventário Completo de Funcionalidades

**Data:** 06/11/2026  
**Versão Auditada:** 0.2.2  
**Status de Referência:** Verificado no código-fonte e em suíte de testes automatizados

---

## 1. Funcionalidades Existentes e Verificadas

| Categoria | Funcionalidade | Status Real no Código | Teste / Validação |
| :--- | :--- | :--- | :--- |
| **Decodificação** | Reconhecimento de formato por magic bytes | **Completo** (`src/sniff.rs`) | [`tests::content_wins_over_the_extension`](../src/tests.rs#L68) |
| **Decodificação** | Rejeição de arquivos truncados/corrompidos | **Completo** (`src/decode.rs`) | [`tests::truncated_png_is_rejected`](../src/tests.rs#L80) |
| **Decodificação** | Isolamento de panics em codecs (`catch_unwind`) | **Completo** (`src/decode.rs:17`) | Verificado em código |
| **Formatos Raster** | PNG e APNG com controle de quadros e delay | **Completo** (`src/raster.rs:146`) | [`tests::animations_keep_order_and_delay`](../src/tests.rs#L120) |
| **Formatos Raster** | JPEG com conversão de SOF CMYK para RGB | **Completo** (`src/meta.rs:9`) | [`tests::raster_color_alpha_depth_and_exif`](../src/tests.rs#L240) |
| **Formatos Raster** | GIF animado com sobreposição de quadros | **Completo** (`src/raster.rs:169`) | [`tests::animations_keep_order_and_delay`](../src/tests.rs#L120) |
| **Formatos Raster** | BMP, TIFF, WebP (estático), ICO, TGA, QOI, PNM | **Completo** (`src/raster.rs`) | [`tests::raster_color_alpha_depth_and_exif`](../src/tests.rs#L240) |
| **Formatos HDR** | OpenEXR e Radiance HDR (RGBE) | **Completo** (`src/raster.rs:85`, `exr`) | [`tests::exr_and_jxl_decode`](../src/tests.rs#L300) |
| **Formatos Vetor** | SVG e SVGZ sem scripts ou recursos externos | **Completo** (`src/svg.rs`) | [`tests::svg_keeps_alpha_and_svgz_opens`](../src/tests.rs#L150) |
| **Formatos Vetor** | EMF (Windows Enhanced Metafile via GDI+) | **Completo** (`src/emf.rs`) | [`tests::emf_draws_a_flat_color`](../src/tests.rs#L380) |
| **Texturas Games** | DDS (BC1 a BC7, RGBA8, mips) e KTX/KTX2 | **Completo** (`src/dds.rs`, `src/ktx.rs`) | [`tests::dds_rgba_and_bc1_and_ktx`](../src/tests.rs#L180) |
| **Formatos Legados**| PCX, WBMP, Sun RAS, SGI, Quake WAL, DIB, CUR | **Completo** (`src/simple.rs`) | [`tests::simple_formats_match_a_known_pixel`](../src/tests.rs#L210) |
| **Câmera RAW** | Prévia descompactada sem demosaic | **Parcial** (`src/rawimg.rs`) | Verificado em código |
| **Metadados** | Extração de tags EXIF (até 40 tags) | **Completo** (`src/meta.rs:58`) | [`tests::metadata_helpers_do_not_execute_profiles`](../src/tests.rs#L400) |
| **Metadados** | Detecção passiva de assinatura de perfil ICC | **Completo** (`src/meta.rs:36`) | [`tests::metadata_helpers_do_not_execute_profiles`](../src/tests.rs#L400) |
| **Exportação** | Exportação individual para 11 formatos | **Completo** (`src/export.rs`) | [`tests::batch_export_writes_each_file`](../src/tests.rs#L95) |
| **Exportação** | Preservação de 16 bits em PNG e TIFF | **Completo** (`src/export.rs:80`) | Verificado em código |
| **Exportação** | Conversão em lote para pasta de saída | **Completo** (`src/export.rs:97`) | [`tests::batch_export_writes_each_file`](../src/tests.rs#L95) |
| **Navegação** | Próxima / anterior (`←`, `→`, botões mouse) | **Completo** (`src/main.rs:301`) | [`tests::lists_images_case_insensitive`](../src/main.rs#L1530) |
| **Visualização** | Zoom proporcional ancorado no cursor | **Completo** (`src/main.rs:1244`)| [`tests::zoom_keeps_the_point_under_the_cursor`](../src/main.rs#L1545) |
| **Visualização** | Rotação 90° horário / anti-horário (`R`) | **Completo** (`src/main.rs:450`) | Verificado em código |
| **Visualização** | Panning da imagem com arrasto do mouse | **Completo** (`src/main.rs:1107`)| Verificado em código |
| **Visualização** | Alternância tela cheia (`F11` / `Esc`) | **Completo** (`src/main.rs:670`) | Verificado em código |
| **Visualização** | Drag and drop de imagens da área de trabalho | **Completo** (`src/main.rs:581`) | Verificado em código |
| **Windows** | Registro de associação em `HKCU` via `SFTA.ps1` | **Completo** (`src/assoc.rs`) | Verificado em código |

---

## 2. Funcionalidades Incompletas ou com Limitações

1. **Carrossel de Miniaturas (Filmstrip):**
   * *Limitação:* Fixado em apenas 7 itens (`index - 3 .. index + 4`), ignorando a largura da tela.
   * *Limitação:* Não possui opção para o usuário recolher/ocultar a barra para focar na imagem.
   * *Limitação:* Falta fila assíncrona com pool (spawna uma OS thread por miniatura).
2. **Modos de Interpolação:**
   * *Limitação:* Forçado em `TextureOptions::LINEAR` (filtro bilinear). Não há suporte a `Nearest Neighbor` para inspeção de pixel art, texturas de jogos ou ícones em zoom extremo.
3. **Plano de Fundo de Transparência (Checkerboard):**
   * *Limitação:* Padrão xadrez escuro fixo em 2x2 pixels. Não permite alternância para xadrez claro, preto ou branco puro, dificultando a inspeção de artes escuras transparentes.
4. **Painel de Formatos Suportados:**
   * *Limitação:* Janela flutuante estática com rolagem vertical de dezenas de itens. Falta filtro ou campo de busca instantânea por extensão.
5. **Painel de Metadados EXIF (`I`):**
   * *Limitação:* Painel lateral rígido de 300px que encolhe o canvas. Não agrupa os dados por categorias fotográficas (Câmera, Abertura, Velocidade, ISO) nem oferece botão para copiar dados com um clique.

---

## 3. Funcionalidades Ausentes (Gaps Críticos Identificados)

| Recurso Ausente | Impacto no Usuário | Prioridade |
| :--- | :--- | :--- |
| **Copiar Imagem para o Clipboard (`Ctrl+C`)** | Impossibilita copiar imagem visualizada para colar no Discord, WhatsApp, Word ou Photoshop. | **P0 (Crítica)** |
| **Excluir para a Lixeira (`Delete`)** | Obriga o usuário a alternar para o Windows Explorer para deletar fotos descartadas durante a triagem. | **P0 (Crítica)** |
| **Menu de Contexto (Botão Direito)** | Usuário fica sem atalhos diretos na imagem (Copiar, Revelar no Explorer, Propriedades, Excluir). | **P1 (Alta)** |
| **Revelar no Windows Explorer** | Usuário não tem como abrir a pasta nativa do arquivo atual diretamente pelo aplicativo. | **P1 (Alta)** |
| **Configuração de Comportamento da Roda do Mouse** | Impossibilita escolher entre "Roda = Zoom" e "Roda = Próxima Imagem". | **P1 (Alta)** |
| **Thread Pool e Cancelamento Atômico** | CPU sobrecarregada em navegações rápidas por threads sem cancelamento. | **P1 (Alta)** |
| **Cache LRU de Imagens Decodificadas** | Recarregamento completo do disco a cada foto revisitada na mesma sessão. | **P1 (Alta)** |
| **Banner Amigável de Associação de Arquivos** | Substituição do comportamento invasivo de popup de configurações por banner discreto. | **P1 (Alta)** |
| **Toolbar Flutuante Estilo Windows 11 (Cápsula)** | Substituição da barra rígida cinza por interface limpa e moderna. | **P1 (Alta)** |

---

## 4. Matriz de Formatos e Nível de Suporte

### 4.1 Formatos de Imagem e Textura Suportados Diretamente (Sem Dependências Externas)

| Formato | Categoria | Extensões | Leitura | Gravação | Observações |
| :--- | :--- | :--- | :---: | :---: | :--- |
| **PNG / APNG** | Raster / Animação | `.png`, `.apng` | Sim | Sim | Alpha preservado; 16 bits na exportação; animação completa. |
| **JPEG** | Raster | `.jpg`, `.jpeg`, `.jfif` | Sim | Sim | `zune-jpeg`. Converte SOF CMYK para RGB. Exportação q=95. |
| **GIF** | Animação | `.gif` | Sim | Sim | Animação com duração por quadro e alpha. |
| **BMP** | Raster | `.bmp` | Sim | Sim | 24 e 32 bits com cabeçalho de arquivo. |
| **TIFF** | Raster | `.tif`, `.tiff` | Sim | Sim | Crate `tiff`. Preserva 16 bits de origem. |
| **WebP** | Raster | `.webp` | Sim | Sim | Estático sem perdas (lossless). WebP animado com leitura. |
| **JPEG XL** | Raster | `.jxl` | Sim | Não | `jxl-oxide`. Leitura com identificação de ICC. |
| **OpenEXR** | Raster | `.exr` | Sim | Não | Leitura RGB de ponto flutuante via crate `exr`. |
| **Radiance HDR**| Raster | `.hdr`, `.pic` | Sim | Sim | RGBE. Exibição 8 bits, gravação float. |
| **ICO / CUR** | Raster | `.ico`, `.cur` | Sim | Sim (ICO) | Suporte a ICO até 256px. CUR como somente leitura. |
| **TGA** | Raster | `.tga` | Sim | Sim | Truevision TGA com rodapé estendido. |
| **QOI** | Raster | `.qoi` | Sim | Sim | Quite OK Image. |
| **PNM** | Raster | `.ppm`, `.pgm`, `.pbm`, `.pnm` | Sim | Sim | Binário PBM, PGM, PPM. |
| **DIB** | Raster | `.dib` | Sim | Não | BITMAPINFOHEADER avulso. |
| **PCX** | Raster | `.pcx` | Sim | Não | RLE 8 bits e 24 bits planares. |
| **Sun RAS** | Raster | `.ras` | Sim | Não | Tipo standard 24 e 32 bits. |
| **SGI** | Raster | `.sgi`, `.rgb`, `.rgba`, `.bw` | Sim | Não | Não comprimido e RLE. |
| **WBMP** | Raster | `.wbmp` | Sim | Não | Monocromático Wireless Bitmap. |
| **Quake WAL** | Textura | `.wal` | Sim | Não | Textura com contagem de mips em cinza. |
| **DDS** | Textura | `.dds` | Sim | Não | DXT1 a DXT5, BC1 a BC7, RGBA8, mips. |
| **KTX / KTX2** | Textura | `.ktx`, `.ktx2` | Sim | Não | RGBA8 sem supercompressão. |
| **SVG / SVGZ** | Vetorial | `.svg`, `.svgz` | Sim | Não | `resvg`. Sem scripts e sem conexões externas. |
| **EMF** | Vetorial | `.emf` | Sim | Não | Desenho via Windows GDI+ sem código ativo. |
| **Camera RAW** | RAW | `.raw`, `.dng`, `.cr2`, `.nef`, `.arw`, etc. | Parcial | Não | Prévia descompactada sem demosaic via `rawloader`. |

### 4.2 Formatos com Conversor Externo Opcional (Requer ImageMagick no `PATH`)
* **PDF, EPS, PS, AI, JP2, CDR:** Rasterizados via comando `magick` isolado sem execução de JavaScript ou scripts de documento.

---

## 5. Mapeamento de Atalhos de Teclado (Atual vs. Planejado)

| Tecla / Atalho | Comportamento Atual | Comportamento Planejado (Modernizado) |
| :--- | :--- | :--- |
| `Ctrl + O` | Abrir diálogo de arquivo | Mantido (diálogo assíncrono não-bloqueante) |
| `Ctrl + S` | Exportar imagem atual | Mantido |
| `Ctrl + C` | **Nenhum (Inexistente)** | **Copiar imagem para o Clipboard do Windows (Novo)** |
| `Delete` | **Nenhum (Inexistente)** | **Mover imagem atual para a Lixeira do Windows (Novo)** |
| `←` / `→` | Imagem anterior / próxima | Mantido com transição fluida e cancelamento |
| `Home` / `End` | Primeira / última imagem da pasta | Mantido |
| `F` | Ajustar imagem à janela | Mantido |
| `1` ou `Num0` | Tamanho real (1:1 / 100%) | Mantido |
| `+` / `−` | Zoom in / Zoom out | Mantido com zoom suave ancorado no cursor |
| `R` / `Shift + R` | Girar 90° horário / anti-horário | Mantido com preview imediato |
| `I` | Exibir painel de metadados | Exibir novo Drawer lateral com cards modernos |
| `T` | **Nenhum** | **Alternar exibição do Filmstrip / Carrossel (Novo)** |
| `P` | **Nenhum** | **Alternar modo de interpolação Linear / Pixel Art (Novo)** |
| `B` | **Nenhum** | **Alternar padrão de fundo (Xadrez escuro/claro/preto/branco) (Novo)** |
| `Espaço` | Pausar / reproduzir animação | Mantido |
| `F11` / `Esc` | Entrar / sair de tela cheia | Mantido |
| `Mouse Dir.` | **Nenhum** | **Menu de contexto com ações rápidas (Novo)** |
