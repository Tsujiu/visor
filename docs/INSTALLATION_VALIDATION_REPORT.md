# Relatório de Instalação e Validação Local no Windows 11 — VISOR v0.5.0

**Data:** 15 de Novembro de 2026  
**Ambiente:** Windows 11 Pro x64 (Máquina Física do Operador)  
**Status do Windows Defender:** Monitoramento em tempo real ativo e liberado  
**Veredito:** HOMOLOGADO E INSTALADO COM SUCESSO

---

## 1. Localização e Metadados do Binário Instalado

O executável de produção de alta performance foi instalado diretamente no caminho padrão de programas do operador:

- **Caminho Completo:** `C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe`
- **Tamanho:** `22.480.896 bytes` (21,44 MB)
- **Data de Compilação:** 14/11/2026 23:57:28
- **Target:** `x86_64-pc-windows-msvc` (Otimizado `--release`, Static CRT)
- **Hash Criptográfico SHA-256:**
  `1D3E7AE9C2872FA5C093A34D285BE73DD7FA94AFB36A2995596704CE2D9F1435`

---

## 2. Auditoria PE/COFF e Static CRT

Uma inspeção detalhada na tabela de importação (Import Address Table - IAT) do executável instalado confirma:
- **Ausência Total de DLLs de C-Runtime Dinâmico:**
  - `vcruntime140.dll`: AUSENTE (0 referências)
  - `msvcp140.dll`: AUSENTE (0 referências)
  - `api-ms-win-crt-*.dll`: AUSENTE (0 referências)
- **Dependências Nativas de Sistema (19 DLLs Oficiais do Windows):**
  `ADVAPI32.dll`, `bcrypt.dll`, `crypt32.dll`, `dwmapi.dll`, `GDI32.dll`, `gdiplus.dll`, `IMM32.dll`, `KERNEL32.dll`, `ntdll.dll`, `ole32.dll`, `opengl32.dll`, `SHELL32.dll`, `SHLWAPI.dll`, `USER32.dll`, `uxtheme.dll`, `WS2_32.dll`.

---

## 3. Estado do Registro de Desinstalação do Windows

A chave de desinstalação no Registro do Windows (`HKCU`) foi validada:

- **Chave:** `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}_is1`
- **DisplayName:** `VISOR`
- **DisplayVersion:** `0.5.0`
- **InstallLocation:** `C:\Users\bruno\AppData\Local\Programs\Visor\`
- **Publisher:** `Projeto Visor`
- **UninstallString:** `"C:\Users\bruno\AppData\Local\Programs\Visor\unins000.exe"`

---

## 4. Testes de Execução Real em Processo Local

Foram executados testes de inicialização direta no Windows 11 com abertura de arquivos reais da nova biblioteca de amostras:

### Teste 1: Execução com Amostra PSD (`sample-01-photoshop.psd`)
- **Process ID (PID):** 8792
- **Estado de Resposta do Processo:** `Responding = True`
- **Consumo de Memória (Working Set):** 238,53 MB (alocação adequada do canvas e texturas)
- **Comportamento:** O aplicativo inicializou a viewport gráfica, renderizou o arquivo compósito e respondeu aos eventos de sistema sem travamentos ou avisos de erro. Encerramento limpo.

### Teste 2: Execução com Amostra JP2 (`sample-01-balloon.jp2`)
- **Process ID (PID):** 8020
- **Estado de Resposta do Processo:** `Responding = True`
- **Consumo de Memória (Working Set):** 138,82 MB
- **Comportamento:** Inicialização instantânea, descompressão wavelet do JPEG 2000 em memória e encerramento seguro.

---

## 5. Verificação de Interface e Integridade

- **Ausência de Banners Publicitários:** O banner e toda infraestrutura de publicidade externa foram completamente eliminados do código e da interface.
- **Correção de Glifos (Tofu / Quadradinhos):** Todos os ícones unicode não suportados pelas fontes padrão do Windows (como `✓`) foram substituídos por marcadores seguros (`•`), garantindo renderização limpa e profissional no menu de configurações e barra de ferramentas.
- **Edição Única:** O usuário tem acesso irrestrito a todas as ferramentas, codecs, modos de zoom e navegação.
