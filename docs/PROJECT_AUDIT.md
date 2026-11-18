# VISOR — Auditoria Técnica Completa do Projeto

**Data:** 06/11/2026  
**Versão Auditada:** 0.2.2 (commit `01e0590`)  
**Toolchain:** `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`, `stable-x86_64-pc-windows-gnu`  
**Escopo:** Análise estática, arquitetural, de concorrência, de interface (UI/UX), segurança e conformidade de dependências.

---

## 1. Visão Geral da Base de Código

O Visor é um visualizador e conversor de imagens para Windows 10 e 11, projetado sob a premissa de execução 100% local, privacidade estrita (zero sockets/telemetria) e codecs escritos em Rust puro (evitando vulnerabilidades clássicas de bibliotecas C legadas).

### 1.1 Inventário de Arquivos e Linhas de Código

| Componente / Arquivo | Linhas | Papel Principal |
| :--- | :--- | :--- |
| [`src/main.rs`](../src/main.rs) | 1.568 | Ponto de entrada executável, estado `Visor`, loop de eventos `egui`, I/O assíncrono ad-hoc, interface gráfica completa. |
| [`src/lib.rs`](../src/lib.rs) | 32 | Fachada pública da biblioteca reexportando codecs, modelos e catálogo. |
| [`src/model.rs`](../src/model.rs) | 187 | Tipos de domínio (`Decoded`, `Frame`, `ImageInfo`, `Support`, `Category`, `Master`, limites). |
| [`src/decode.rs`](../src/decode.rs) | 296 | Despachante de decodificação protegido por `catch_unwind`, leitura de edges (magic bytes) e extração básica de metadados. |
| [`src/sniff.rs`](../src/sniff.rs) | 287 | Identificador de formatos baseado em magic numbers nos primeiros e últimos bytes do arquivo. |
| [`src/catalog.rs`](../src/catalog.rs) | 472 | Tabela estática de formatos suportados, categorias, extensões e notas de validação. |
| [`src/raster.rs`](../src/raster.rs) | 285 | Decodificação de formatos comuns e animações via crate `image` (PNG, APNG, GIF, WebP, JPEG, etc.). |
| [`src/simple.rs`](../src/simple.rs) | 477 | Codecs implementados à mão em Rust para formatos legados (PCX, WBMP, Sun RAS, SGI/RGB, Quake WAL, DIB, CUR, ICO). |
| [`src/export.rs`](../src/export.rs) | 207 | Motor de gravação e exportação individual e em lote (11 formatos suportados). |
| [`src/dds.rs`](../src/dds.rs) | 132 | Texturas DirectDraw (BC1 a BC7 via `ddsfile` e `texture2ddecoder`). |
| [`src/ktx.rs`](../src/ktx.rs) | 128 | Texturas Khronos (KTX1 manual e KTX2 via crate `ktx2`). |
| [`src/svg.rs`](../src/svg.rs) | 57 | Renderização de SVG e SVGZ estáticos via `resvg`/`tiny-skia` (scripts desativados). |
| [`src/emf.rs`](../src/emf.rs) | 155 | Enhanced Metafile do Windows via GDI+ (`windows-sys`). |
| [`src/rawimg.rs`](../src/rawimg.rs) | 84 | Prévia não interpolada de RAWs de câmera via `rawloader`. |
| [`src/jxl.rs`](../src/jxl.rs) | 36 | Decodificação de JPEG XL via `jxl-oxide`. |
| [`src/magick.rs`](../src/magick.rs) | 57 | Fallback opcional externo para PDF, EPS, PS, AI, JP2 e CDR via CLI `magick`. |
| [`src/meta.rs`](../src/meta.rs) | 91 | Extração passiva de EXIF (`kamadak-exif`) e detecção de assinatura de perfil ICC. |
| [`src/assoc.rs`](../src/assoc.rs) | 177 | Registro por usuário em `HKCU` e chamada a `SFTA.ps1` para `UserChoiceLatest`. |
| [`src/binio.rs`](../src/binio.rs) | 28 | Utilitários de leitura binária com checagem de limites (`slice::get`). |
| [`src/tests.rs`](../src/tests.rs) | 668 | 13 testes de integração e unitários cobrindo decodificadores e propriedades. |
| [`build.rs`](../build.rs) | 76 | Invocação de `windres` para embutir ícone e manifesto de versão Windows. |
| [`Cargo.toml`](../Cargo.toml) | 53 | Configuração de dependências e profile release. |
| [`installer/visor.iss`](../installer/visor.iss) | 68 | Script Inno Setup para instalação em `%LOCALAPPDATA%\Programs\Visor`. |
| [`third_party/SFTA.ps1`](../third_party/SFTA.ps1) | 1.870 | Script PowerShell para geração de hash de associação no Windows 10/11 (MIT). |

---

## 2. Auditoria Detalhada de Dependências (`Cargo.toml`)

### 2.1 Análise das Crates Diretas
* **`eframe` (`0.37`):** Framework GUI baseado em egui com backend Glow (OpenGL). Recursos ativados: `clipboard`, `default_fonts`, `glow`. Atende às metas de portabilidade e renderização rápida, mas a versão do egui não é explorada em sua capacidade visual completa.
* **`image` (`0.25`):** Crates em Rust puro configuradas explicitamente sem dependências C (`default-features = false`).
* **`rfd` (`0.17`):** Rust File Dialogs nativos do Windows. Usado de forma síncrona, causando congelamentos momentâneos na thread da UI.
* **`resvg` (`0.48`):** Renderizador SVG de alta conformidade e livre de execução de scripts maliciosos.
* **`jxl-oxide` (`0.12`):** Decodificador JPEG XL puro em Rust.
* **`kamadak-exif` (`0.6`):** Leitor de metadados EXIF passivo.
* **`ddsfile` (`0.5`) + `texture2ddecoder` (`0.1`):** Decodificação de texturas sem dependência de DirectXTex C++.
* **`ktx2` (`0.4`):** Leitura de contêineres KTX2.
* **`rawloader` (`0.37`):** Leitura de RAW sem dependência de LibRaw C.
* **`winreg` (`0.55`):** Manipulação de chaves de registro do Windows.
* **`windows-sys` (`0.61`):** Bindings diretas da API Win32 (`GdiPlus`, `UI_Shell`, `Foundation`).

---

## 3. Problemas Técnicos Identificados

Abaixo está o registro formal e verificado de cada vulnerabilidade, gargalo, limitação arquitetural e defeito de usabilidade presente no código atual.

---

### Problema 1: Monólito Arquitetural no Módulo da Interface (`src/main.rs`)
* **Arquivo e Localização:** [`src/main.rs:1-1568`](../src/main.rs#L1-L1568)
* **Problema Identificado:** O struct `Visor` funciona como um "God Object" contendo 31 campos. Concentra gerenciamento de estado da aplicação, eventos de teclado, polling de canais IPC/mpsc, renderização de toda a árvore de widgets, matemática de zoom/pan, formatação de títulos, geração de miniaturas, comandos do shell do Windows e manipulação de registro.
* **Impacto Técnico:** Alto acoplamento, impossibilidade de testar componentes visuais isoladamente, manutenibilidade degradada e alto risco de regressões a cada nova funcionalidade.
* **Gravidade:** **ALTA**
* **Solução Recomendada:** Modularizar a aplicação em camadas claras conforme arquitetura modular proposta em `docs/ARCHITECTURE.md`: camada de apresentação (`src/ui/`), gerenciamento de estado (`src/app/`), lógica de negócio e carregamento (`src/core/`), integração de SO (`src/system/`) e despachante assíncrono (`src/workers/`).
* **Dependências:** Nenhuma dependência externa; puramente refatoração estrutural em Rust.
* **Validação:** Compilação com `cargo check`, preservação dos testes de integração existentes em `src/main.rs:1512-1568`.

---

### Problema 2: Criação Descontrolada de Threads do Sistema Operacional (Sem Thread Pool)
* **Arquivo e Localização:** [`src/main.rs:369-385`](../src/main.rs#L369-L385), [`src/main.rs:510-517`](../src/main.rs#L510-L517), [`src/main.rs:555-570`](../src/main.rs#L555-L570), [`src/main.rs:1004-1021`](../src/main.rs#L1004-L1021)
* **Problema Identificado:** Toda operação assíncrona (carregamento da imagem principal, exportação individual, lote e, criticamente, cada miniatura do carrossel) invoca `std::thread::Builder::new().spawn(...)` ou `std::thread::spawn(...)`.
* **Impacto Técnico:**
  1. Ao navegar por uma pasta com muitas imagens, dezenas de threads do Windows são criadas e destruídas continuamente, gerando overhead de alocação de stack de thread no kernel do Windows.
  2. Não há controle de saturação de CPU/IO nem limite máximo de tarefas simultâneas.
* **Gravidade:** **ALTA**
* **Solução Recomendada:** Implementar um worker thread pool dedicado e dimensionado (ex: baseado em `num_cpus` ou pool fixo de workers em background com canal de mensagens e fila prioritária).
* **Dependências:** Crates leves de concorrência ou implementação interna baseada em workers `std::sync::mpsc`.
* **Validação:** Monitoramento de contagem de threads no Windows Task Manager / Process Explorer durante navegação contínua rápida.

---

### Problema 3: Falta de Cancelamento Cooperativo de Decodificações Obsoletas
* **Arquivo e Localização:** [`src/main.rs:294-299`](../src/main.rs#L294-L299), [`src/main.rs:336-349`](../src/main.rs#L336-L349), [`src/main.rs:363-385`](../src/main.rs#L363-L385)
* **Problema Identificado:** O aplicativo utiliza um contador `generation: u64` para descartar mensagens obsoletas que chegam ao canal da interface. No entanto, a thread que já foi disparada para decodificar a imagem anterior continua executando até o fim, lendo dezenas ou centenas de megabytes do disco e descompactando pixels desnecessariamente.
* **Impacto Técnico:** Desperdício de ciclos de CPU e largura de banda de I/O em navegações rápidas por setas do teclado (`←` e `→`), gerando picos de consumo de energia e atraso no processamento da imagem em foco.
* **Gravidade:** **MÉDIA**
* **Solução Recomendada:** Introduzir cancelamento cooperativo através de `std::sync::atomic::AtomicBool` ou tokens de geração atômicos compartilhados com a rotina de decodificação.
* **Dependências:** Tipos atômicos da biblioteca padrão de Rust.
* **Validação:** Teste de troca rápida de 30 fotos consecutivas verificando que jobs obsoletos abortam imediatamente após o início.

---

### Problema 4: Filtragem de Textura Estática Inadequada para Zoom Extremo e Pixel Art
* **Arquivo e Localização:** [`src/main.rs:402`](../src/main.rs#L402), [`src/main.rs:426`](../src/main.rs#L426)
* **Problema Identificado:** Toda textura de imagem é enviada para a GPU com `TextureOptions::LINEAR` fixo:
  ```rust
  ctx.load_texture(..., TextureOptions::LINEAR);
  picture.texture.set(color, TextureOptions::LINEAR);
  ```
* **Impacto Técnico:** Ao visualizar ícones pequenos (16×16, 32×32), texturas de jogos (DDS) ou aplicar zoom alto (>100%), a imagem fica embaçada e distorcida por interpolação bilinear, sem opção de exibição *Pixel-Perfect* (`Nearest Neighbor`).
* **Gravidade:** **MÉDIA**
* **Solução Recomendada:** Implementar comutação entre modos de interpolação:
  1. `Auto` (Linear para zoom <= 100%, Nearest para zoom > 100%).
  2. `Linear / Smooth` (sempre linear).
  3. `Nearest / Pixel-Perfect` (sempre vizinho mais próximo).
* **Dependências:** Suporte nativo do `egui::TextureOptions`.
* **Validação:** Teste visual com ícone de 16x16 em zoom de 800% garantindo contornos nítidos e quadrados de pixel.

---

### Problema 5: Ausência de Cópia para a Área de Transferência do Sistema Operacional (`Ctrl+C`)
* **Arquivo e Localização:** [`src/main.rs:609-696`](../src/main.rs#L609-L696)
* **Problema Identificado:** O aplicativo possui atalhos para Abrir (`Ctrl+O`), Exportar (`Ctrl+S`), Info (`I`), Girar (`R`), etc., mas não implementa a funcionalidade essencial de copiar a imagem exibida para o Clipboard do Windows (`Ctrl+C`).
* **Impacto Técnico:** Severa perda de produtividade. Usuários de visualizadores modernos necessitam copiar imagens para colar diretamente em mensageiros (WhatsApp, Discord, Slack) e editores gráficos. A crate `eframe` já possui a feature `clipboard` ligada, mas o código não a utiliza para bitmaps.
* **Gravidade:** **ALTA** (Usabilidade)
* **Solução Recomendada:** Implementar suporte nativo para gravação de DIB/PNG no clipboard do Windows via API Win32 ou integração com o clipboard do `eframe`/`arboard`.
* **Dependências:** `arboard` (já transitiva) ou `windows-sys` (já instalada).
* **Validação:** Pressionar `Ctrl+C` no Visor e executar `Ctrl+V` em outro aplicativo de edição de imagem conferindo fidelidade e canal alfa.

---

### Problema 6: Invasão de UX no Primeiro Início (Abertura Forçada de Configurações do Windows)
* **Arquivo e Localização:** [`src/main.rs:1425-1427`](../src/main.rs#L1425-L1427), [`src/main.rs:1431-1449`](../src/main.rs#L1431-L1449)
* **Problema Identificado:** Na primeira execução (`default-attempted.txt` ausente), o código executa em background:
  ```rust
  if open_settings_on_failure && !report.png_ok {
      open_default_apps_settings(); // Executa cmd /C start "" ms-settings:defaultapps
  }
  ```
* **Impacto Técnico:** Quando o usuário abre o aplicativo pela primeira vez, uma janela de configurações do sistema operacional é projetada na tela sem autorização, causando atrito, estranheza e percepção de software invasivo.
* **Gravidade:** **ALTA** (UX / Reputação do Software)
* **Solução Recomendada:** Remover a chamada forçada no primeiro launch. Substituir por um banner discreto na UI principal que pergunta educadamente se o usuário deseja associar arquivos, com ações claras: `[Definir como padrão]` e `[Dispensar]`.
* **Dependências:** Apenas reestruturação da lógica de notificação.
* **Validação:** Inicialização em ambiente limpo sem popup de `ms-settings:defaultapps`.

---

### Problema 7: Carrossel de Miniaturas (Filmstrip) com Janela Rígida e Sem Ocultação
* **Arquivo e Localização:** [`src/main.rs:922-968`](../src/main.rs#L922-L968), [`src/main.rs:994-1000`](../src/main.rs#L994-L1000)
* **Problema Identificado:** O cálculo das miniaturas usa valores fixos:
  ```rust
  let start = self.index.saturating_sub(3);
  let end = (self.index + 4).min(self.files.len());
  ```
  Isso gera exatamente 7 miniaturas, ignorando se a janela está em tela cheia (onde caberiam 20+) ou espremida. Além disso, o filmstrip não pode ser ocultado pelo usuário.
* **Impacto Técnico:** Uso ineficiente do espaço visual, poluição visual quando o usuário quer focar exclusivamente na imagem e renderização de miniaturas vazias com texto bruto desenhado dentro.
* **Gravidade:** **MÉDIA**
* **Solução Recomendada:**
  1. Tornar o carrossel retrátil com atalho `T` e botão dedicado.
  2. Calcular a quantidade de miniaturas dinamicamente com base em `ui.available_width()`.
  3. Adicionar placeholders discretos com cantos arredondados enquanto o carregamento assíncrono ocorre.
* **Dependências:** Cálculo geométrico no `src/ui/filmstrip.rs`.
* **Validação:** Teste em larguras de 600px, 1200px e 2560px verificando distribuição proporcional.

---

### Problema 8: Falta de Cache LRU de Imagens e Texturas
* **Arquivo e Localização:** [`src/main.rs:156-186`](../src/main.rs#L156-L186), [`src/main.rs:290`](../src/main.rs#L290)
* **Problema Identificado:** Não há cache de imagens completas em memória. Se o usuário estiver na foto 5, for para a foto 6 e voltar para a foto 5, a foto 5 é inteiramente relida do disco e decodificada novamente do zero. O cache de thumbnails (`thumbs`) é apenas um `HashMap` plano sem teto de memória configurável.
* **Impacto Técnico:** Latência desnecessária ao alternar entre imagens consecutivas para comparação e desperdício de energia.
* **Gravidade:** **MÉDIA**
* **Solução Recomendada:** Implementar um cache LRU (*Least Recently Used*) de decodificação com limite estrito de memória (ex: 128 MB ou 3 a 5 imagens recentes completas), liberando texturas não utilizadas.
* **Dependências:** Estrutura LRU simples ou baseada em `std::collections::VecDeque`.
* **Validação:** Alternância entre foto A e foto B com tempo de carregamento < 2ms (cache hit).

---

### Problema 9: Bloqueio da Thread Principal por Diálogos Nativos Síncronos (`rfd`)
* **Arquivo e Localização:** [`src/main.rs:465-484`](../src/main.rs#L465-L484), [`src/main.rs:486-517`](../src/main.rs#L486-L517), [`src/main.rs:519-535`](../src/main.rs#L519-L535), [`src/main.rs:537-571`](../src/main.rs#L537-L571)
* **Problema Identificado:** As chamadas a `rfd::FileDialog::new().pick_file()`, `save_file()` e `pick_folder()` são executadas sincronamente no início do ciclo de update da interface.
* **Impacto Técnico:** Enquanto o diálogo do Explorer está aberto, a thread da interface fica bloqueada, o que pode disparar advertências do Windows sobre aplicativo que não responde caso o Explorer demore a listar unidades de rede.
* **Gravidade:** **MÉDIA**
* **Solução Recomendada:** Despachar diálogos de arquivo em tarefas assíncronas com retorno via canal, ou utilizar as APIs assíncronas do `rfd`.
* **Dependências:** API assíncrona do `rfd` ou canal `mpsc`.
* **Validação:** Abertura do diálogo de arquivos sem travamento de repintura da interface.

---

### Problema 10: Ausência de Atalho e Ação de Enviar para a Lixeira do Windows (`Delete`)
* **Arquivo e Localização:** [`src/main.rs:609-696`](../src/main.rs#L609-L696)
* **Problema Identificado:** O aplicativo não implementa exclusão de arquivos com tecla `Delete`.
* **Impacto Técnico:** Triagem de fotos (descarte de fotos tremidas ou duplicadas) é o fluxo de trabalho mais comum em visualizadores de fotos. Sem isso, o usuário é obrigado a alternar para o Windows Explorer para apagar arquivos.
* **Gravidade:** **ALTA** (Usabilidade)
* **Solução Recomendada:** Implementar envio seguro de arquivos para a Lixeira do Windows usando a API `SHFileOperationW` / `IFileOperation` (`FO_DELETE` com flag `FOF_ALLOWUNDO`), atualizando a lista de navegação sem fechar a sessão.
* **Dependências:** `windows-sys` (`Win32_UI_Shell`).
* **Validação:** Pressionar `Delete`, confirmar remoção para a Lixeira e verificar se o arquivo permanece recuperável na Lixeira do Windows.

---

## 4. Auditoria do Ambiente de Compilação e Qualidade

Durante os testes locais de auditoria, verificamos o comportamento dos comandos de inspeção:

1. **`cargo check --all-targets --all-features`:** Executado com sucesso em 8.92s. Zero warnings do compilador.
2. **`cargo test --all-targets`:** 17 testes executados (13 em `lib.rs`, 4 em `main.rs`), 100% aprovados em ~1.21s.
3. **`cargo clippy`:** Falha de execução devido à ausência do binário `cargo-clippy.exe` na toolchain `stable-x86_64-pc-windows-gnu`.
4. **`cargo fmt --check`:** Falha de execução devido à ausência do binário `cargo-fmt.exe` na toolchain local.
5. **Tamanho do Executável Release:** `target/release/visor.exe` possui 17.776.640 bytes (~17,7 MB), o que demonstra compactação eficiente para um binário autônomo com dezenas de codecs embutidos e runtime OpenGL.

---

## 5. Conclusão da Auditoria

O motor central de codecs e o modelo de domínio do Visor possuem excelência técnica: não sofrem de vazamento de rede, tratam erros de decodificação com segurança (`catch_unwind`) e respeitam especificações de arquivos binários com fidelidade.

O gargalo e o déficit de qualidade concentram-se quase exclusivamente no **design visual e na camada de apresentação/concorrência de `src/main.rs`**, que opera com padrões ultrapassados de GUI, ausência de recursos cruciais de produtividade (`Ctrl+C`, `Delete`, menu de contexto) e concorrência sem controle de pool. A intervenção planejada atacará exatamente essas fraquezas sem desestabilizar os codecs já testados e aprovados.
