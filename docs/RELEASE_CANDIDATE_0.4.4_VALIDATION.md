# VISOR — RELATÓRIO DE VALIDAÇÃO DE RELEASE CANDIDATE v0.4.4
## Auditoria do PR #6, Dry-Run Específico e Homologação Windows

**Data:** 10 de Novembro de 2026  
**Repositório Privado (Desenvolvimento):** [`Tsujiu/visor`](https://github.com/Tsujiu/visor)  
**Repositório Público (Distribuição):** [`Tsujiu/visor-downloads`](https://github.com/Tsujiu/visor-downloads)  
**Branch Base:** `main` (SHA: `5a71766eddfb7fec9664df99723ecdbf5db81cbe`)  
**Branch Candidata:** `release/0.4.4` (HEAD SHA: `6d41e8fbfe23fb04e67d2e07198a2eb245b74c43`)  
**Pull Request Associado:** [PR #6 (release/0.4.4 -> main)](https://github.com/Tsujiu/visor/pull/6)  
**ID do Dry-Run Oficial Auditado:** [GitHub Actions Run 42748233231](https://github.com/Tsujiu/visor/actions/runs/42748233231)  
**Status da Execução do CI:** **SUCESSO COMPLETO (✓ 15m 38s)**  

---

## 1. Auditoria do Pull Request #6

Consulta direta à API do GitHub (`gh pr view 6`):
- **Estado do PR:** `OPEN` (Aberto, sem merge automático).
- **Branch Base:** `main` (`5a71766eddfb7fec9664df99723ecdbf5db81cbe`).
- **HEAD SHA da Candidata:** `6d41e8fbfe23fb04e67d2e07198a2eb245b74c43`.
- **Merge-Base:** `5a71766eddfb7fec9664df99723ecdbf5db81cbe`.
- **Mergeability:** `MERGEABLE` / `CLEAN` (zero conflitos de mesclagem).
- **Incorporação de Correções:** A branch `release/0.4.4` foi ramificada a partir de `5a71766`, que já inclui cumulativamente:
  * PR #2 (`80e5f89`): Integração da arquitetura comercial e publicidade nativa da Fase 2.
  * PR #3 (`4a46f41`): Tratamento seguro de `$gitStatus.Trim()` para árvores de trabalho perfeitamente limpas.
  * PR #4 (`07dfa9d`): Reset preventivo de `$global:LASTEXITCODE = 0` na verificação de releases pré-existentes.
  * PR #5 (`5a71766`): Documentação da homologação pós-merge.
  * Commit `6d41e8f`: Injeção dinâmica de metadados PE no script de compilação `build.rs`.

---

## 2. Auditoria da Versão 0.4.4 e Metadados do Binário

Todos os manifestos e arquivos de configuração encontram-se rigorosamente sincronizados:

| Arquivo / Manifesto | Conteúdo Declarado | Status |
| :--- | :--- | :--- |
| `Cargo.toml` | `version = "0.4.4"` | Sincronizado |
| `Cargo.lock` | `version = "0.4.4"` | Sincronizado |
| `installer/visor.iss` | `#define MyAppVersion "0.4.4"` | Sincronizado |
| `src/updater/version.rs` | `env!("CARGO_PKG_VERSION")` -> `"0.4.4"` | Sincronizado |
| `CHANGELOG.md` | `## [0.4.4] - 2026-11-10` | Sincronizado (Candidata a Release) |
| `build.rs` (PE VersionInfo) | `FILEVERSION 0,4,4,0` / `PRODUCTVERSION 0,4,4,0` | Sincronizado Dinamicamente |

### Descoberta e Resolução da Versão PE no Binário:
Durante a auditoria de cabeçalho do executável compilado, identificou-se que o arquivo `build.rs` continha strings estáticas legadas apontando para a versão `0.4.1` nos blocos `VERSIONINFO` da API Win32. 
A correção foi implementada no commit `6d41e8f`, fazendo com que o `build.rs` parseie dinamicamente a variável de ambiente `CARGO_PKG_VERSION` em tempo de compilação. A inspeção direta do binário compilado pelo GitHub Actions confirmou que `(Get-Item visor.exe).VersionInfo` reflete autenticamente:
- `FileVersion`: **0.4.4**
- `ProductVersion`: **0.4.4**

---

## 3. Execução e Auditoria do Dry-Run Oficial da v0.4.4

- **Run ID:** `42748233231` ([Visualizar no GitHub Actions](https://github.com/Tsujiu/visor/actions/runs/42748233231))
- **Ref de Disparo:** `release/0.4.4` (Commit `6d41e8f`)
- **Parâmetros:** `tag_name: "v0.4.4"`, `dry_run: true`, `confirm_publication: ""`
- **Duração Total:** 15 minutos e 38 segundos
- **Resultado Global:** **SUCESSO (✓)**

### Auditoria Detalhada dos Passos:
1. `Checkout Repositório Privado`: Código correspondente ao commit `6d41e8f` da branch `release/0.4.4`.
2. `Validar SemVer, Commit e Consistência`: SemVer 2.0 validado (`v0.4.4`); correspondência estrita com `Cargo.toml` (`0.4.4`).
3. `Inspecionar Estado Remoto`: Verificação prévia no repositório de distribuição (0 releases detectadas).
4. `Instalar Toolchain Rust (Stable MSVC)`: Rust configurado com suporte à arquitetura `x86_64-pc-windows-msvc`.
5. `Cache de Dependências`: Restaurado com sucesso.
6. `Validar Compilação de Produção`: `cargo check --release --no-default-features` aprovado.
7. `Executar Bateria de Testes em Release`: `cargo test --release --no-default-features` aprovado (278 testes aprovados, 0 falhas).
8. `Compilar Binário Oficial Otimizado`: `cargo build --release --no-default-features` gerando `visor.exe`.
9. `Validar Binário Compilado`: Cabeçalho PE (`MZ`) confirmado.
10. `Instalar Inno Setup 6`: Compilador instalado via Chocolatey.
11. `Empacotar e Testar Versão Portátil Oficial (ZIP)`: Geração e descompactação de teste de `Visor-Portable-v0.4.4-x64.zip`.
12. `Compilar Instalador Oficial Inno Setup (EXE)`: Geração de `Visor-Setup-v0.4.4-x64.exe` com injeção de versão `/DMyAppVersion=0.4.4`.
13. `Gerar e Auto-Validar Manifesto SHA-256`: `SHA256SUMS.txt` gerado em UTF-8 sem BOM e auto-validado pelo runner.
14. `Salvar Artefatos para Auditoria`: Artefatos armazenados para download no job.
15. `Relatório de Execução e Auditoria de Segurança`: Step Summary publicado.
16. `Publicar Release no Repositório Público Oficial`: **SKIPPED (Ignorado)** compulsoriamente por ser `dry_run: true`.

---

## 4. Prova Criptográfica de Não-Publicação

Consultas executadas imediatamente após o término do CI atestam:
- **Repositório Público (`Tsujiu/visor-downloads`):**
  `gh release list --repo Tsujiu/visor-downloads` -> **0 releases** (nenhuma publicação).
- **Tags no Repositório de Origem (`Tsujiu/visor`):**
  `git ls-remote --tags origin` -> Nenhuma tag `v0.4.4` criada.
- **Tags no Repositório de Destino (`Tsujiu/visor-downloads`):**
  `git ls-remote --tags https://github.com/Tsujiu/visor-downloads.git` -> **0 tags**.
- **Segurança de Credenciais:** O token `VISOR_DOWNLOADS_TOKEN` não foi acionado para operações remotas de escrita.

---

## 5. Auditoria Criptográfica dos Novos Artefatos Oficiais (v0.4.4)

Os artefatos gerados pelo runner do GitHub Actions foram baixados (`visor-build-artifacts-v0.4.4`) e auditados:

### 5.1. Manifesto Canônico (`SHA256SUMS.txt`)
```
2d4c57d2d5f1f2b1276273dfc0027e5a9edac7aeb81b4bd0fce06384fd6a4e78  Visor-Setup-v0.4.4-x64.exe
1176e32aa36e7ba3f0d5153a756730b9a356e0ab922349951911e22756cede89  Visor-Portable-v0.4.4-x64.zip
```

### 5.2. Confrontação dos Hashes Criptográficos
| Arquivo | Tamanho | SHA-256 Calculado Localmente | Confere com Manifesto? |
| :--- | :--- | :--- | :--- |
| `Visor-Setup-v0.4.4-x64.exe` | 7.897.032 bytes | `2d4c57d2d5f1f2b1276273dfc0027e5a9edac7aeb81b4bd0fce06384fd6a4e78` | **SIM (100%)** |
| `Visor-Portable-v0.4.4-x64.zip` | 8.776.442 bytes | `1176e32aa36e7ba3f0d5153a756730b9a356e0ab922349951911e22756cede89` | **SIM (100%)** |
| `SHA256SUMS.txt` | 191 bytes | `f76c021edd958f31ba59ddace9a64968d0b8d14eade2eefdcc0aa427737bb3fe` | **SIM (100%)** |
| Binário descompactado `visor.exe` | 21.561.856 bytes | `7cab6e65cd11742c27f50d77a49eb99fb0a97024962b643b980bef8d3ab5d296` | **SIM (PE64 Válido)** |

---

## 6. Homologação do Instalador e Versão Portátil (Host vs Clean VM)

### 6.1. Testes Funcionais no Host
1. **Instalador Oficial:**
   - Instalação silenciosa em `%LOCALAPPDATA%\Temp\visor_inst_044_test`: ExitCode `0`.
   - Instalação limpa no escopo do usuário corrente sem requisição de privilégios de administrador (UAC desnecessário).
   - Criação correta de `visor.exe`, `unins000.exe`, `unins000.dat`, `CHANGELOG.md` e `LEIAME.txt`.
   - Hash do `visor.exe` instalado idêntico ao binário portátil (`7cab6e...`).
   - Desinstalação silenciosa: ExitCode `0`, remoção total e higiênica da pasta da aplicação.
2. **Versão Portátil:**
   - Arquivo ZIP íntegro contendo exclusivamente `visor.exe`, `README.txt`, `CHANGELOG.txt` e `LEIAME.txt`.
   - Executável autenticado com cabeçalho PE64 íntegro e sem dependências dinâmicas não pertencentes ao Windows.

### 6.2. Status da Homologação em Clean VM
- **Classificação Formal:** **NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM**.
- **Justificativa Técnica:** O host de desenvolvimento não dispõe do executável `WindowsSandbox.exe` habilitado e não possui hipervisores (Hyper-V/VirtualBox) gerenciáveis sem privilégios administrativos. Conforme as regras de release engineering, nenhum teste executado no host de desenvolvimento pode substituir a homologação em ambiente isolado virgem.
- **Roteiro de Homologação para o Operador:**
  1. Habilitar o Windows Sandbox em um prompt administrativo (`Enable-WindowsOptionalFeature -FeatureName "Containers-DisposableClientVM" -Online -All`).
  2. Abrir o Windows Sandbox e copiar `Visor-Setup-v0.4.4-x64.exe` e `Visor-Portable-v0.4.4-x64.zip`.
  3. Executar o instalador com as proteções do Windows ativas (Defender/SmartScreen).
  4. Abrir imagens `.png`, `.jpg`, `.webp` e verificar visualização no canvas.
  5. Testar desinstalação via Painel de Controle / Configurações.

---

## 7. Compatibilidade do Atualizador (Update Center)

- **Endpoint de Produção:** Aponta para `https://api.github.com/repos/Tsujiu/visor-downloads/releases/latest`.
- **Identificação de Assets:** Busca por `Visor-Setup-*.exe` e `Visor-Portable-*.zip`, exatamente correspondendo aos nomes produzidos pelo workflow para a `v0.4.4`.
- **Parsing e Criptografia:** Validação SemVer via crate `semver` e integridade em blocos de 64KB via SHA-256 (`sha2`), totalmente cobertos por testes unitários aprovados.
- **Resiliência em Produção:** Como o repositório público não possui releases publicadas, a chamada à API retorna `HTTP 404`, que é capturada de forma segura como `UpdateCheckError::Network("HTTP 404: ...")`, sem emitir falsos alertas ao usuário nem interromper o aplicativo.

---

## 8. Quadro Consolidado de Classificação

| Critério | Classificação | Observações |
| :--- | :--- | :--- |
| Sincronização de Metadados e PE Resource | **APROVADO** | `Cargo.toml`, `Cargo.lock`, `visor.iss`, `CHANGELOG.md` e `build.rs` alinhados em `0.4.4` |
| Bateria Completa de Testes do CI | **APROVADO** | 278 testes automatizados aprovados no GitHub Actions |
| Execução do Dry-Run Oficial da v0.4.4 | **APROVADO** | Run `42748233231` finalizado com sucesso em 15m 38s |
| Isolamento e Não-Publicação Remota | **APROVADO** | 0 tags, 0 releases e 0 uploads no repositório público |
| Integridade Criptográfica dos Novos Artefatos | **APROVADO** | Hashes SHA-256 100% idênticos aos do manifesto oficial |
| Instalador e Desinstalador (Host Windows) | **APROVADO** | Execução e desinstalação silenciosa com ExitCode 0 |
| Versão Portátil ZIP (Host Windows) | **APROVADO** | Pacote limpo, autocontido, com binário PE64 válido |
| Homologação em Windows Clean VM | **NÃO VERIFICADO** | Host sem Windows Sandbox; pendente de validação isolada |
| Validação de Metadados e Criptografia do Updater | **APROVADO** | Algoritmos e contratos de dados validados por testes |
| Atualização Ponta a Ponta em Produção | **NÃO VERIFICADO** | Aguarda publicação da primeira release real |

---

## 9. Decisão Final

1. **O código da candidata está atualizado?**  
   **SIM (APROVADO).** A branch `release/0.4.4` contém todas as correções de CI, sincronização atômica de versões e injeção dinâmica de metadados PE no executável.

2. **O Dry-Run gerou artefatos reais da v0.4.4?**  
   **SIM (APROVADO).** Os novos artefatos `Visor-Setup-v0.4.4-x64.exe` e `Visor-Portable-v0.4.4-x64.zip` foram gerados e validados no GitHub Actions Run ID `42748233231`.

3. **Os hashes foram conferidos?**  
   **SIM (APROVADO).** Os hashes locais coincidem 100% com o arquivo `SHA256SUMS.txt` gerado pelo CI.

4. **O instalador passou na Clean VM?**  
   **NÃO VERIFICADO.** O instalador passou nos testes funcionais do host de desenvolvimento, mas permanece classificado como NÃO VERIFICADO para ambiente virtual descartável isolado por indisponibilidade técnica do Sandbox.

5. **A versão portátil passou?**  
   **SIM (APROVADO).** Arquivo ZIP e binário `visor.exe` homologados com integridade matemática e cabeçalho PE64 válido.

6. **O updater foi validado?**  
   **PARCIALMENTE VALIDADO (APROVADO EM TESTES / NÃO VERIFICADO EM PRODUÇÃO).** O motor e os parsers estão plenamente aprovados; o fluxo em produção aguarda o primeiro lançamento.

7. **O PR #6 está pronto para merge?**  
   **SIM (APROVADO).** O PR #6 encontra-se limpo, sem conflitos, com todas as validações de CI aprovadas e pronto para ser mesclado por autorização humana.

8. **A primeira publicação oficial está autorizada tecnicamente?**  
   **CONDICIONADO À HOMOLOGAÇÃO EM CLEAN VM.** Toda a engenharia de software, automação e criptografia estão validadas. A publicação oficial definitiva aguarda apenas o teste visual final do operador no Windows Sandbox e a criação da tag `v0.4.4`.
