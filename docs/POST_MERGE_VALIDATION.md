# VISOR — RELATÓRIO DE HOMOLOGAÇÃO PÓS-MERGE
## Main Validation + Release Dry-Run + Windows Host & Clean VM Audit

**Data:** 10 de Novembro de 2026  
**Repositório Privado (Fonte):** `Tsujiu/visor`  
**Repositório Público (Distribuição):** `Tsujiu/visor-downloads`  
**Branch Auditada:** `main`  
**Commit Inicial de Merge (PR #2):** `80e5f89a38230c5d867bfe07bdf606e60d40e747`  
**Commit Final Validado na Main:** `07dfa9d2f0905d118c342fde747f32e1f1287c17`  
**Workflow Executado:** `.github/workflows/release-distribution.yml`  
**Status do Pipeline (Dry-Run):** **SUCESSO COMPLETO (✓ 23m 30s - Run ID: 42736430116)**  

---

## 1. Contexto e Objetivos

Após a aprovação e integração do Pull Request #2 (`feature/visor-ad-network` -> `main`), foi iniciada a fase de homologação pós-merge da branch principal. O objetivo desta auditoria foi:

1. Validar a integridade técnica do código integrado na branch `main`.
2. Resolver e documentar a sincronização de versões entre os manifestos do projeto (`Cargo.toml`, `Cargo.lock`, `installer/visor.iss`, `src/updater/version.rs` e tags Git).
3. Executar o workflow completo de compilação, testes e empacotamento em modo **Dry-Run**, garantindo que nenhum binário ou tag fosse publicado prematuramente.
4. Auditar a segurança do isolamento do Dry-Run perante o repositório público `Tsujiu/visor-downloads`.
5. Inspecionar e validar criptograficamente os artefatos de distribuição gerados no pipeline.
6. Testar o comportamento do instalador e da versão portátil.
7. Auditar a compatibilidade integral entre o Update Center interno do VISOR e os artefatos do workflow.
8. Classificar formalmente o estado de homologação em ambiente limpo (Clean VM).

---

## 2. Rastreabilidade Git e Estabilização da Branch Main

### 2.1. Histórico de Commits na Main
A integração do PR #2 ocorreu no commit `80e5f89`. Durante a execução inicial do workflow de distribuição no GitHub Actions, dois ajustes técnicos finos no script PowerShell de validação do workflow foram identificados e corrigidos através de PRs controlados:

1. **Commit `80e5f89` (PR #2):** Merge da branch `feature/visor-ad-network` trazendo a arquitetura comercial, rede publicitária Fase 2 e o pipeline `.github/workflows/release-distribution.yml`.
2. **Commit `4a46f41` (PR #3 - `fix/ci-clean-git-status-trim`):** Tratamento para evitar chamada de método em expressão nula quando a árvore Git está perfeitamente limpa (`$((git status --porcelain)).Trim()`).
3. **Commit `07dfa9d` (PR #4 - `fix/ci-reset-lastexitcode-on-release-check`):** Reset preventivo de `$global:LASTEXITCODE = 0` após execução do comando `gh release view` na verificação de existência prévia de releases, evitando falso positivo no shell do runner.

O commit ativo e certificado na branch `main` é:
```
SHA: 07dfa9d2f0905d118c342fde747f32e1f1287c17
Mensagem: Merge pull request #4 from Tsujiu/fix/ci-reset-lastexitcode-on-release-check
```

---

## 3. Análise e Resolução de Divergência de Versões

Foi realizada uma auditoria nos arquivos de manifesto e configuração de versão:

| Arquivo / Componente | Versão Declarada | Status |
| :--- | :--- | :--- |
| `Cargo.toml` | `0.4.3` | Sincronizado |
| `Cargo.lock` | `0.4.3` | Sincronizado |
| `installer/visor.iss` | `0.4.3` (`#define MyAppVersion "0.4.3"`) | Sincronizado |
| `src/updater/version.rs` | `env!("CARGO_PKG_VERSION")` -> `0.4.3` | Sincronizado |
| `CHANGELOG.md` | `[0.4.3] - 2026-11-08` | Sincronizado |

### Esclarecimento Histórico das Versões (v0.4.3 vs v0.4.4):
- A tag `v0.4.3` existe remotamente no repositório privado `Tsujiu/visor` apontando para o commit histórico `9249cac` (fase pré-comercial).
- No relatório de segurança anterior (`FINAL_RELEASE_SECURITY_AUDIT.md`), a versão `v0.4.4` foi sugerida como próximo incremento semântico natural para o primeiro release oficial.
- Para o teste de **Dry-Run**, o workflow utilizou a versão declarada nos arquivos do repositório (`0.4.3`). Como o repositório público `Tsujiu/visor-downloads` encontra-se completamente vazio (zero releases), não houve colisão de tags de distribuição, permitindo a validação ponta a ponta do pipeline.
- Para o lançamento oficial futuro, bastará elevar os manifestos para `0.4.4` (ou versão comercial definida) e acionar a publicação real com a tag correspondente.

---

## 4. Execução e Auditoria do Pipeline Dry-Run

### 4.1. Dados da Execução no GitHub Actions
- **Workflow:** `VISOR Release & Distribution Pipeline`
- **ID da Execução:** `42736430116`
- **URL da Execução:** `https://github.com/Tsujiu/visor/actions/runs/42736430116`
- **Branch:** `main` (`07dfa9d`)
- **Parâmetros:**
  - `tag: "v0.4.3"`
  - `dry_run: true`
- **Duração Total:** 23 minutos e 30 segundos
- **Resultado Global:** **SUCCESS (✓)**

### 4.2. Auditoria Passo a Passo das Etapas
1. `Checkout código-fonte`: Sucesso (checkout do commit exato da main).
2. `Auditoria de Tag e Prevenção de Sobrescrita`: Sucesso (tag validada via SemVer 2.0; verificação preventiva no repositório de downloads realizada com êxito).
3. `Configurar toolchain Rust (stable)`: Sucesso (Rustc / Cargo configurados com cache de dependências).
4. `Instalar Inno Setup`: Sucesso (Inno Setup 6 instalado via Chocolatey).
5. `Executar Testes Automatizados`: Sucesso (todos os testes de unidade e integração aprovados).
6. `Auditoria de Segurança de Dependências`: Sucesso (zero vulnerabilidades detectadas).
7. `Compilar Binário Release`: Sucesso (`cargo build --release` gerando `visor.exe`).
8. `Auditoria de Integridade do Binário Compilado`: Sucesso (verificação de integridade PE e ausência de símbolos de debug não autorizados).
9. `Empacotar Versão Portátil (ZIP)`: Sucesso (`Visor-Portable-v0.4.3-x64.zip` gerado com documentação limpa).
10. `Auditoria de Integridade do Pacote Portátil`: Sucesso (arquivo ZIP verificado).
11. `Compilar Instalador Inno Setup`: Sucesso (`Visor-Setup-v0.4.3-x64.exe` compilado).
12. `Auditoria de Integridade do Instalador`: Sucesso (verificação de tamanho e cabeçalho do executável).
13. `Gerar Manifesto SHA-256 (SHA256SUMS.txt)`: Sucesso (hashes calculados).
14. `Upload de Artefatos no Job (Dry-Run / Auditoria)`: Sucesso (artefatos armazenados com 7 dias de retenção).
15. `Publicar Release e Fazer Upload dos Assets no Repositório Público`: **SKIPPED (Ignorado)** por avaliação estrita da condição:
   ```yaml
   if: ${{ success() && !inputs.dry_run }}
   ```

### 4.3. Auditoria de Segurança contra Publicação Prematura
Realizamos consultas diretas via GitHub CLI e Git para atestar o isolamento:
- **Repositório Público (`Tsujiu/visor-downloads`):**
  ```powershell
  gh release list --repo Tsujiu/visor-downloads
  # Retorno: Nenhum registro (0 releases existentes). Repositório intocado.
  ```
- **Tags no Repositório Privado (`Tsujiu/visor`):**
  ```powershell
  git ls-remote --tags origin
  # Retorno: Nenhuma tag nova criada pelo Dry-Run.
  ```
- **Uso de Credenciais:** O token com permissões de escrita `VISOR_DOWNLOADS_TOKEN` permaneceu inativo durante a execução, pois o passo de publicação foi omitido.

---

## 5. Auditoria Criptográfica dos Artefatos de Build

Os artefatos gerados pelo runner do GitHub Actions foram baixados (`Run ID: 42736430116`) e inspecionados:

### 5.1. Manifesto Oficial (`SHA256SUMS.txt`)
Conteúdo extraído do runner:
```
07c6d8f67d57b4483bbcd7fe3146bd16dc58b33a6a309b99f6b23555a09c5884  Visor-Setup-v0.4.3-x64.exe
98e9f51b55b15f2ea89dc96667c766ba5deb123fbc08b13add7cee6533f5cf76  Visor-Portable-v0.4.3-x64.zip
```

### 5.2. Comparação de Hashes Criptográficos Locais
| Arquivo | Tamanho | Hash Calculado Localmente (SHA-256) | Confere com Manifesto? |
| :--- | :--- | :--- | :--- |
| `Visor-Setup-v0.4.3-x64.exe` | 7.896.542 bytes | `07c6d8f67d57b4483bbcd7fe3146bd16dc58b33a6a309b99f6b23555a09c5884` | **SIM (100%)** |
| `Visor-Portable-v0.4.3-x64.zip` | 8.775.785 bytes | `98e9f51b55b15f2ea89dc96667c766ba5deb123fbc08b13add7cee6533f5cf76` | **SIM (100%)** |

### 5.3. Inspeção do Conteúdo da Versão Portátil
O arquivo compactado `Visor-Portable-v0.4.3-x64.zip` foi descompactado e auditado:
- Arquivos presentes:
  * `visor.exe` (21.561.856 bytes, SHA-256: `1B795072FCF930F447993E58632762ECA8B4CFB0C34C2454215D8977AE9B6505`)
  * `README.txt` (6.536 bytes)
  * `LEIAME.txt` (6.682 bytes)
  * `CHANGELOG.txt` (14.287 bytes)
- Validação de higiene: Zero arquivos de depuração (`.pdb`), zero dependências dinâmicas ausentes no diretório, zero metadados Git.
- Cabeçalho PE: `MZ / PE64` verificado com sucesso.

---

## 6. Homologação do Instalador e Desinstalador

O instalador `Visor-Setup-v0.4.3-x64.exe` gerado pelo pipeline de CI foi testado em ambiente de execução local:

1. **Instalação Silenciosa:**
   ```powershell
   Visor-Setup-v0.4.3-x64.exe /DIR="<temp_dir>" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-
   ```
   - **ExitCode:** `0` (Sucesso imediato).
   - **Arquivos Criados:**
     * `visor.exe` (SHA-256 idêntico ao binário portátil: `1B7950...`)
     * `CHANGELOG.md`
     * `LEIAME.txt`
     * `unins000.exe` e `unins000.dat`
2. **Desinstalação Silenciosa:**
   ```powershell
   unins000.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
   ```
   - **ExitCode:** `0` (Sucesso).
   - **Resultado:** Remoção completa e limpa dos binários e arquivos da aplicação.

---

## 7. Status de Homologação em Clean VM

Em estrita conformidade com os requisitos de auditoria pré-lançamento do VISOR:

> **Critério de Auditoria:** "Se o seu ambiente não tiver uma VM limpa real ou Windows Sandbox funcional, declare o teste da VM limpa como: NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM. Nunca declare como aprovado algo que rodou na máquina de desenvolvimento."

- **Diagnóstico:** O binário `WindowsSandbox.exe` não está habilitado/disponível no host de desenvolvimento, e não há hipervisor configurado para criação de máquina descartável durante esta sessão.
- **Classificação Formal:** **NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM**.
- **Observação Técnica:** A homologação funcional de instalação, desinstalação e integridade de binário foi aprovada no host Windows de teste, mas a validação isolada em SO Windows virgem sem VC++ Redistributable pré-existente permanece pendente para o teste final de pré-release pelo operador em máquina isolada.

---

## 8. Auditoria de Compatibilidade do Update Center

O módulo interno de atualização do VISOR (`src/updater/`) foi auditado contra os padrões do pipeline de CI:

| Requisito do Atualizador | Implementação no Código | Validação no Pipeline de Distribuição | Status |
| :--- | :--- | :--- | :--- |
| **Endpoint Oficial** | `https://api.github.com/repos/Tsujiu/visor-downloads/releases/latest` (`github.rs:10`) | O workflow publica exatamente no repositório `Tsujiu/visor-downloads` | **COMPATÍVEL** |
| **Padrão de Tag** | Suporte a `vX.Y.Z` via `clean_version_str()` (`version.rs:9`) | O workflow gera releases com tags prefixadas com `v` (ex: `v0.4.3`) | **COMPATÍVEL** |
| **Nome do Instalador** | `find_installer_asset()` busca `visor-setup-*.exe` (`github.rs:108`) | O workflow gera `Visor-Setup-v<version>-x64.exe` | **COMPATÍVEL** |
| **Nome da Versão Portátil** | `find_portable_asset()` busca `visor-portable-*.zip` (`github.rs:119`) | O workflow gera `Visor-Portable-v<version>-x64.zip` | **COMPATÍVEL** |
| **Manifesto de Integridade** | `find_manifest_asset()` busca `SHA256SUMS.txt` (`github.rs:130`) | O workflow gera `SHA256SUMS.txt` | **COMPATÍVEL** |
| **Formato das Somas SHA-256** | `parse_sha256sums()` aceita formato GNU `<hash>  <filename>` (`integrity.rs:19`) | O workflow produz hashes no formato GNU exato com 2 espaços | **COMPATÍVEL** |
| **Algoritmo Criptográfico** | SHA-256 via crate `sha2` (`integrity.rs:8`) | O workflow utiliza `Get-FileHash -Algorithm SHA256` | **COMPATÍVEL** |

---

## 9. Conclusão da Homologação Pós-Merge

1. **O pipeline passou no Dry-Run?**  
   **SIM.** A execução `42736430116` completou todas as 14 etapas de teste, compilação, empacotamento e cálculo criptográfico com 100% de sucesso em 23m 30s.

2. **Os artefatos correspondem ao build testado?**  
   **SIM.** Os arquivos gerados coincidem perfeitamente com os hashes registrados no `SHA256SUMS.txt`.

3. **O instalador passou na Clean VM?**  
   **NÃO VERIFICADO / PENDENTE DE TESTE EM CLEAN VM.** O instalador passou em modo silencioso com saída limpa no host local, mas não foi testado em VM virgem/descartável por indisponibilidade do Windows Sandbox no ambiente.

4. **A versão portátil passou?**  
   **SIM.** O arquivo ZIP foi extraído, validado e o executável `visor.exe` possui cabeçalho Windows PE64 válido e sem arquivos residuais de compilação.

5. **O atualizador está compatível com os metadados?**  
   **SIM.** Todos os critérios de endpoint, formato de tags, nomes de assets e validação SHA-256 estão em perfeita consonância.

6. **O VISOR está tecnicamente pronto para sua primeira Release oficial?**  
   **SIM.** A branch `main` está estabilizada e testada, a concorrência e integridade do pipeline estão comprovadas e as proteções de publicação estão ativas. Para o lançamento oficial, basta sincronizar o version bump (`0.4.4`), gerar a tag e disparar o workflow com `dry_run = false` após validação visual na Clean VM.
