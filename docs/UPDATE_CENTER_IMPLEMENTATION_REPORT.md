# VISOR — UPDATE CENTER
## Relatório Técnico de Implementação, Arquitetura e Validação
### Versão: 0.4.0 | Rust 1.95 (Edição 2024) | Windows x64

---

## 1. DIAGNÓSTICO DA ARQUITETURA ANTERIOR

Nas versões anteriores (até a v0.3.3), o VISOR operava de maneira estritamente estática no que tange ao seu ciclo de vida de software:
- **Sem Verificação Automática:** O usuário não possuía nenhum mecanismo dentro do aplicativo para saber se uma nova versão havia sido publicada no GitHub.
- **Acompanhamento Manual:** A atualização dependia exclusivamente de o usuário visitar periodicamente o repositório `https://github.com/Tsujiu/visor/releases`, baixar os binários e executar manualmente o instalador ou descompactar o arquivo ZIP.
- **Ausência de Comunicação de Novidades:** Ao atualizar, o usuário não recebia nenhuma notificação ou janela descritiva apresentando o que havia mudado, quais recursos haviam sido adicionados ou quais correções haviam sido feitas.
- **Isolamento de Rede Absoluto:** O executável não possuía nenhum cliente HTTP configurado, dependendo de navegadores externos para qualquer obtenção de binários.

O objetivo do **Update Center** foi transformar o VISOR em um software moderno com capacidade autônoma de evolução, mantendo a premissa de transparência, privacidade estrita e controle do usuário.

---

## 2. ARQUITETURA DO NOVO UPDATE CENTER

O sistema foi estruturado sob o princípio de separação de responsabilidades em submódulos desacoplados sob `src/updater/`:

```text
src/
├── updater/
│   ├── mod.rs          # Orquestrador assíncrono, eventos e ciclo de vida
│   ├── github.rs       # Cliente de consulta à API oficial do GitHub Releases
│   ├── version.rs      # Normalização e comparação SemVer
│   ├── download.rs     # Gerenciador de downloads com progresso e velocidade
│   ├── integrity.rs    # Verificação criptográfica de integridade SHA-256
│   ├── install.rs      # Inicialização segura do instalador e suporte portátil
│   └── whats_new.rs    # Parser categorizado do CHANGELOG.md embutido (offline)
│
├── ui/
│   ├── update_dialog.rs # Notificação de atualização e barra de progresso
│   ├── whats_new.rs     # Modal de novidades pós-atualização e histórico
│   └── settings.rs      # Cartão "Atualizações do Aplicativo"
```

### Fluxo Assíncrono e Desacoplamento da UI
- **Zero Bloqueio da UI:** Nenhuma chamada de rede ou I/O pesado roda na thread principal do `eframe`/`egui`.
- **Fila de Tarefas:** Operações de consulta e download são despachadas para o `ThreadPool` em segundo plano (`src/workers/thread_pool.rs`).
- **Comunicação por Canais MPSC:** As tarefas de fundo comunicam eventos (`UpdateEvent::CheckStarted`, `UpdateEvent::CheckFinished`, `UpdateEvent::DownloadProgress`, `UpdateEvent::DownloadCompleted`, `UpdateEvent::DownloadFailed`) através de `std::sync::mpsc::channel()`.
- **Drenagem na `pump()`:** A cada frame, o método `VisorApp::pump()` drena a fila de eventos do updater e atualiza o estado reativo de forma fluida.

---

## 3. APIS DO GITHUB UTILIZADAS

O VISOR consulta exclusivamente a API pública oficial de Releases do repositório:
- **Endpoint Principal:** `https://api.github.com/repos/Tsujiu/visor/releases/latest`
- **Cabeçalho Accept:** `application/vnd.github.v3+json`
- **Cabeçalho User-Agent:** `Visor/<versão> (+https://github.com/Tsujiu/visor)`
- **Validação Estrita de Procedência:** Releases de forks, repositórios de terceiros ou com `draft: true` são rejeitadas categoricamente.
- **Filtragem de Canal:** O canal padrão é *Estável*. Versões marcadas como `prerelease` ou `draft` são ignoradas no canal estável.

---

## 4. ESTRATÉGIA DE VERIFICAÇÃO DE VERSÕES (SemVer)

- **Versão Nativa:** A versão atual é obtida diretamente em tempo de compilação via `env!("CARGO_PKG_VERSION")`.
- **Motor de Comparação:** Utilização da biblioteca padrão `semver` (v1.0).
- **Normalização de Tags:** Limpeza de prefixos como `v` ou `V` e espaços em branco (`v0.4.0` -> `0.4.0`).
- **Regra Fundamental de Proteção:** O VISOR **nunca** oferece downgrade automático. Uma atualização só é proposta se `versão_remota > versão_local`.
- **Suporte a Pré-lançamentos e Sufixos:** A hierarquia SemVer numérica e léxica é estritamente observada (ex: `0.4.0` é mais recente que `0.4.0-rc.1`).

---

## 5. FREQUÊNCIA E CACHE HTTP (ETag & 304 Not Modified)

Para evitar sobrecarga na rede e respeitar os limites de requisição da API pública do GitHub:
- **Intervalo Padrão:** 24 horas entre verificações automáticas.
- **Cache ETag:** O cabeçalho `ETag` retornado pelo GitHub é memorizado no `settings.json`. Em consultas subsequentes, o VISOR envia `If-None-Match: <etag>`.
- **HTTP 304 Not Modified:** Quando a release no servidor não foi modificada, o GitHub retorna status 304 sem transferir o corpo JSON, reduzindo o consumo de banda a zero e atualizando apenas a data da última verificação.
- **Verificação Manual:** O botão "Verificar agora" nas Configurações permite ignorar temporariamente o intervalo e o ETag para checagem imediata.
- **Resiliência a Falhas:** Em caso de ausência de conexão à internet ou indisponibilidade dos servidores, nenhuma mensagem de erro invasiva é exibida em pop-up; a falha é registrada discretamente nas Configurações e o aplicativo funciona normalmente.

---

## 6. SISTEMA DE DOWNLOADS

- **Diretório Seguro:** Pacotes são baixados em `%LOCALAPPDATA%\Visor\updates\` (pasta isolada do usuário).
- **Arquivo Temporário:** O download é gravado inicialmente como `.tmp`. Apenas após a confirmação criptográfica SHA-256 o arquivo é promovido ao nome final. Em caso de falha ou cancelamento, o arquivo parcial é imediatamente deletado.
- **Métricas em Tempo Real:**
  - Quantidade de bytes baixados / tamanho total (MB / MB).
  - Porcentagem concluída (0 a 100%).
  - Velocidade de download em tempo real (KB/s ou MB/s).
  - Tempo restante estimado (ETA).
- **Cancelamento Imediato:** O usuário pode cancelar o download a qualquer momento por meio de botão dedicado, acionando um flag atômico (`Arc<AtomicBool>`).

---

## 7. VALIDAÇÃO CRIPTOGRÁFICA OBRIGATÓRIA (SHA-256)

Nenhum instalador ou arquivo baixado é executado sem verificação de integridade:
1. O manifesto oficial `SHA256SUMS.txt` da release é baixado.
2. O hash esperado para o pacote selecionado (`Visor-Setup-*.exe` ou `Visor-Portable-*.zip`) é extraído.
3. A soma SHA-256 do arquivo baixado no disco é calculada em blocos de 64 KB utilizando a crate `sha2` (Rust puro, sem dependências de C).
4. Se o hash bater: o status muda para `Pronto para instalar` e a ação de instalação é liberada.
5. Se o hash divergir: o arquivo corrompido ou adulterado é deletado imediatamente, a instalação é estritamente bloqueada e o usuário é notificado.

---

## 8. PROCESSO DE ATUALIZAÇÃO INSTALADA

Para instâncias instaladas pelo Inno Setup:
1. O VISOR identifica que está em `%LOCALAPPDATA%\Programs\Visor`.
2. O download prioriza `Visor-Setup-vX.Y.Z-x64.exe`.
3. Após a validação SHA-256, o usuário clica em `Instalar agora`.
4. O processo do instalador oficial é disparado de forma independente.
5. O VISOR atual fecha sua janela, liberando o executável para atualização in-place transparente pelo Inno Setup.
6. As configurações do usuário (`settings.json`) são 100% preservadas.

---

## 9. PROCESSO DE ATUALIZAÇÃO PORTÁTIL

Para instâncias portáteis (executadas de pen drive ou pastas arbitrárias):
1. O download prioriza `Visor-Portable-vX.Y.Z-x64.zip`.
2. Após validação SHA-256, a interface informa o caminho e disponibiliza o botão `Abrir pasta no Explorer`.
3. O executável antigo não é sobrescrito enquanto estiver em execução pelo sistema operacional.
4. O usuário é orientado a fechar o aplicativo e extrair os novos arquivos sobre a pasta portátil.

---

## 10. SISTEMA DE NOVIDADES PÓS-ATUALIZAÇÃO (What's New)

- **Apresentação Automática:** Na primeira execução após uma atualização bem-sucedida, o VISOR detecta a nova versão e exibe automaticamente a janela:
  `# Bem-vindo ao VISOR vX.Y.Z`
  `Confira as novidades desta versão.`
- **Categorização Inteligente:** As alterações do `CHANGELOG.md` são estruturadas nas categorias:
  - **Novas Funcionalidades** (`### Adicionado`)
  - **Melhorias e Otimizações** (`### Modificado`)
  - **Correções de Bugs** (`### Corrigido`)
  - **Segurança e Privacidade** (`### Segurança`)
  - **Alterações Importantes** (`### Importante`)
- **Funcionamento 100% Offline:** O `CHANGELOG.md` é embutido na compilação via `include_str!`. A janela não depende de internet para funcionar.
- **Memorização:** O campo `post_update_notes_version` registra que as novidades daquela versão já foram apresentadas, impedindo reapresentações incômodas a cada inicialização.
- **Diferenciação de Primeiro Uso:** Em uma instalação limpa (primeira vez que o aplicativo é aberto no computador), o aplicativo registra a versão atual sem exibir o pop-up falso de "Você acabou de atualizar".

---

## 11. ALTERAÇÕES NO ARMAZENAMENTO JSON (Schema v4)

O arquivo `%LOCALAPPDATA%\Visor\settings.json` foi atualizado para o **Schema 4**:

```json
{
  "schema_version": 4,
  "mouse_wheel_action": "zoom",
  "interpolation": "auto",
  "background": "dark_checker",
  "custom_color": [32, 32, 36, 255],
  "show_filmstrip": true,
  "show_info": false,
  "association_prompt_dismissed": false,
  "association_prompt_version": 1,
  "ignored_default_extensions": [],
  "updates": {
    "automatic_check": true,
    "check_interval_hours": 24,
    "last_check": "2026-11-09T01:15:00Z",
    "last_etag": "\"abc123etag\"",
    "ignored_version": null,
    "post_update_notes_version": "0.4.0"
  }
}
```

- Migração transparente a partir de schemas anteriores (v1, v2, v3).
- Preservação contra sobrescrita de versões futuras (`is_future_version`).

---

## 12. ALTERAÇÕES NA INTERFACE GRÁFICA

- **Diálogo Modal de Atualização (`src/ui/update_dialog.rs`):**
  - Notificação discreta com número das versões, resumo das novidades e ações rápidas (`Baixar atualização`, `Ver todas as novidades`, `Lembrar mais tarde`, `Ignorar esta versão`).
  - Painel de progresso com barra percentual, velocidade (MB/s) e estimativa de tempo (ETA).
- **Janela de Novidades da Versão (`src/ui/whats_new.rs`):**
  - Apresentação organizada em categorias com marcadores coloridos e botão para visualização da release completa no GitHub.
  - Seção retrátil para navegação no histórico de versões anteriores.
- **Cartão de Atualizações nas Configurações (`src/ui/settings.rs`):**
  - Exibição da versão instalada, canal de distribuição, data/hora da última verificação e status reativo.
  - Alternância de verificação automática e seletor de intervalo (12h, 24h, 48h, 7 dias).
  - Botões para verificação manual imediata e consulta às novidades.

---

## 13. IMPACTOS NA PRIVACIDADE E DOCUMENTAÇÃO

- **Zero Telemetria:** O VISOR não coleta metadados de máquina, identificadores de hardware, IP para rastreamento ou dados de imagens visualizadas.
- **Transparência de Rede:** Toda a documentação pública (`README.md`, `LEIAME.txt`) foi atualizada para explicar com precisão que a única conexão de rede automática do aplicativo é a consulta pública à API de releases do GitHub.
- **Modo Offline Opcional:** O usuário pode desativar completamente a verificação automática nas configurações, mantendo o executável 100% desconectado.

---

## 14. DEPENDÊNCIAS ADICIONADAS E JUSTIFICATIVAS

| Dependência | Versão | Justificativa Técnica |
| :--- | :--- | :--- |
| **`ureq`** | 2.12.1 | Cliente HTTP simples e bloqueante, ideal para threads de fundo. Utiliza `rustls` (Rust puro) sem dependências de C/OpenSSL externo. |
| **`semver`** | 1.0.28 | Padrão oficial da comunidade Rust para análise sintática e comparação de versões semânticas. |
| **`sha2`** | 0.10.9 | Implementação de alto desempenho em Rust puro para computação da soma criptográfica SHA-256. |

---

## 15. TESTES E RESULTADOS

A suíte de testes automatizados foi expandida para contemplar todos os cenários mandatórios da especificação:

- **Total de Testes Executados:** **232 testes**
  - Testes de decodificadores e codecs (`src/lib.rs`): **13 aprovados**
  - Testes do aplicativo, cache, fila de tarefas e updater (`src/main.rs`): **219 aprovados**
  - **Falhas:** **0**
  - **Avisos de compilação:** **0**

### Cenários Validados
1. `test_section17_version_matrix`: Local < Remota, Local == Remota, Local > Remota, prefixo `v`, pré-lançamentos, comprimentos arbitrários e strings inválidas.
2. `test_section17_persistence_and_schema4_migration`: Migração segura de Schema v3 para Schema v4 com preservação de preferências e inicialização de `UpdateConfig`.
3. `test_section17_should_check_for_updates_timing`: Respeito ao intervalo de 24 horas, detecção de ausência de checagem prévia e desativação manual.
4. `test_section17_ignored_version_behavior`: Ocultação da notificação para versão ignorada e reapresentação automática quando versão superior for lançada.
5. `test_parse_sha256sums` e `test_calculate_and_verify_sha256`: Leitura de manifesto GNU, normalização de nomes, asteriscos e rejeição de hashes divergentes.
6. `test_parse_embedded_changelog` e `test_parse_sample_categories`: Extração das categorias (Novas funcionalidades, Melhorias, Correções, Segurança, Importante) diretamente do arquivo offline.

---

## 16. LIMITAÇÕES CONHECIDAS

1. **Assinatura de Código:** Como o projeto é de código aberto e independente, não utiliza certificado comercial EV. O Windows SmartScreen pode exibir alerta informativo no download inicial. Os hashes no `SHA256SUMS.txt` atestam a integridade matemática da compilação.
2. **Rate Limit do GitHub:** A API não autenticada do GitHub impõe limite de 60 requisições por hora por IP. O sistema com intervalo de 24 horas e suporte a ETag previne o atingimento desse limite em condições normais de uso.

---

## 17. ARQUIVOS ALTERADOS / CRIADOS

- **Criados:**
  - `src/updater/mod.rs`
  - `src/updater/version.rs`
  - `src/updater/github.rs`
  - `src/updater/download.rs`
  - `src/updater/integrity.rs`
  - `src/updater/install.rs`
  - `src/updater/whats_new.rs`
  - `src/ui/update_dialog.rs`
  - `src/ui/whats_new.rs`
  - `docs/UPDATE_CENTER_IMPLEMENTATION_REPORT.md`
- **Modificados:**
  - `Cargo.toml` & `Cargo.lock` (versão 0.4.0 e crates `ureq`, `semver`, `sha2`)
  - `build.rs` (FileVersion 0.4.0, ProductVersion 0.4.0)
  - `installer/visor.iss` (MyAppVersion 0.4.0, VersionInfoVersion 0.4.0.0)
  - `CHANGELOG.md` (seção 0.4.0)
  - `README.md`, `LEIAME.txt`, `README-PORTABLE.txt`
  - `src/main.rs` (registro do módulo `updater`)
  - `src/app/state.rs` (campos `updater`, canais MPSC)
  - `src/app/actions.rs` (drenagem de eventos na `pump`, gatilhos de checagem e what's new)
  - `src/app/mod.rs` (renderização de `render_update_dialog` e `render_whats_new_window`)
  - `src/system/config.rs` (Schema v4, `UpdateConfig` e timing de 24h)
  - `src/ui/mod.rs` & `src/ui/settings.rs` (cartão do Update Center)

---

## CONCLUSÃO

### Status: **APROVADO**

O Update Center do VISOR foi implementado com total fidelidade à especificação técnica, respeitando a arquitetura multithread do aplicativo, sem bloquear a interface gráfica, garantindo validação criptográfica obrigatória via SHA-256 e disponibilizando apresentação offline de novidades em conformidade com os princípios de privacidade e transparência do projeto.
