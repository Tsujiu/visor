# VISOR — Relatório de Auditoria Funcional da Fase 5
## Integração com o Windows, Produtividade e Persistência

**Data:** 08/11/2026  
**Status:** Fase 5 Concluída e Fase 6 / Passo 1 Concluído — Linha de Base Atual: 196 testes aprovados (13 em `src/lib.rs`, 183 no executável)  
**Branch de Trabalho:** `feature/visor-modernization`  
**Linha de Base de Testes Inicial:** 109 testes aprovados (13 em `src/lib.rs`, 96 no executável)

---

### 1. Diagnóstico do Estado Atual

A base de código do VISOR encerrou a Fase 4 em estado de estabilidade comprovada. Todos os 109 testes de unidade, integração, estresse de concorrência e auditoria de memória passam sem falhas ou avisos (`cargo check` e `cargo test` com 0 erros).

A integridade do núcleo de decodificação foi mantida rigorosamente:
- `src/lib.rs`: 100% intacto.
- Codecs (`catalog.rs`, `decode.rs`, `export.rs`, `raster.rs`, `svg.rs`, `rawimg.rs`, `dds.rs`, `ktx.rs`, `jxl.rs`, `emf.rs`, `simple.rs`, `sniff.rs`, `magick.rs`, `model.rs`, `binio.rs`, `assoc.rs`): 100% intactos.

---

### 2. Auditoria Detalhada das Funcionalidades

#### A. Área de Transferência — Ctrl+C

1. **Estado Atual no Código:**
   - O aplicativo possui apenas cópia de texto para a área de transferência:
     - Na barra de status (`src/ui/status.rs`, linhas 150-153): `ui.ctx().copy_text(path.display().to_string())`.
   - **Cópia de imagem (pixels / bitmap) não existe no VISOR.**
   - O atalho `Ctrl+C` não é interceptado em `src/app/shortcuts.rs`.
   - O `eframe` possui a feature `"clipboard"` habilitada no `Cargo.toml`, mas a API de clipboard do `egui` (`egui::Context::copy_text`) suporta exclusivamente texto UTF-8. Não há suporte nativo no `egui` para cópia de dados binários de imagem para a área de transferência do Windows.

2. **Acesso aos Pixels de `visor::Decoded` e Frame Atual:**
   - O estado do aplicativo (`src/app/state.rs`) armazena a imagem ativa em `app.picture: Option<Picture>`.
   - A estrutura `Picture` mantém uma referência compartilhada zero-copy `decoded: Arc<visor::Decoded>` e o índice do frame em exibição `frame_i: usize`.
   - O método `picture.frame()` retorna uma referência direta `&visor::Frame`, contendo:
     - `pub rgba: Vec<u8>` (pixels brutos de 8 bits por canal descompactados em memória).
     - `pub width: u32`.
     - `pub height: u32`.
     - `pub delay_ms: u32`.
   - **Vantagem arquitetural:** O buffer RGBA já está em memória RAM no `Arc<Decoded>`. Não é necessário ler o disco novamente nem redecodificar a imagem para executar a cópia.

3. **Preservação de Transparência no Windows:**
   - O formato padrão legado do Windows `CF_BITMAP` (DDB - Device Dependent Bitmap) descarta o canal alfa.
   - O formato `CF_DIB` clássico com `BITMAPINFOHEADER` não possui especificação padronizada para canal alfa de 32 bits, resultando em transparência preta ou corrompida em diversos softwares.
   - Para garantir transparência perfeita em 100% das aplicações modernas e clássicas do Windows, devem ser registrados no clipboard:
     - **Formato Registrado `"PNG"`:** Obtido via `RegisterClipboardFormatW(w!("PNG"))`. Aplicações modernas como navegadores (Chrome, Edge), Discord, Slack, Telegram, Photoshop e Paint do Windows 11 leem prioritariamente o stream PNG do clipboard, preservando total transparência de 8 bits e compressão sem perdas.
     - **Formato `CF_DIBV5`:** Cabeçalho moderno `BITMAPV5HEADER` (124 bytes) configurado com `bV5Compression = BI_BITFIELDS`, máscaras BGRA de 32 bits (`bV5AlphaMask = 0xFF000000`, `bV5RedMask = 0x00FF0000`, `bV5GreenMask = 0x0000FF00`, `bV5BlueMask = 0x000000FF`), com linhas invertidas (bottom-up scanlines). Suportado por pacotes Microsoft Office, Paint clássico e editores gráficos.
     - **Formato `CF_HDROP` (Opcional/Complementar):** Lista de arquivos contendo o caminho absoluto do arquivo no disco. Permite colar o arquivo como anexo de e-mail ou no Windows Explorer.

4. **Gerenciamento de Memória e Desempenho:**
   - A alocação da memória global (`GlobalAlloc(GMEM_MOVEABLE, ...)`) só ocorre no momento em que o usuário aciona o comando de cópia (`Ctrl+C`).
   - O stream PNG é gerado em memória de forma compacta (usando o encoder PNG já presente no ecossistema Rust do VISOR), ocupando poucos megabytes.
   - O acesso aos pixels brutos do frame atual é referencial (`&frame.rgba`), evitando clonagens redundantes prévias.

---

#### B. Exclusão Segura — Delete

1. **Estado Atual no Código:**
   - Inexistente. A tecla `Key::Delete` não é capturada em `src/app/shortcuts.rs`.
   - Não há métodos de exclusão em `src/app/actions.rs` nem opções nos botões da toolbar.

2. **APIs Windows para a Lixeira:**
   - **`IFileOperation` (Recomendada / Primária):**
     - Interface COM moderna do Windows Vista, 7, 10 e 11.
     - Criação via `CoCreateInstance(&CLSID_FileOperation, ...)`.
     - Flags de operação: `FOFX_RECYCLEONDELETE | FOFX_ADDUNDORECORD`.
     - **Benefício crítico:** Registra a exclusão no histórico de Desfazer (`Ctrl+Z`) do Windows Explorer. Se o usuário excluir acidentalmente, pode desfazer no próprio Windows.
     - Gerencia nativamente diálogos de confirmação do sistema e arquivos protegidos por UAC sem travar a interface.
   - **`SHFileOperationW` (Fallback Secundário):**
     - Função clássica da `shell32.dll` com `FO_DELETE` e flag `FOF_ALLOWUNDO`.
     - Útil caso a inicialização de COM falhe ou em ambientes restritos.
   - **Regra Absoluta:** O VISOR nunca utilizará `std::fs::remove_file` como ação de exclusão padrão. Arquivos sempre serão enviados para a Lixeira do Windows.

3. **Gerenciamento de COM:**
   - A chamada COM requer `CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED)` na thread que executa a operação (ou verificação resiliente de `RPC_E_CHANGED_MODE` caso o egui/winit já tenha inicializado COM em outro modo).
   - Uso de RAII para garantir `CoUninitialize()` correspondente.

4. **Tratamento de Locks de Arquivo no VISOR:**
   - A auditoria comprovou que o VISOR utiliza `std::fs::read(path)` em `src/decode.rs` e `src/raster.rs`.
   - O handle do arquivo é aberto, os bytes são lidos para um vetor em RAM, e o handle é imediatamente fechado pelo drop do Rust.
   - **Conclusão:** O VISOR não mantém arquivos abertos ou travados no sistema de arquivos. A Lixeira do Windows não enfrentará conflitos de compartilhamento causados pelo próprio visualizador.

5. **Sincronização de Estado e Invalidação de Cache:**
   - O `ImageCache` (`src/core/image_cache.rs`) já possui o método implementado e testado:
     ```rust
     pub fn invalidate_path(&self, path: &Path) -> usize
     ```
   - Fluxo completo após exclusão confirmada:
     1. Invocar `app.cache.invalidate_path(&path)` para expurgar a imagem e seus metadados da memória RAM.
     2. Remover o caminho de `app.files` no índice atual.
     3. Ajustar `app.index`: se `app.index >= app.files.len()`, avançar para `app.files.len().saturating_sub(1)`.
     4. Se ainda restarem imagens na pasta: disparar carregamento imediato da nova imagem no índice atual via `app.load_at(...)`.
     5. Se a pasta ficar vazia: definir `app.picture = None`, `app.error = None` e exibir o Empty State do canvas.
     6. Limpar miniatura correspondente do Filmstrip.
     7. Exibir notificação discreta: `app.notice = Some("Imagem enviada para a Lixeira".to_owned())`.

---

#### C. Menu de Contexto (Clique com Botão Direito) — Concluído no Passo 4

1. **Estado Atual no Código (Implementado e Homologado):**
   - Módulo dedicado `src/ui/context_menu.rs` com renderização personalizada Fluent Design.
   - Integração completa via `.context_menu(|ui| ...)` no Canvas (`src/ui/canvas.rs`) e em cada miniatura do Filmstrip (`src/ui/filmstrip.rs`).
   - Módulo nativo `src/system/explorer.rs` com `reveal_in_explorer()` utilizando `SHOpenFolderAndSelectItems` e fallback `explorer.exe /select,"<caminho>"`.
   - Método `app.copy_path()` para cópia do caminho absoluto da imagem ativa ou do arquivo clicado.
   - Método `app.delete_image_at_path()` unificado para exclusão segura tanto da imagem atual quanto de qualquer miniatura do filmstrip.

2. **Arquitetura no egui:**
   - O `egui` oferece suporte nativo e performático a menus de contexto através do método `.context_menu(|ui| { ... })` disponível em qualquer `Response`.
   - No Canvas (`src/ui/canvas.rs`):
     ```rust
     let response = ui.allocate_rect(canvas, Sense::click_and_drag());
     response.context_menu(|ui| {
         super::context_menu::render_context_menu(app, ui, None);
     });
     ```
   - No Filmstrip (`src/ui/filmstrip.rs`):
     - Cada miniatura associa seu próprio `.context_menu(...)`, contextualizado para o arquivo específico (`target_path`) sobre o qual o usuário clicou.

3. **Reutilização Integral de Ações (Sem Duplicação de Código):**
   - O menu de contexto funciona como uma fachada de despacho para ações já existentes em `VisorApp`:
     - *Copiar Imagem* -> `app.copy_current_image(ctx)`.
     - *Copiar Caminho Completo* -> `app.copy_path(ctx, target)`.
     - *Revelar no Explorador de Arquivos* -> `app.reveal_in_explorer(target)`.
     - *Girar 90° à Direita* -> `app.rotate(ctx, true)`.
     - *Girar 90° à Esquerda* -> `app.rotate(ctx, false)`.
     - *Ajustar à Tela* -> `app.view.set_fit()`.
     - *Tamanho Real (100%)* -> `app.view.set_actual(ppp)`.
     - *Modo Pixel Perfect* -> alternância de `app.interpolation`.
     - *Alternar Fundo* -> `app.background.cycle()`.
     - *Painel de Informações* -> `app.show_info = !app.show_info`.
     - *Mover para a Lixeira* -> `app.delete_image_at_path(...)`.

4. **Identidade Visual Fluent Design:**
   - Os itens do menu adotam rigorosamente as métricas do Design System:
     - Fundo do flyout: `surface_flyout` (`#202024F0`).
     - Borda: `border_subtle` (1px translúcido).
     - Cantos: `RADIUS_MD` (8px no container) e `RADIUS_SM` (5px nos itens sob hover).
     - Separadores horizontais sutis de 1px entre grupos funcionais.
     - Exibição de rótulos alinhados à esquerda (13px) e atalhos alinhados à direita (11.5px, cor secundária).
     - Destaque suave em tom avermelhado para a ação destrutiva ("Mover para a Lixeira").

---

#### D. Configurações Persistentes

1. **Estado Atual no Código:**
   - **Nenhum sistema de persistência existe atualmente.**
   - A cada inicialização, os valores padrão são reinstanciados em `VisorApp::new(...)`:
     - Interpolação: `Auto`.
     - Fundo: `DarkChecker`.
     - Painel de Informações: oculto (`false`).
     - Filmstrip: visível (`true`).
     - Dimensões de janela: `1200 x 800` fixas no `NativeOptions`.

2. **Diretório Adequado no Windows:**
   - Conforme as diretrizes da Microsoft para aplicações desktop:
     - Configurações do usuário pertencem ao diretório Roaming: `%APPDATA%\Visor\settings.json` (ou no diretório unificado `%LOCALAPPDATA%\Visor\settings.json` já utilizado por `src/system/paths.rs::local_dir()`).
     - Manter tudo sob `%LOCALAPPDATA%\Visor\` centraliza relatórios, marcadores e preferências em um único local previsível e isolado por usuário.

3. **Campos Essenciais de Configuração (`VisorConfig`):**
   - `interpolation: InterpolationMode` (Auto, Linear, Nearest).
   - `background: CanvasBackground` (DarkChecker, LightChecker, Black, White).
   - `show_filmstrip: bool` (padrão: `true`).
   - `show_info: bool` (padrão: `false`).
   - `mouse_wheel_action: MouseWheelAction` (Zoom vs Navegação).
   - `confirm_delete: bool` (padrão: `true`).
   - `cache_max_mb: usize` (padrão: 128).
   - `assoc_prompt_dismissed: bool` (padrão: `false`).
   - `window_size: Option<[f32; 2]>`.
   - `window_maximized: bool`.

4. **Estratégia de Serialização sem Dependências Pesadas:**
   - Atualmente, o `Cargo.toml` **não inclui `serde` nem `serde_json`**.
   - Para respeitar o princípio de zero dependências externas supérfluas:
     - Um leitor/escritor compacto em `src/system/config.rs` em formato JSON simples ou chave-valor (~80 linhas de Rust puro e seguro).
     - Tolerância a falhas: se o arquivo não existir, estiver corrompido, bloqueado por permissão ou for de versão antiga, o sistema faz fallback gracioso para `VisorConfig::default()` sem emitir pânico ou travar o programa.

5. **Ciclo de Vida de Salvamento:**
   - Carregamento assíncrono ou síncrono ultra-rápido no `main.rs` antes de instanciar `VisorApp`.
   - Salvamento automático acionado quando uma preferência é modificada ou na finalização do aplicativo (`eframe::App::on_exit`).

---

#### E. Comportamento da Roda do Mouse

1. **Estado Atual no Código:**
   - Implementado diretamente dentro de `src/ui/canvas.rs` (linhas 86-93):
     ```rust
     if response.hovered() {
         let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
         if scroll_y.abs() > 0.1 {
             let anchor = response.hover_pos().unwrap_or_else(|| canvas.center());
             let factor = (scroll_y * 0.0016).exp();
             apply_zoom(app, canvas, image_px, anchor, factor, min_scale, max_scale);
         }
     }
     ```
   - O scroll vertical do mouse está acoplado de forma rígida exclusivamente ao zoom ancorado.
   - Não existe opção para navegar entre imagens anteriores/próximas usando a roda.

2. **Requisitos de Usabilidade e Padrões da Indústria:**
   - Em visualizadores populares (Honeyview, IrfanView, Visualizador de Fotos do Windows):
     - **Modo 1 (Padrão Atual):** Roda = Zoom; Ctrl + Roda = Zoom.
     - **Modo 2 (Modo Navegação):** Roda = Próxima / Anterior; Ctrl + Roda = Zoom.
   - A inclusão do modificador `Ctrl` permite que, mesmo quando o usuário escolhe navegar com a roda, o recurso de zoom rápido continue disponível sem necessidade de troca de configuração.

3. **Prevenção de Disparos Múltiplos (Debounce / Acumulador):**
   - Mouses modernos com rolagem suave (touchpads, mouses de precisão) disparam dezenas de eventos `smooth_scroll_delta` por segundo.
   - Na navegação por imagens, é essencial implementar um acumulador com limiar (ex: delta acumulado > 25.0 ou uso de `raw_scroll_delta.y.signum()`) para evitar que uma única passada de dedo avance 15 imagens de uma vez.

---

#### F. Associações de Arquivos e Primeiro Lançamento

1. **Estado Atual no Código:**
   - O arquivo `src/system/assoc.rs` orquestra a chamada de associação:
     ```rust
     pub fn maybe_register_on_first_launch(tx: Sender<String>) {
         // Se o PNG não estiver associado e o marcador default-attempted.txt não existir:
         std::thread::spawn(move || {
             let _ = tx.send(register_and_store(true));
         });
     }
     ```
   - No método `register_and_store(true)`:
     ```rust
     if open_settings_on_failure && !report.png_ok {
         open_default_apps_settings();
     }
     ```
   - O método `open_default_apps_settings()` executa:
     `cmd /C start "" ms-settings:defaultapps`
   - **Problema de UX:** No Windows 11 moderno, o sistema operacional bloqueia a troca forçada de visualizador padrão por programas (mecanismo UCPD). Como consequência, o `report.png_ok` quase sempre retorna falso, abrindo compulsoriamente a janela de Configurações do Windows logo no primeiro segundo em que o usuário abre o VISOR. Essa experiência é invasiva e indesejada.

2. **Diretriz de Correção:**
   - **Eliminar a abertura automática do `ms-settings:defaultapps` no lançamento inicial.**
   - Registrar as capacidades do aplicativo no Registro do Windows (`HKCU\Software\Classes`, `Capabilities`, `RegisteredApplications`) de forma limpa e silenciosa em segundo plano.
   - Exibir um **Banner Não-Invasivo** na base ou topo da janela principal:
     - Texto: *"O VISOR ainda não é o visualizador padrão para suas fotos."*
     - Botão primário: `[Definir como Padrão]` (que aciona o registro e abre as configurações do Windows com consentimento explícito).
     - Botão secundário: `[Não Perguntar Novamente]` (que grava `assoc_prompt_dismissed = true` na configuração).
   - O VISOR já suporta o registro de todas as extensões válidas catalogadas através de `visor::catalog::verified_extensions()`.

---

### 3. Auditoria Arquitetural e Divisão de Responsabilidades

| Camada | Módulos Atuais | Novos Módulos Propostos | Responsabilidade Estrita |
|---|---|---|---|
| **`src/system/`** | `assoc.rs`, `paths.rs`, `mod.rs` | `clipboard.rs`<br/>`trash.rs`<br/>`config.rs`<br/>`explorer.rs` | **APIs do Windows e SO.** Win32 Clipboard, `IFileOperation`/Shell Recycle Bin, leitura/gravação de `%LOCALAPPDATA%\Visor\settings.json`, e `SHOpenFolderAndSelectItems`. **Zero código de egui aqui.** |
| **`src/app/`** | `actions.rs`, `shortcuts.rs`, `state.rs`, `mod.rs` | *(nenhum arquivo novo necessário)* | **Orquestração e Estado.** Despacho de teclas (`Ctrl+C`, `Del`), manipulação de listas de arquivos, coordenação entre cache e comandos do sistema. |
| **`src/ui/`** | `canvas.rs`, `toolbar.rs`, `filmstrip.rs`, `info_panel.rs`, `status.rs`, `dialogs.rs`, `theme.rs` | `context_menu.rs`<br/>`banner.rs` | **Apresentação e Interação.** Menus de contexto no canvas e filmstrip, banner discreto de associação, diálogos de confirmação de exclusão. |
| **`src/core/`** | `geometry.rs`, `navigation.rs`, `image_ops.rs`, `image_cache.rs`, `stress_tests.rs` | *(nenhum arquivo novo necessário)* | **Regras de Negócio Puras.** Geometria, LRU cache e ordenação. Não depende de Windows nem de interface gráfica. |
| **`src/workers/`** | `thread_pool.rs`, `task_queue.rs`, `cancellation.rs` | *(nenhum arquivo novo necessário)* | **Concorrência.** Execução assíncrona desacoplada de decodificações e operações de I/O pesadas. |

---

### 4. Inventário Consolidado de Funcionalidades

| Funcionalidade | Status Atual | Diagnóstico da Auditoria |
|---|---|---|
| **Copiar Caminho de Arquivo** | **Existente** | Implementado via clique no nome na Status Bar (`ui.ctx().copy_text`). |
| **Copiar Imagem (Ctrl+C)** | **Concluído** | Implementado em `src/system/clipboard.rs` com suporte simultâneo a `"PNG"` e `CF_DIBV5`. |
| **Copiar Arquivo (CF_HDROP)** | **Ausente** | Pode ser fornecido junto ao stream de imagem no clipboard. |
| **Mover para Lixeira (Delete)** | **Concluído** | Implementado em `src/system/trash.rs` com `IFileOperation` (`FOFX_RECYCLEONDELETE | FOFX_ADDUNDORECORD`) e executor STA. |
| **Menu de Contexto (Canvas)** | **Concluído** | Implementado em `src/ui/context_menu.rs` com ações nativas. |
| **Menu de Contexto (Filmstrip)** | **Concluído** | Integrado com o filmstrip preservando a identidade do arquivo clicado. |
| **Revelar no Windows Explorer** | **Concluído** | Implementado em `src/system/explorer.rs` (`SHOpenFolderAndSelectItems` com PIDLs de pasta e relativo). |
| **Configurações Persistentes** | **Concluído** | Implementado em `src/system/config.rs` gravando em `%LOCALAPPDATA%\Visor\settings.json` com gravação atômica. |
| **Roda do Mouse: Seleção de Ação** | **Concluído** | Suporte a Zoom vs Navegação com acumulador de inércia e isolamento de viewport. |
| **Integridade de Cor RGBA** | **Concluído** | Convenção única sRGBA desmultiplicada em `AppSettings` e `Color32::from_rgba_unmultiplied`. |
| **Associações Windows & Primeiro Uso** | **Concluído** | ProgID `Visor.Image`, `Capabilities`, `OpenWithProgids` e banner não-intrusivo em `src/ui/banner.rs`. |
| **Invalidação de Cache após Delete** | **Pronta** | Método `ImageCache::invalidate_path` já existe e está homologado com 100% de testes. |

---

### 5. Análise de Riscos Específicos do Windows e Mitigações

1. **Concorrência ao Abrir Clipboard (`OpenClipboard`):**
   - *Risco:* Outro processo em segundo plano (gerenciadores de área de transferência, antivírus) pode manter o clipboard aberto momentaneamente, fazendo `OpenClipboard` falhar com `ERROR_ACCESS_DENIED`.
   - *Mitigação:* Implementar laço com 5 tentativas espaçadas por `sleep(10ms)` antes de desistir e notificar o usuário graciosamente.

2. **Deadlock ou Bloqueio de Thread de UI na Exclusão:**
   - *Risco:* A exclusão via `IFileOperation` pode exibir diálogos do próprio Windows (ex: arquivo em uso por outro software, permissão especial).
   - *Mitigação:* Passar o `HWND` da janela principal para que qualquer modal do Windows seja parente da janela do VISOR, ou despachar a operação com timeout seguro.

3. **Arquivos em Unidades de Rede ou Pendrives (Sem Lixeira):**
   - *Risco:* Unidades de rede (SMB/NAS) ou pen drives FAT32/exFAT não suportam Lixeira do Windows; a API Shell retorna erro ou solicita confirmação de exclusão permanente.
   - *Mitigação:* `IFileOperation` lida nativamente com essa situação perguntando ao usuário se deseja excluir permanentemente quando a Lixeira não estiver disponível. O VISOR nunca forçará exclusão permanente silenciosa.

4. **Corrupção do Arquivo de Configuração:**
   - *Risco:* Queda de energia ou fechamento abrupto durante a gravação de `settings.json` pode truncar o arquivo.
   - *Mitigação:* Gravação atômica: escrever em `settings.json.tmp` e renomear atomicamente para `settings.json` via `fs::rename`.
