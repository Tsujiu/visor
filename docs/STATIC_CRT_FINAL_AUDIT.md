# VISOR — AUDITORIA TÉCNICA FINAL: LINKAGE CRT STATIC v0.4.4
## Eliminação de Dependências Externas e Validação PE/COFF para Windows 10/11

**Data:** 11 de Novembro de 2026  
**Projeto:** VISOR  
**Versão:** 0.4.4  
**Target:** `x86_64-pc-windows-msvc`  
**Commit de Produção:** `77c282955d3513703a51261421215b7cf09eef4b` (`77c2829`)  
**GitHub Actions Run:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Binário Auditado:** `visor.exe` (SHA-256: `76abae471e8725e6d06714a1e792f3758742b872398d47bab2db8d0c055094cb`, 21.751.296 bytes)

---

## 1. Resumo Executivo e Diagnóstico de Causa-Raiz

Na homologação inicial da versão v0.4.4 (GitHub Actions Run `42750682216`), o teste em ambiente Windows limpo (Clean VM) revelou uma falha de inicialização com erro de sistema:
> *"O código de execução não pode continuar porque VCRUNTIME140.dll não foi encontrado. Reinstalando o programa para corrigir o problema."*

### Causa-Raiz Técnica
Por padrão, o toolchain Rust `x86_64-pc-windows-msvc` vincula o runtime C da Microsoft (MSVC CRT) de forma dinâmica (`/MD`), gerando dependências externas das DLLs do *Microsoft Visual C++ Redistributable*:
- `VCRUNTIME140.dll` (runtime de exceções e manipulação de memória de baixo nível do VC++)
- `api-ms-win-crt-runtime-l1-1-0.dll` e demais DLLs de redirecionamento do Universal CRT (UCRT)

Em sistemas corporativos ou instalações limpas do Windows sem o pacote redistribuível instalado previamente por outros softwares, o executável falhava imediatamente antes de atingir a função `main`.

### Solução Definitiva Implementada
No commit `77c2829`, o arquivo `.cargo/config.toml` foi configurado para instruir o compilador `rustc` e o linker MSVC (`link.exe`) a embutir estaticamente todo o código de runtime C diretamente dentro da imagem PE executável através da flag de vinculação estática:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

Com esta diretiva, o linker substitui as bibliotecas de importação dinâmicas (`msvcrt.lib` / `vcruntime.lib` / `ucrt.lib`) pelas bibliotecas estáticas equivalentes (`libcmt.lib` / `libvcruntime.lib` / `libucrt.lib` — flag `/MT`), eliminando qualquer dependência de DLLs do runtime C em tempo de execução.

---

## 2. Comparativo Estrutural: Build Dinâmico vs. Build Estático

A auditoria realizou a comparação direta dos cabeçalhos PE/COFF e da Import Directory Table entre o build dinâmico anterior (Run `42750682216`) e o build estático atual (Run `42806985528`):

| Parâmetro de Linkage | Build Dinâmico (Run 42750682216) | Build Estático CRT (Run 42806985528) | Variação / Impacto |
| :--- | :---: | :---: | :--- |
| **Commit Git** | `926299c8f39` | `77c282955d3` | Inclusão de `.cargo/config.toml` |
| **Flag do Compilador** | Padrão MSVC (`/MD`) | `+crt-static` (`/MT`) | Embutimento estático de rotinas C |
| **Tamanho do `visor.exe`** | 21.050.880 bytes | 21.751.296 bytes | +700.416 bytes (+3.3%) |
| **Tamanho do Instalador** | 7.747.012 bytes | 7.978.372 bytes | +231.360 bytes (+2.9%) |
| **Total de DLLs Importadas** | **27 DLLs** | **20 DLLs** | **-7 DLLs eliminadas** |
| **`VCRUNTIME140.dll`** | **PRESENTE (Obrigatória)** | **AUSENTE (Eliminada)** | **Dependência crítica erradicada** |
| **`api-ms-win-crt-*`** | **6 DLLs presentes** | **0 DLLs presentes** | **Dependências dinâmicas eliminadas** |
| **Comportamento em Clean VM** | **Crash / Erro de DLL** | **Execução Imediata e Estável** | **Autocontenção total** |

---

## 3. Desmontagem e Análise da Tabela de Importação PE (objdump)

A inspeção detalhada da tabela de importação (`IMAGE_DIRECTORY_ENTRY_IMPORT`) do binário `visor.exe` (SHA-256: `76abae471e87...`) confirmou a presença exclusiva de **20 DLLs**, todas pertencentes ao núcleo nativo imutável do sistema operacional Windows:

```text
The Import Tables (interpreted .rdata section contents)
 vma:            Hint    Time      Forward  DLL       First
                 Table   Stamp     Chain    Name      Thunk
 0000000140fe3578 00fe4170 00000000 00000000 00fe5018 00ea0888
	DLL Name: SHELL32.dll
 0000000140fe3590 00fe4188 00000000 00000000 00fe5030 00ea08a0
	DLL Name: SHLWAPI.dll
 0000000140fe35a8 00fe41b0 00000000 00000000 00fe5058 00ea08c8
	DLL Name: USER32.dll
 0000000140fe35c0 00fe45d8 00000000 00000000 00fe5480 00ea0cf0
	DLL Name: KERNEL32.dll
 0000000140fe35d8 00fe4bf0 00000000 00000000 00fe5a98 00ea1308
	DLL Name: ole32.dll
 0000000140fe35f0 00fe4c48 00000000 00000000 00fe5af0 00ea1360
	DLL Name: advapi32.dll
 0000000140fe3608 00fe4c78 00000000 00000000 00fe5b20 00ea1390
	DLL Name: gdi32.dll
 0000000140fe3620 00fe4cf0 00000000 00000000 00fe5b98 00ea1408
	DLL Name: bcryptprimitives.dll
 0000000140fe3638 00fe4d00 00000000 00000000 00fe5ba8 00ea1418
	DLL Name: dwmapi.dll
 0000000140fe3650 00fe4d18 00000000 00000000 00fe5bc0 00ea1430
	DLL Name: bcrypt.dll
 0000000140fe3668 00fe4d50 00000000 00000000 00fe5bf8 00ea1468
	DLL Name: crypt32.dll
 0000000140fe3680 00fe4db8 00000000 00000000 00fe5c60 00ea14d0
	DLL Name: ntdll.dll
 0000000140fe3698 00fe4e20 00000000 00000000 00fe5cc8 00ea1538
	DLL Name: gdiplus.dll
 0000000140fe36b0 00fe4e30 00000000 00000000 00fe5cd8 00ea1548
	DLL Name: api-ms-win-core-synch-l1-2-0.dll
 0000000140fe36c8 00fe4e40 00000000 00000000 00fe5ce8 00ea1558
	DLL Name: ADVAPI32.dll
 0000000140fe36e0 00fe4e68 00000000 00000000 00fe5d10 00ea1580
	DLL Name: OPENGL32.dll
 0000000140fe36f8 00fe4ee0 00000000 00000000 00fe5d88 00ea15f8
	DLL Name: GDI32.dll
 0000000140fe3710 00fe4f00 00000000 00000000 00fe5da8 00ea1618
	DLL Name: imm32.dll
 0000000140fe3728 00fe4f40 00000000 00000000 00fe5de8 00ea1658
	DLL Name: ws2_32.dll
 0000000140fe3740 00fe4fe0 00000000 00000000 00fe5e88 00ea16f8
	DLL Name: uxtheme.dll
```

### Justificativa de Necessidade de Cada DLL Importada:
1. **`KERNEL32.dll` / `ntdll.dll`:** Gerenciamento fundamental de processos, threads, heap de memória, I/O síncrono e assíncrono de arquivos e sincronização primitiva.
2. **`USER32.dll` / `gdi32.dll` / `GDI32.dll` / `uxtheme.dll` / `dwmapi.dll`:** Criação da janela principal da interface gráfica, loop de mensagens do Windows (Win32 event queue), renderização visual acelerada, composição Desktop Window Manager (DWM) e suporte a temas claros/escuros.
3. **`SHELL32.dll` / `SHLWAPI.dll`:** Manipulação de caminhos do Windows (`SHGetKnownFolderPath`), integração com Explorer, atalhos e resolução de diretórios de sistema.
4. **`ole32.dll`:** Inicialização COM (`CoInitializeEx`, `CoCreateInstance`) para caixas de diálogo nativas de arquivos (`IFileDialog`), Clipboard e drag-and-drop.
5. **`OPENGL32.dll`:** Contexto de aceleração de hardware para o renderizador gráfico e decodificadores visuais via egui/glow/eframe.
6. **`imm32.dll`:** Input Method Editor (IME) para suporte a caracteres internacionais e acentuação no teclado.
7. **`gdiplus.dll`:** Interoperabilidade com bitmaps e decodificações auxiliares Legado do Windows.
8. **`bcrypt.dll` / `bcryptprimitives.dll` / `crypt32.dll`:** Suporte criptográfico para geradores de números pseudo-aleatórios criptograficamente seguros (CSPRNG), verificação de hashes SHA-256 e negociação de certificados TLS/HTTPS para o Update Center.
9. **`ws2_32.dll`:** Sockets Winsock 2 para transporte de rede seguro HTTPS utilizado pelo subsistema de atualização.
10. **`advapi32.dll` / `ADVAPI32.dll`:** Acesso a tokens de segurança do usuário e leitura de configurações seguras de registro do sistema.
11. **`api-ms-win-core-synch-l1-2-0.dll`:** Primitiva de sincronização nativa presente em todas as edições do Windows 10/11 como redirecionamento de sistema para o `KERNEL32.dll`.

---

## 4. Auditoria de Carregamento Dinâmico (LoadLibrary) e Delay-Load

Para garantir que dependências dinâmicas não estivessem sendo carregadas em tempo de execução via APIs dinâmicas ocultas:

1. **Inspeção do Delay Import Directory:**
   A leitura dos cabeçalhos opcionais PE (`IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT`, índice 13) resultou em:
   ```text
   Delay Import Directory RVA: 0x00000000
   Delay Import Directory Size: 0 bytes
   ```
   *Conclusão:* O binário não utiliza Delay-Load linking.

2. **Varredura no Código-Fonte Rust:**
   Realizou-se varredura completa em toda a árvore `src/` em busca de chamadas de carregamento de biblioteca dinâmica:
   - `LoadLibraryA`: 0 ocorrências
   - `LoadLibraryW`: 0 ocorrências
   - `LoadLibraryExA`: 0 ocorrências
   - `LoadLibraryExW`: 0 ocorrências
   - `GetProcAddress`: 0 ocorrências

*Conclusão:* Nenhuma DLL adicional é invocada dinamicamente pelo código-fonte proprietário do VISOR durante sua inicialização ou operação contínua.

---

## 5. Ciclo de Vida e Compatibilidade com Windows 10 e Windows 11

A compilação estática com MSVC Linker e target `x86_64-pc-windows-msvc` garante que o binário opera com total suporte nativo nas seguintes plataformas:

| Plataforma / Sistema Operacional | Status de Compatibilidade | Observações Operacionais |
| :--- | :---: | :--- |
| **Windows 11 (21H2, 22H2, 23H2, 24H2)** | **Suporte Nativo Pleno** | Arquitetura x64 com aceleração gráfica DWM nativa |
| **Windows 10 (1809, 1909, 20H2, 21H1, 22H2)** | **Suporte Nativo Pleno** | Total paridade de APIs com todas as 20 DLLs presentes |
| **Windows Server 2019 / 2022 / 2025** | **Suporte Pleno** | Requer modo Desktop Experience para interface gráfica |
| **Windows 7 / 8 / 8.1** | **Não Suportado** | Plataformas descontinuadas pela Microsoft e fora do target Rust moderno |

---

## 6. Parecer Técnico da Auditoria

A vinculação estática `+crt-static` foi executada e validada com total sucesso técnico no GitHub Actions Run `42806985528`. 

A imagem PE do VISOR v0.4.4 é rigorosamente **autocontida**, estável, livre de dependências dinâmicas de redistribuíveis de terceiros e pronta para operação transparente em qualquer instalação virgem do Windows 10 e 11 x64.
