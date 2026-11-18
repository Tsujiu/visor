# VISOR — RELATÓRIO DE HOMOLOGAÇÃO FINAL NO WINDOWS 11 HOST v0.4.4
## Teste Funcional Completo Sem Máquina Virtual

**Data:** 10 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Sistema Operacional:** Microsoft Windows 11 Pro 64-bit  
**Branch Base:** `main`  
**Commit de Produção Auditado:** `926299c8f396bf07c786fd6707bc2761ffa9cb96`  
**GitHub Actions Run de Origem:** [Run ID 42750682216](https://github.com/Tsujiu/visor/actions/runs/42750682216)  
**Diretório de Artefatos:** `C:\Artifacts\`

> [!IMPORTANT]
> **Classificação Obrigatória de Ambiente:**  
> Esta homologação foi realizada diretamente no **Host Físico Windows 11 de Uso Real**. Em estrito cumprimento às diretrizes de integridade técnica e auditoria de QA, estes resultados são classificados como **TESTE NO HOST WINDOWS 11** e **NUNCA** como Clean VM ou instalação em sistema operacional virgem.

---

## 1. Identificação do Sistema Operacional e Ambiente

| Parâmetro | Valor Identificado | Análise Técnica |
| :--- | :--- | :--- |
| **Sistema Operacional** | Microsoft Windows 11 Pro | Edição profissional moderna compatível |
| **Versão / Build** | Versão 10.0.26300 (Build 26300) | Windows 11 24H2 Insider / Release Preview |
| **Arquitetura** | `64 bits` (`x86_64`) | Compatível com a arquitetura do binário oficial |
| **Segurança Ativa** | Windows Defender e SmartScreen ativos | Sem exceções artificiais ou rebaixamento de proteção |
| **Isolamento** | Host Físico de Desenvolvimento | Ambiente já utilizado, dotado de ferramentas e runtimes pré-instalados |

---

## 2. Identificação do Usuário e Privilégios

| Parâmetro | Valor Identificado | Conformidade |
| :--- | :--- | :--- |
| **Usuário Atual** | `MUTSUJII\bruno` | Conta de usuário padrão do sistema |
| **Privilégios de Administrador** | `IsAdministrator: False` | **Sessão Não-Elevada (Sem UAC)** |
| **Integridade de Execução** | Usuário Padrão | Nenhuma tentativa de elevação de privilégios foi realizada |

---

## 3. Integridade Criptográfica dos Artefatos

Todos os arquivos foram verificados diretamente em `C:\Artifacts\` antes de qualquer execução:

| Arquivo Auditado | Tamanho (Bytes) | SHA-256 Esperado | SHA-256 Recalculado Localmente | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Visor-Setup-v0.4.4-x64.exe` | 7.898.381 | `e937a18d2940e9b6ea5c9e428bca64589744bf309835edef2c7e7c0814009cde` | `e937a18d2940e9b6ea5c9e428bca64589744bf309835edef2c7e7c0814009cde` | **APROVADO (100%)** |
| `Visor-Portable-v0.4.4-x64.zip` | 8.776.440 | `97c5bbaba08a9aa82ec088764a2b10ca2d823ec7c2ecafdbbc7fc732a315de62` | `97c5bbaba08a9aa82ec088764a2b10ca2d823ec7c2ecafdbbc7fc732a315de62` | **APROVADO (100%)** |
| `SHA256SUMS.txt` | 191 | `c571ff6f7c5268e041fb8622242ab9697a3110ae19092f3f348b35b5258a92dc` | `c571ff6f7c5268e041fb8622242ab9697a3110ae19092f3f348b35b5258a92dc` | **APROVADO (100%)** |

Manifesto verificado (`SHA256SUMS.txt`):
```text
e937a18d2940e9b6ea5c9e428bca64589744bf309835edef2c7e7c0814009cde  Visor-Setup-v0.4.4-x64.exe
97c5bbaba08a9aa82ec088764a2b10ca2d823ec7c2ecafdbbc7fc732a315de62  Visor-Portable-v0.4.4-x64.zip
```

---

## 4. Proteção da Instalação e Backup de Dados Existentes

Antes de iniciar qualquer operação de teste no host:
1. **Identificação de Dados Pessoais:**  
   * Arquivo de configuração: `C:\Users\bruno\AppData\Local\Visor\settings.json` (902 bytes, schema_version: 4).
   * Diretório de instalação anterior: `C:\Users\bruno\AppData\Local\Programs\Visor\`.
2. **Backup Realizado:**  
   * Cópia de segurança criada em: `C:\Users\bruno\AppData\Local\VISOR-QA\Backup\settings.json.bak`.
   * Exportação do registro de desinstalação: `C:\Users\bruno\AppData\Local\VISOR-QA\Backup\visor_uninstall.reg`.
3. **Verificação de Processos:** Nenhum processo remanescente em execução antes do início das operações.

---

## 5. Teste Completo da Versão Portátil (ZIP)

O pacote oficial `Visor-Portable-v0.4.4-x64.zip` foi extraído em `%LOCALAPPDATA%\VISOR-QA\Portable`:
* Arquivos conferidos: `visor.exe` (21.561.856 bytes, SHA256: `0b736daf...`), `README.txt`, `CHANGELOG.txt`, `LEIAME.txt`.
* Atributos de Versão PE: `FileVersion: 0.4.4`, `ProductVersion: 0.4.4`, `FileDescription: Visualizador de imagens`.

### Bateria de Cenários Executados:

| Cenário de Teste | Parâmetro / Imagem | PID | Memória (WS) | HWND | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Inicialização sem Argumentos** | N/A (Bare Launch) | 12348 | 225,77 MB | 197688 | **APROVADO** |
| **2. Abertura de Imagem PNG** | `sample_test.png` | 18080 | 250,97 MB | 131936 | **APROVADO** |
| **3. Abertura de Imagem JPG/JPEG** | `sample_test.jpg` | 6228 | 107,98 MB | 197472 | **APROVADO** |
| **4. Abertura de Imagem BMP** | `sample_test.bmp` | 13924 | 108,20 MB | 263008 | **APROVADO** |
| **5. Caminhos com Espaços e Acentos** | `teste com espaços & acentuação (versão 0.4.4).png` | 12296 | 108,58 MB | 328544 | **APROVADO** |
| **6. Zoom, Rotação e Navegação** | `sample_test.png` (via mensagens Win32) | 8316 | 111,91 MB | 590938 | **APROVADO** |
| **7. Redimensionamento e Tela Cheia** | `ShowWindow(SW_MAXIMIZE)` e `VK_F11` | 15304 | 114,72 MB | 525368 | **APROVADO** |
| **8. Tratamento de Arquivo Corrompido** | `corrupted_file.png` (bytes truncados) | 9972 | 102,33 MB | 590904 | **APROVADO** |
| **9. Fechamento e Reabertura** | `sample_test.png` | 12476 | 108,11 MB | 263440 | **APROVADO** |

**Observação de Robustez:** No cenário 8, o visualizador lidou com o arquivo corrompido de forma graciosa, mantendo a janela estável e sem gerar crashes, corrupção de memória ou exceções não tratadas.

---

## 6. Teste do Instalador Inno Setup

Inspecionou-se `installer/visor.iss`, confirmando as diretivas `PrivilegesRequired=lowest` e `DefaultDirName={localappdata}\Programs\Visor`.

### 6.1. Execução do Instalador
* **Executável:** `C:\Artifacts\Visor-Setup-v0.4.4-x64.exe`
* **Parâmetros:** `/SILENT /SUPPRESSMSGBOXES /TASKS="registerassoc"`
* **ExitCode:** `0` (Sucesso absoluto).
* **Ausência de UAC:** Nenhuma solicitação de privilégios de administrador foi exigida pelo sistema.

### 6.2. Estrutura Instalada
* **Destino:** `%LOCALAPPDATA%\Programs\Visor` (`C:\Users\bruno\AppData\Local\Programs\Visor`).
* **Arquivos Gerados:** `visor.exe`, `LEIAME.txt`, `CHANGELOG.md`, `unins000.exe`, `unins000.dat`.
* **Hash SHA-256 do Binário Instalado:** `0b736daf318ce1028146ee6bd4d70400b0bd36e120a89176791288880b85b393` (100% idêntico ao binário portátil oficial).
* **Atalho no Menu Iniciar:** Criado em `%APPDATA%\Microsoft\Windows\Start Menu\Programs\VISOR.lnk`, apontando corretamente para `C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe` com WorkingDirectory configurado.

### 6.3. Execução e Reabertura do Aplicativo Instalado
* **Inicialização:** PID 13928, WorkingSet ~252 MB, HWND 132486.
* **Reabertura:** PID 4760, WorkingSet ~252 MB.
* **Encerramento:** Finalização limpa e descarregamento completo da memória.

---

## 7. Teste de Associações de Arquivos no Windows

Auditou-se a integração nativa com o Windows 11 realizada pelo parâmetro `--registrar-padrao`:

| Chave de Registro / Elemento | Caminho no Registro | Diagnóstico | Status |
| :--- | :--- | :--- | :--- |
| **ProgID Oficial** | `HKCU:\Software\Classes\Visor.Image` | `(default) = "VISOR Image"` | **APROVADO** |
| **Comando de Abertura** | `HKCU:\Software\Classes\Visor.Image\shell\open\command` | `"C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe" "%1"` | **APROVADO** |
| **RegisteredApplications** | `HKCU:\Software\RegisteredApplications` | Entrada `VISOR` apontando para `Software\Visor\Capabilities` | **APROVADO** |
| **Capacidades Declaradas** | `HKCU:\Software\Visor\Capabilities\FileAssociations` | 55 extensões mapeadas para `Visor.Image` (`.png`, `.jpg`, `.bmp`, `.webp`, `.svg`, etc.) | **APROVADO** |
| **OpenWithProgids** | `HKCU:\Software\Classes\.<ext>\OpenWithProgids` | `Visor.Image` registrado nas extensões testadas | **APROVADO** |

> [!NOTE]
> O VISOR foi registrado como candidato oficial nos menus "Abrir com..." e no painel de "Aplicativos Padrão" do Windows 11 sem alterar silenciosamente o `UserChoice`, respeitando as políticas do sistema operacional.

---

## 8. Teste de Desinstalação e Reinstalação

### 8.1. Desinstalação Oficial
* **Desinstalador:** `%LOCALAPPDATA%\Programs\Visor\unins000.exe /VERYSILENT`
* **ExitCode:** `0`.
* **Remoção de Executáveis:** `Test-Path "%LOCALAPPDATA%\Programs\Visor\visor.exe"` retornou `False`.
* **Remoção de Atalhos:** `Test-Path "%APPDATA%\Microsoft\Windows\Start Menu\Programs\VISOR.lnk"` retornou `False`.
* **Preservação de Dados Pessoais:** `Test-Path "%LOCALAPPDATA%\Visor\settings.json"` retornou `True` (dados do usuário intactos).
* **Limpeza do Registro:** Chave de desinstalação `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}_is1` removida com sucesso (`False`).

### 8.2. Reinstalação Limpa
* **Reinstalação:** Executada a partir de `Visor-Setup-v0.4.4-x64.exe` com ExitCode `0`.
* **Restauração:** `visor.exe` e atalho do Menu Iniciar recriados perfeitamente.
* **Execução pós-reinstalação:** Processo PID 16468 ativo e funcional, consumindo ~110 MB de memória, comprovando restauração sem inconsistências.

---

## 9. Auditoria de Dependências de Runtime

A auditoria inspecionou as DLLs estáticas declaradas e os módulos carregados em runtime:

### 9.1. Dependências Declaradas e Módulos em Memória
* **DLLs de Sistema Windows:** `ntdll.dll`, `KERNEL32.DLL`, `USER32.dll`, `GDI32.dll`, `SHELL32.dll`, `ADVAPI32.dll`, `gdiplus.dll`, `dwmapi.dll`, `opengl32.dll`, `uxtheme.dll`, `ws2_32.dll`, `ucrtbase.dll`.
* **Visual C++ Runtime:** O módulo `VCRUNTIME140.dll` foi carregado a partir de `C:\WINDOWS\SYSTEM32\VCRUNTIME140.dll`.

> [!WARNING]
> **Achado Técnico Crucial de QA:**  
> Nos artefatos gerados pelo Run `42750682216`, o compilador vinculou a CRT dinamicamente (`/MD`). No computador de desenvolvimento, o aplicativo funciona perfeitamente porque `VCRUNTIME140.dll` já existe no Windows do usuário. Em uma máquina Windows virgem sem o *Visual C++ Redistributable*, esta compilação falha.  
> *(Este problema foi diagnosticado e sanado no repositório através do PR #11 com a flag `+crt-static`, gerando a Run 42806985528 onde a dependência externa foi eliminada).*

---

## 10. Testes do Atualizador (Update Center)

* **Bateria Automatizada:** Executados os 28 testes unitários e de integração do módulo `updater::` (`cargo test --release --no-default-features updater::`). Todos os 28 testes foram **APROVADOS (100%)**.
* **Validações Cobertas:**
  * Parsing e comparação estrita de SemVer 2.0.
  * Validação criptográfica de hashes em `SHA256SUMS.txt`.
  * Sanitização de nomes de arquivos contra ataques de path traversal.
  * Rejeição de URLs inseguras ou forks não confiáveis.
  * Resiliência a falhas de rede e manifestos malformados.
* **Comportamento em Ambiente Real:** Requisição de teste ao endpoint da API GitHub (`/repos/Tsujiu/visor-downloads/releases/latest`) retornou HTTP 404 (repositório ainda sem releases públicas), tratado com segurança pelo aplicativo sem causar travamentos.
* **Status em Produção:** **NÃO VERIFICADO EM PRODUÇÃO** (condicionado à existência da primeira release oficial no repositório de distribuição).

---

## 11. Quadro Consolidado de Classificação

| Critério Auditado | Classificação | Justificativa Técnica |
| :--- | :--- | :--- |
| **1. Identificação do SO e Privilégios** | **APROVADO** | Windows 11 Pro Build 26300, sessão de usuário padrão (sem UAC) |
| **2. Integridade dos Artefatos Certificados** | **APROVADO** | Hashes SHA-256 de instalador, portátil e manifesto 100% conferidos |
| **3. Backup e Preservação de Dados** | **APROVADO** | Configurações e registros salvos e preservados sem corrupção |
| **4. Versão Portátil ZIP no Host** | **APROVADO** | 9 cenários funcionais executados com estabilidade e baixo consumo |
| **5. Instalador Oficial Inno Setup** | **APROVADO** | Instalação em `%LOCALAPPDATA%\Programs\Visor` sem UAC com ExitCode 0 |
| **6. Atalhos no Menu Iniciar** | **APROVADO** | Atalho funcional criado em `{autoprograms}\VISOR.lnk` |
| **7. Associações de Arquivos e ProgID** | **APROVADO** | `Visor.Image`, `RegisteredApplications` e 55 extensões registradas |
| **8. Desinstalação Oficial** | **APROVADO** | Remoção de binários e atalhos com ExitCode 0 e preservação de configs |
| **9. Reinstalação Limpa** | **APROVADO** | Restauração íntegra de executáveis e funcionamento imediato |
| **10. Auditoria de Dependências** | **APROVADO COM RESSALVAS** | Funciona no host com runtimes existentes; identificada necessidade de CRT estático para Clean VM |
| **11. Testes do Atualizador (Engine)** | **APROVADO** | 28 testes automatizados aprovados; tratamento HTTP 404 certificado |
| **12. Atualização Ponta a Ponta em Produção** | **NÃO VERIFICADO** | Aguarda publicação da primeira release pública oficial |
| **13. Homologação em Clean VM (SO Virgem)** | **NÃO VERIFICADO** | Host de testes já utilizado; não foi executado em ambiente descartável nesta etapa |

---

## 12. DECISÃO FINAL

### A. Homologação Funcional no Windows 11 Host:
**SIM (APROVADO).**  
O instalador oficial, a versão portátil, os atalhos, o desinstalador e o sistema de associações de arquivos funcionaram perfeitamente no ambiente real do Windows 11 Pro 64-bit, sem falhas de interface, sem privilégios administrativos e sem regressões funcionais.

### B. Homologação em Windows Clean VM:
**NÃO VERIFICADO.**  
Esta homologação foi realizada diretamente no computador físico de desenvolvimento. Nenhum ambiente virtual ou Windows Sandbox foi utilizado nesta etapa, mantendo a honestidade técnica estrita de QA.

### C. Recomendação para Primeira Publicação Oficial:
**APROVADA COM RESSALVAS.**  
Os artefatos da versão 0.4.4 da Run `42750682216` estão plenamente homologados para usuários de Windows que possuem runtimes de C++ instalados. Para garantir execução universal imediata em máquinas Windows 10/11 virgens sem nenhuma dependência prévia, a distribuição pública definitiva deve utilizar o pacote com vinculação estática (`+crt-static`) gerado a partir do commit `77c2829` (Run `42806985528`).
