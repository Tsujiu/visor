# VISOR — FASE 4: DIAGNÓSTICO DO COMPUTADOR REAL DO OPERADOR
## Inventário do Ambiente Físico de Homologação Windows 11 x64

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Candidata:** 0.4.4  
**Commit de Produção:** `77c282955d3513703a51261421215b7cf09eef4b`  
**GitHub Actions Run:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Ambiente de Execução:** Computador Real do Operador (Windows 11 Físico — Sem VM, Sem Sandbox)

---

## 1. Princípio Fundamental de Homologação Local

Em conformidade estrita com a governança da Fase 4, **nenhum ambiente virtualizado (Virtual Machine, Hyper-V, VirtualBox, VMware) ou Windows Sandbox foi utilizado**. Todos os diagnósticos e execuções funcionais desta fase foram realizados diretamente no hardware físico do operador, garantindo a fidelidade absoluta da experiência de usuário final.

---

## 2. Inventário do Sistema Operacional e Hardware

As informações abaixo foram coletadas utilizando exclusivamente comandos nativos do PowerShell (`Get-CimInstance`, `Get-PSDrive`, `Get-Process`):

| Parâmetro de Sistema | Valor Diagnosticado | Avaliação Técnica |
| :--- | :--- | :--- |
| **Sistema Operacional** | Microsoft Windows 11 Pro | Edição comercial padrão de uso profissional |
| **Versão do Sistema** | 10.0.26300 | Plataforma Windows moderna atualizada |
| **Build do Windows** | 26300 | Windows 11 versão 24H2 / Insider Preview |
| **Arquitetura do SO** | 64 bits (`x86_64` / AMD64) | Compatível com o target `x86_64-pc-windows-msvc` |
| **Processador (CPU)** | AMD Ryzen 5 5500 | 6 núcleos físicos, 12 processadores lógicos, arquitetura 9 (x64) |
| **Espaço Livre no Disco C:\** | ~659 GB livres (de 1.023 GB) | Amplo espaço disponível para descompactação e testes |
| **Espaço Livre no Disco D:\** | ~824 GB livres (de 1.000 GB) | Volume secundário de armazenamento disponível |
| **Identidade do Usuário** | `MUTSUJII\bruno` | Sessão de usuário padrão interativa |
| **Nível de Privilégios** | **Não-Elevado (`IsAdministrator: False`)** | **Ambiente ideal para homologar instalação não-elevada** |

---

## 3. Estado das Instalações e Dados Existentes do VISOR

O diagnóstico inspecionou os diretórios padrão do usuário para identificar instalações prévias e evitar perda acidental de configurações:

### 3.1. Instalação Atual em Disco
- **Localização:** `C:\Users\bruno\AppData\Local\Programs\Visor\`
- **Status:** **Instalação existente detectada.**
- **Conteúdo Identificado:**
  - `visor.exe` (21.561.856 bytes, FileVersion: 0.4.4, SHA-256: `0b736daf318ce1028146ee6bd4d70400b0bd36e120a89176791288880b85b393`)
  - `unins000.exe` (4.442.388 bytes)
  - `unins000.dat` (6.113 bytes)
  - `CHANGELOG.md` (15.874 bytes)
  - `LEIAME.txt` (6.682 bytes)
- **Processos Ativos em Execução:** 0 processos com nome `visor` ativos no início da homologação.

### 3.2. Configurações de Usuário Existentes
- **Localização:** `C:\Users\bruno\AppData\Local\Visor\`
- **Status:** **Diretório e preferências existentes.**
- **Arquivo de Configuração:** `settings.json` (902 bytes, última modificação: 11/11/2026 12:08:43).
- **Diretriz de Segurança:** As configurações e preferências pessoais do operador foram estritamente preservadas, isolando os testes da Fase 4 em diretório independente (`%LOCALAPPDATA%\VISOR-Validation\v0.4.4\`).

---

## 4. Auditoria de Runtimes e Componentes de Sistema

### 4.1. Visual C++ Redistributable
A verificação no registro do Windows (`HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`) identificou:
- `Microsoft Visual C++ 2015-2022 Redistributable (x64) - 14.44.35211.0`
- `Microsoft Visual C++ 2015-2022 Redistributable (x86) - 14.44.35211.0`

> [!NOTE]
> **Preservação de Componentes:**  
> Conforme as regras de segurança do projeto, **o Visual C++ Redistributable NÃO foi desinstalado ou alterado**. A comprovação de que o VISOR v0.4.4 não depende deste pacote apoia-se na auditoria da Tabela de Importações PE/COFF (`objdump -p` e ausência total de imports de `VCRUNTIME140.dll` e `api-ms-win-crt-*`), e não na modificação arriscada de componentes globais do sistema operacional.

### 4.2. Estado do Windows Defender e SmartScreen
A consulta através de `Get-MpComputerStatus` e chaves de registro confirmou:
- **Windows Defender Antivirus:** Ativo (`AntivirusEnabled: True`, Versão das assinaturas: `1.463.242.0`).
- **SmartScreen do Sistema:** Operando sob as políticas padrão do Windows 11.
- **Diretriz de Segurança:** O Windows Defender e o SmartScreen foram mantidos integralmente ativos. Nenhum mecanismo de segurança foi desabilitado durante a homologação.
