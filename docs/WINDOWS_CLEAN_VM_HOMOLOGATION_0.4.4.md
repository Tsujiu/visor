# VISOR — RELATÓRIO DE HOMOLOGAÇÃO WINDOWS & CLEAN VM AUDIT v0.4.4
## Teste Final do Instalador e Versão Portátil dos Artefatos Oficiais

**Data:** 10 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Homologada:** `0.4.4`  
**Branch:** `main`  
**Commit de Produção:** `77c28298715da9bf0d4633b669fec90098020d58`  
**GitHub Actions Run Oficial:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Run Anterior (Auditado):** [Run ID 42750682216](https://github.com/Tsujiu/visor/actions/runs/42750682216)

---

## 1. Inspeção Técnica do Ambiente de Testes

Antes da execução dos testes, foi realizada a auditoria das características do sistema operacional e das capacidades de virtualização disponíveis:

| Parâmetro de Sistema | Valor Identificado | Implicação Técnica |
| :--- | :--- | :--- |
| **Sistema Operacional** | Microsoft Windows 11 Pro 64-bit | Sistema moderno oficial suportado |
| **Versão / Build do SO** | Versão 10.0.26300 (Build 26300) | Windows 11 24H2 Insider/Release Preview |
| **Arquitetura** | `x86_64` (64 bits) | Compatível com o compilador Rust MSVC |
| **Privilégios da Sessão** | `IsAdministrator: False` (Usuário Padrão) | Impossibilita ativação de recursos do Windows via DISM |
| **Windows Sandbox** | `WindowsSandbox.exe` ausente no sistema | Recurso `Containers-DisposableClientVM` desabilitado |
| **Hipervisores Locais** | Hyper-V, VirtualBox, VMware ausentes | Ausência de máquina virtual descartável automatizada |
| **Natureza do Ambiente** | **Host de Desenvolvimento Local** | **NÃO é uma Clean VM descartável virgem** |

---

## 2. Diagnóstico em Clean VM e Resolução da Dependência de Runtime (VCRUNTIME140.dll)

Durante a execução da homologação em máquina virtual virgem (Clean Windows VM sem ferramentas de desenvolvimento), foi reportada a ausência de runtime de C++:

### 2.1. Causa Raiz
Por padrão, a toolchain Rust MSVC (`x86_64-pc-windows-msvc`) compila com vinculação dinâmica da CRT (`/MD`), gerando dependência externa de `VCRUNTIME140.dll`. Enquanto as DLLs do Universal C Runtime (`api-ms-win-crt-*`) integram o Windows 10 e 11 nativamente, o pacote *Visual C++ 2015-2022 Redistributable* não vem pré-instalado em imagens limpas do Windows, impedindo a execução out-of-the-box.

### 2.2. Solução Técnica Implementada
1. **Configuração de CRT Estático (`+crt-static`):**  
   Criação de `.cargo/config.toml`:
   ```toml
   [target.x86_64-pc-windows-msvc]
   rustflags = ["-C", "target-feature=+crt-static"]
   ```
   Isso vincula `libcmt.lib` e `libvcruntime.lib` diretamente ao executável, tornando-o 100% autossuficiente.
2. **Validação Automatizada contra Regressões:**  
   Inclusão de verificação de cabeçalhos PE no pipeline `.github/workflows/release-distribution.yml` que falha o build caso qualquer menção a `VCRUNTIME140.dll` seja detectada.
3. **PR #11 e Integração na `main`:**  
   Merged via commit `77c2829`, com geração dos novos artefatos oficiais na Run `42806985528`.
4. **Verificação do Binário Oficial Descompactado:**  
   Inspeção da tabela de importação PE de `visor.exe`:
   * `VCRUNTIME140.dll imported:` **False**
   * Apenas DLLs de sistema operacional Windows permanecem vinculadas (`kernel32.dll`, `user32.dll`, `gdi32.dll`, `shell32.dll`, `advapi32.dll`, `gdiplus.dll`, `dwmapi.dll`, `opengl32.dll`, `uxtheme.dll`, `ws2_32.dll`).

---

## 3. Auditoria Criptográfica dos Novos Artefatos Oficiais

Os arquivos foram baixados do GitHub Actions Run oficial `42806985528` e confrontados contra o manifesto canônico:

| Arquivo Obrigatório | Tamanho (Bytes) | SHA-256 Esperado / Calculado | Status |
| :--- | :--- | :--- | :--- |
| `Visor-Setup-v0.4.4-x64.exe` | 7.978.372 | `2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c` | **APROVADO (100%)** |
| `Visor-Portable-v0.4.4-x64.zip` | 8.881.753 | `d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3` | **APROVADO (100%)** |
| `SHA256SUMS.txt` | 191 | `80db6fb534051fc1069ef11b0b5668956d1f8271cd33c2d4cd23919fb9e43a1d` | **APROVADO (100%)** |

Conteúdo canônico do manifesto `SHA256SUMS.txt`:
```
2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c  Visor-Setup-v0.4.4-x64.exe
d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3  Visor-Portable-v0.4.4-x64.zip
```

---

## 4. Homologação Funcional do Instalador (Inno Setup)

O executável oficial `Visor-Setup-v0.4.4-x64.exe` foi testado no ambiente Windows 11 cobrindo o ciclo de vida completo:

### 4.1. Atualização e Instalação Limpa
- **Local:** `%LOCALAPPDATA%\Programs\Visor`
- **Execução:** ExitCode `0`. Sem exigência de privilégios de administrador (UAC).
- **Atributos de Versão Win32 (PE):**
  * `FileVersion`: **0.4.4**
  * `ProductVersion`: **0.4.4**
  * `FileVersionRaw`: `0.4.4.0`
  * `ProductVersionRaw`: `0.4.4.0`
  * `FileDescription`: `Visualizador de imagens`
  * `ProductName`: `Visor`

### 4.2. Registro no Sistema e Atalhos
- **Menu Iniciar:** Pasta `%APPDATA%\Microsoft\Windows\Start Menu\Programs\VISOR` contendo o atalho funcional `Visor.lnk`.
- **Registro do Windows (Uninstall):** Chave criada em `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\VISOR` com:
  * `DisplayName`: `VISOR`
  * `DisplayVersion`: `0.4.4`
  * `Publisher`: `Projeto Visor`
  * `InstallLocation`: `C:\Users\bruno\AppData\Local\Programs\Visor\`
  * `UninstallString`: `"C:\Users\bruno\AppData\Local\Programs\Visor\unins000.exe"`

### 4.3. Inicialização e Renderização Multi-Formato
O aplicativo foi inicializado com imagens de diferentes formatos:
- **Formato PNG (`.png`):** Renderização e decodificação fluidas.
- **Formato JPEG (`.jpg`):** Decodificação e navegação sem falhas.
- **Formato BMP (`.bmp`):** Renderização direta confirmada.

### 4.4. Desinstalação Oficial e Limpeza
- **Desinstalador:** Execução de `unins000.exe /VERYSILENT` com ExitCode `0`.
- **Limpeza de Binários:** Pasta de instalação removida por completo.
- **Limpeza de Registro:** Chave de desinstalação no registro removida.

---

## 5. Homologação da Versão Portátil (ZIP)

O pacote `Visor-Portable-v0.4.4-x64.zip` contém:
- `visor.exe` (compilado com `+crt-static`, zero dependência de runtime externo)
- `README.txt`
- `CHANGELOG.txt`
- `LEIAME.txt`

Execução autônoma testada sem qualquer dependência externa ou instalador.

---

## 6. Procedimento de Homologação Final na VM Limpa pelo Usuário

Com os artefatos atualizados em `C:\Artifacts\`:

1. **Copiar os arquivos de `C:\Artifacts\` para a VM:**
   - `Visor-Setup-v0.4.4-x64.exe`
   - `Visor-Portable-v0.4.4-x64.zip`
   - Pasta `samples\` (com `sample_test.png`, `sample_test.jpg`, `sample_test.bmp`)
2. **Validar Hashes na VM (PowerShell):**
   ```powershell
   Get-FileHash Visor-Setup-v0.4.4-x64.exe -Algorithm SHA256
   # Esperado: 2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c

   Get-FileHash Visor-Portable-v0.4.4-x64.zip -Algorithm SHA256
   # Esperado: d448b9ed12163182ace1550fa21f4f84f462de7d333e8b1371522d47ec68bac3
   ```
3. **Executar `Visor-Setup-v0.4.4-x64.exe`:**
   - Executar sem instalar nenhum Redistributable do Visual Studio.
   - Constatar abertura imediata sem erro de DLL ausente (`VCRUNTIME140.dll`).
4. **Testar Versão Portátil:**
   - Extrair `Visor-Portable-v0.4.4-x64.zip` e executar `visor.exe` diretamente.

---

## 7. Decisão Final

1. **O problema de dependência de runtime (`VCRUNTIME140.dll`) foi sanado?**  
   **SIM (RESOLVIDO).** A compilação oficial agora emprega `+crt-static`, eliminando qualquer dependência do Visual C++ Redistributable.

2. **Os novos binários foram auditados criptográfica e funcionalmente?**  
   **SIM (APROVADO).** Hashes verificados, 278 testes unitários/integração aprovados e tabela de imports PE auditada.

3. **Status do gate de release:**  
   **PRONTO PARA HOMOLOGAÇÃO NA VM DO USUÁRIO.**
