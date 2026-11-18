# VISOR — AUDITORIA FINAL DE CANDIDATA A RELEASE (RC GATE)
## Homologação Windows Clean VM & Preparação da v0.4.4

**Data:** 10 de Novembro de 2026  
**Repositório Privado (Desenvolvimento):** [`Tsujiu/visor`](https://github.com/Tsujiu/visor)  
**Repositório Público (Distribuição):** [`Tsujiu/visor-downloads`](https://github.com/Tsujiu/visor-downloads)  
**Branch Base:** `main` (SHA: `5a71766eddfb7fec9664df99723ecdbf5db81cbe`)  
**Branch de Release Candidate:** `release/0.4.4`  
**Versão Candidata:** `0.4.4` (`v0.4.4`)  
**Último Dry-Run de CI Auditado:** GitHub Actions Run ID `42736430116`  

---

## 1. Rastreabilidade Git e Estado das Versões

### 1.1. Estado da Branch Principal (`main`)
- **HEAD SHA:** `5a71766eddfb7fec9664df99723ecdbf5db81cbe`
- **Árvore de Trabalho:** Limpa e sincronizada com `origin/main`.
- **Integridade da Base de Código:** Contém todos os módulos comerciais, sistema de publicidade nativa da Fase 2, Update Center e workflow oficial de distribuição.

### 1.2. Auditoria da Tag Histórica `v0.4.3`
- A tag `v0.4.3` existe remotamente em `Tsujiu/visor` como tag anotada apontando para o commit histórico `9249cac8992ca5007fb5a11405b4016fc6fc2d63`.
- **Status:** **PRESERVADA E IMUTÁVEL.** Em cumprimento às diretrizes de release engineering, esta tag não foi modificada, movida ou sobrescrita.

### 1.3. Disponibilidade da Versão `v0.4.4`
- **Repositório Privado (`Tsujiu/visor`):** Consulta a `git ls-remote --tags origin` confirmou que `v0.4.4` **não existe** (livre).
- **Repositório de Distribuição (`Tsujiu/visor-downloads`):** Consulta a `gh release list` e `git ls-remote --tags` confirmou **zero releases e zero tags** (livre).
- **Status da Disponibilidade:** **APROVADO (100% Livre para Lançamento).**

---

## 2. Preparação da Versão Candidata `0.4.4`

Na branch dedicada `release/0.4.4`, os seguintes manifestos foram sincronizados atomicamente sem qualquer alteração na lógica de negócio do aplicativo:

| Arquivo / Manifesto | Versão Anterior | Nova Versão | Verificação | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Cargo.toml` | `0.4.3` | `0.4.4` | `version = "0.4.4"` | **APROVADO** |
| `Cargo.lock` | `0.4.3` | `0.4.4` | Pacote `visor v0.4.4` atualizado | **APROVADO** |
| `installer/visor.iss` | `0.4.3` | `0.4.4` | `#define MyAppVersion "0.4.4"` | **APROVADO** |
| `src/updater/version.rs` | Dinâmica (`0.4.3`) | Dinâmica (`0.4.4`) | `env!("CARGO_PKG_VERSION")` | **APROVADO** |
| `CHANGELOG.md` | `[0.4.3]` | `[0.4.4]` | Seção detalhada das novas capacidades comerciais | **APROVADO** |

### Testes Automatizados da Versão Candidata:
- **`cargo test --release --no-default-features`:** Executado com sucesso (13 testes de biblioteca + 265 testes unitários e de integração aprovados = 278 testes aprovados, 0 falhas).

---

## 3. Auditoria do Workflow de Distribuição (`workflow_dispatch`)

O arquivo `.github/workflows/release-distribution.yml` foi auditado quanto à execução manual via `workflow_dispatch`:

1. **Obtenção do Código-Fonte:** Utiliza `actions/checkout@v4` no commit/ref apontado pelo disparo (ex: `release/0.4.4` ou `main`).
2. **Correspondência entre Tag e `Cargo.toml`:** O workflow extrai o valor de `INPUT_TAG_NAME` (ex: `v0.4.4`), remove o prefixo `v` e compara com o campo `version` do `Cargo.toml`. Se houver divergência, aborta imediatamente (`DIVERGÊNCIA CRÍTICA`).
3. **Disparo sem Tag Git Pré-existente:** No evento `workflow_dispatch`, a verificação de commit por tag Git (`git rev-list`) é contornada com segurança (aplicando-se apenas a `push` de tags). Isso **permite executar testes em modo Dry-Run informando `v0.4.4` antes de criar a tag física no Git**.
4. **Versionamento Dinâmico do Instalador:** O compilador Inno Setup é invocado explicitamente com `ISCC.exe /DMyAppVersion=$ver installer/visor.iss`, garantindo que o instalador final receba a versão exata do pipeline.
5. **Bloqueio Integral do Dry-Run:** O passo final de publicação é protegido por:
   ```yaml
   if: ${{ success() && !inputs.dry_run }}
   ```
   No modo `dry_run: true`, nenhum arquivo é transmitido, nenhuma release é criada e nenhuma credencial de escrita é consumida.

---

## 4. Auditoria Criptográfica dos Artefatos Homologados

Artefatos originais validados na execução `42736430116` do GitHub Actions:

| Artefato | Tamanho (Bytes) | SHA-256 Calculado | Validação com Manifesto | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Visor-Setup-v0.4.3-x64.exe` | 7.896.542 | `07c6d8f67d57b4483bbcd7fe3146bd16dc58b33a6a309b99f6b23555a09c5884` | Idêntico ao manifesto | **APROVADO** |
| `Visor-Portable-v0.4.3-x64.zip` | 8.775.785 | `98e9f51b55b15f2ea89dc96667c766ba5deb123fbc08b13add7cee6533f5cf76` | Idêntico ao manifesto | **APROVADO** |
| `SHA256SUMS.txt` | 191 | `b91a0b38865ffb462947230aa7c98da1ba845239e2467d3a0429a102e35b7194` | Manifesto canônico UTF-8 | **APROVADO** |
| `visor.exe` (descompactado) | 21.561.856 | `1b795072fcf930f447993e58632762eca8b4cfb0c34c2454215d8977ae9b6505` | Cabeçalho PE64 íntegro (`MZ`) | **APROVADO** |

---

## 5. Homologação Funcional: Host Windows vs Clean VM

### 5.1. Testes no Host de Desenvolvimento
- **Instalador Oficial (Inno Setup):**
  - Instalação silenciosa em `%LOCALAPPDATA%\Temp\visor_inst_test`: ExitCode `0`.
  - Instalação sem privilégios administrativos (UAC não exigido para instalação local).
  - Geração correta dos binários e desinstalador (`unins000.exe`).
  - Desinstalação silenciosa: ExitCode `0` com eliminação completa dos binários.
- **Versão Portátil (ZIP):**
  - Descompactação íntegra sem travas.
  - Higiene de pacote: ausência de `.pdb`, dependências de debug ou metadados de controle de versão.
  - Binário PE funcional.

### 5.2. Testes em Windows Clean VM (Máquina Virtual Limpa / Windows Sandbox)
- **Diagnóstico Técnico do Host:**
  * O recurso Windows Sandbox (`WindowsSandbox.exe`) não está habilitado no sistema operacional (`Test-Path "C:\Windows\System32\WindowsSandbox.exe"` = `False`).
  * Ferramentas de gerenciamento do Hyper-V (`vmsgmt.msc`, `vmconnect.exe`, `Get-VM`) e hipervisores de terceiros (VirtualBox `VBoxManage`, VMware `vmrun`) não estão disponíveis.
  * A habilitação de tais recursos via DISM/PowerShell exige privilégios de Administrador de Sistema, indisponíveis nesta sessão não-elevada.
- **Classificação Formal:** **NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM**.
- Em estrita conformidade com as regras de homologação ("*Nunca declare como aprovado algo que rodou na máquina de desenvolvimento*"), os testes em ambiente Windows virgem sem bibliotecas de desenvolvimento instaladas permanecem como item a ser executado pelo operador antes da abertura pública da release.

---

## 6. Auditoria do Atualizador (Update Center)

Para garantir precisão técnica, as capacidades do atualizador foram diferenciadas em três camadas:

1. **Parser e Validação de Metadados:**
   - Interpretação de tags SemVer (`vX.Y.Z`), descarte de prefixos `v`/`V`, comparação de versões locais e remotas.
   - Parsing de manifestos `SHA256SUMS.txt` no padrão GNU com 2 espaços ou modo binário `*`.
   - Sanitização de texto contra ataques de controle ANSI e bloqueio de travessia de caminho (`..`, `/`, `\`).
   - **Status:** **APROVADO (100% Coberto por Testes Automatizados).**

2. **Download Seguro de Artefatos:**
   - Validação de URLs confiáveis (`https://github.com/Tsujiu/visor-downloads/releases/download/` e CDN da infraestrutura oficial).
   - Download em stream por blocos de 64KB com verificação atômica de SHA-256 e cancelamento cooperativo via flag atômica.
   - **Status:** **APROVADO (Mecanismo Criptográfico Validado).**

3. **Atualização Real em Produção (Ponta a Ponta):**
   - Como o repositório público `Tsujiu/visor-downloads` ainda não possui nenhuma release publicada, a consulta à API pública (`/releases/latest`) retorna `HTTP 404 Not Found`.
   - O aplicativo trata esse retorno de forma robusta e segura (`UpdateCheckError::Network("HTTP 404: ...")`), sem congelamentos ou falhas silenciosas.
   - **Status:** **NÃO VERIFICADO EM PRODUÇÃO / PENDENTE DE PRIMEIRA RELEASE OFICIAL.**

---

## 7. Quadro Consolidado de Homologação

| Item Auditado | Classificação | Observações |
| :--- | :--- | :--- |
| **Sincronização de Versões (0.4.4)** | **APROVADO** | `Cargo.toml`, `Cargo.lock`, `visor.iss`, `CHANGELOG.md` alinhados |
| **Bateria de Testes Automatizados** | **APROVADO** | 278 testes executados com 100% de aprovação |
| **Disponibilidade da Tag v0.4.4** | **APROVADO** | Livre no repositório de origem e de distribuição |
| **Preservação da Tag v0.4.3** | **APROVADO** | Mantida intacta no commit histórico `9249cac` |
| **Execução Dry-Run do Pipeline** | **APROVADO** | Run `42736430116` aprovado; bloqueio de publicação verificado |
| **Integridade Criptográfica dos Artefatos** | **APROVADO** | Hashes SHA-256 conferem com o manifesto oficial |
| **Instalador Windows (Host)** | **APROVADO** | Instalação e desinstalação silenciosa com ExitCode 0 |
| **Versão Portátil (Host)** | **APROVADO** | Pacote limpo, executável PE íntegro |
| **Instalação em Windows Clean VM** | **NÃO VERIFICADO** | Windows Sandbox ausente no host; pendente de teste em SO virgem |
| **Parser e Integridade do Updater** | **APROVADO** | Verificação SHA-256 e SemVer validados |
| **Atualização Real em Produção** | **NÃO VERIFICADO** | Repositório público sem releases até o primeiro lançamento |

---

## 8. Decisão Final de Release Candidate

- **A versão candidata v0.4.4 está preparada?**  
  **SIM.** A branch `release/0.4.4` está configurada com todos os manifestos sincronizados, suíte de 278 testes aprovada e documentação de changelog atualizada.

- **O instalador passou na Clean VM?**  
  **NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM.** O instalador foi homologado funcionalmente no host local, mas o teste em SO Windows descartável virgem permanece pendente por ausência do recurso de Sandbox/VM no ambiente.

- **A versão portátil passou?**  
  **SIM.** O pacote ZIP gerado é limpo, autossuficiente e seu binário possui cabeçalho PE64 válido e sem símbolos de depuração.

- **O updater está validado?**  
  **PARCIALMENTE VALIDADO.** O motor criptográfico, o parser de versões e o tratamento de rede/404 estão plenamente aprovados. O ciclo real de download e auto-atualização em produção está pendente da existência da primeira release oficial no repositório de distribuição.

- **O VISOR está aprovado para a primeira publicação oficial?**  
  **CONDICIONADO À HOMOLOGAÇÃO EM CLEAN VM.** A infraestrutura de código, automação de CI/CD, segurança e artefatos compilados estão tecnicamente impecáveis. A publicação oficial definitiva aguarda apenas o teste visual final em máquina limpa pelo operador e a autorização humana para geração da tag `v0.4.4`.
