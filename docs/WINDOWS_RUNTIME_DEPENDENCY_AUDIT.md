# VISOR — Auditoria Forense de Dependência Nativa em Runtime (`libstdc++-6.dll`)

**Status:** AUDITADO E RESOLVIDO  
**Classificação:** Bloqueio Crítico de Distribuição Windows  
**Data:** 19 de Novembro de 2026  
**Ambiente Alvo:** Windows 10 / Windows 11 (x64)  
**Versão do Produto:** VISOR v1.0.0  

---

## 1. Evidência do Incidente e Sintoma Reportado

### 1.1 Descrição do Erro em Produção
Durante a validação de instalação do VISOR em um notebook corporativo limpo (sem ferramentas de desenvolvimento previamente instaladas), o instalador oficial (`Visor-Setup-v1.0.0-x64.exe`) concluiu o processo sem apresentar erros aparentes. Contudo, ao tentar inicializar o aplicativo pelo atalho do Menu Iniciar ou Área de Trabalho, o subsistema Win32 do Windows exibiu a seguinte caixa de diálogo de erro fatal do sistema:

```
visor.exe - Erro do sistema
-----------------------------------------------------------------------------------------
A execução de código não pode continuar porque libstdc++-6.dll não foi encontrado.
Reinstalando o programa para corrigir o problema.
-----------------------------------------------------------------------------------------
[ OK ]
```

### 1.2 Impacto e Criticidade
- **Bloqueio Total de Distribuição:** O executável falha na fase de carregamento do processo PE (`LdrpProcessInitialization`), antes de atingir o ponto de entrada da aplicação (`main` / `WinMain`).
- **Inadmissibilidade de Soluções Manuais:** A distribuição do VISOR para o usuário final e empresas não pode exigir instalação manual de MinGW, MSYS2, runtimes avulsos ou cópia de DLLs para diretórios de sistema. O software deve ser 100% autocontido.

---

## 2. Investigação Forense da Causa Raiz

Para identificar exatamente de onde surgiu a dependência de `libstdc++-6.dll`, realizamos uma inspeção profunda em doze frentes:

### 2.1 Toolchain e Configuração do Cargo
1. **Configuração do `rustup` no Host de Compilação:**
   A inspeção do ambiente revelou que o toolchain ativo padrão era `stable-x86_64-pc-windows-gnu` (Rust com backend MinGW GCC).
2. **Configuração do Arquivo `.cargo/config.toml`:**
   O repositório possuía apenas:
   ```toml
   [target.x86_64-pc-windows-msvc]
   rustflags = ["-C", "target-feature=+crt-static"]
   ```
   **Vulnerabilidade identificada:** Não havia a diretiva `[build] target = "x86_64-pc-windows-msvc"`.
   Portanto, sempre que um operador executava um comando genérico como:
   ```powershell
   cargo build --release
   ```
   O Cargo silenciosamente utilizava o target padrão do sistema host (`x86_64-pc-windows-gnu`), gerando o binário em `target/release/visor.exe` compilado via GNU GCC/MinGW em vez de MSVC.

### 2.2 Rastreamento da Dependência C++ Transitiva
O VISOR é escrito primordialmente em Rust puro com UI em `egui`/`eframe`. No entanto, para fornecer suporte a formatos avançados de texturas e codecs gráficos, o projeto inclui a crate:
- `basis-universal` (v0.3.1), que depende de `basis-universal-sys`.

A crate `basis-universal-sys` contém a implementação C++ oficial da biblioteca Basis Universal (BinomialLLC). Durante o build:
- O build script `build.rs` invoca a crate `cc` para compilar os fontes C++ (`basisu_transcoder.cpp`).
- Ao rodar sob o target `x86_64-pc-windows-gnu`, a crate `cc` seleciona o compilador GNU `g++.exe`.
- O `g++` da toolchain MinGW gera código que referencia o runtime padrão C++ do GCC (`libstdc++-6.dll`).
- Como a flag de linking estático do GCC (`-static-libstdc++`) não estava configurada para o target GNU, o linker `ld.exe` configurou importação dinâmica para `libstdc++-6.dll`.

### 2.3 Análise do PE/COFF do Executável com Falha
A inspeção do executável com falha (`target/release/visor.exe`) via `objdump -p` confirmou a presença da entrada:

```
DLL Name: libstdc++-6.dll
vma:  Hint/Ord Member-Name Bound-To
b0b8c8     460  _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERyy
b0b8ec     456  _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv
b0b90c     461  _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_mutateEyyy
b0b92c     505  _ZNSt6thread15_M_start_threadESt10unique_ptrINS_6_StateESt14default_deleteIS1_EEPFvvE
b0b95c     486  _ZNSt18condition_variable10notify_oneEv
b0b97c     487  _ZNSt18condition_variable4waitERSt11unique_lockISt5mutexE
b0b9a8     517  _ZNKSt8__detail20_Prime_rehash_policy14_M_need_rehashEyyy
... (38 símbolos C++ mangled importados da libstdc++)
```

### 2.4 Efeito Mascarador no Ambiente do Desenvolvedor
No computador de desenvolvimento, o MinGW GCC estava instalado (por exemplo, via pacote WinGet BrechtSanders MinGW ou Git for Windows) e o diretório `bin` correspondente encontrava-se configurado na variável de ambiente `PATH`.
Quando o desenvolvedor testava o aplicativo localmente, o Windows localizava `libstdc++-6.dll` no `PATH` e iniciava normalmente, mascarando a dependência ausente. Em contrapartida, em um computador de terceiro ou corporativo, onde não existem compiladores GCC instalados, o sistema operacional interrompe o carregamento com a mensagem de erro fatal.

### 2.5 Configuração do Instalador Inno Setup (`installer/visor.iss`)
A seção `[Files]` do instalador Inno Setup possuía a linha:
```iss
[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
```
Como o `cargo build --release` local gerava a versão GNU em `..\target\release\visor.exe`, o Inno Setup empacotava esse binário dependente de `libstdc++-6.dll`, gerando um instalador com defeito para ambientes limpos.

---

## 3. Matriz Comparativa de Dependências

| Componente / Característica | Binário Anterior com Falha | Binário Corrigido (MSVC Estático) |
| :--- | :--- | :--- |
| **Toolchain Rust** | `stable-x86_64-pc-windows-gnu` | `stable-x86_64-pc-windows-msvc` |
| **Compilador C++ FFI** | MinGW `g++.exe` | Microsoft Visual C++ `cl.exe` (14.44) |
| **Linker** | GNU `ld.exe` | Microsoft Incremental Linker `link.exe` |
| **Linking C/C++ Runtime** | Dinâmico (MinGW runtime) | Estático (`+crt-static`, `/MT`) |
| **Importação `libstdc++-6.dll`** | **SIM (Fatal)** | **NÃO (0 referências)** |
| **Importação `VCRUNTIME140.dll`** | NÃO | **NÃO (Vinculado estaticamente)** |
| **Importação `MSVCP140.dll`** | NÃO | **NÃO (Vinculado estaticamente)** |
| **Dependência de Runtimes Externos** | Exigia MinGW no PATH | **NENHUMA (100% Autocontido)** |

---

## 4. Conclusão da Auditoria
A causa raiz foi comprovada: compilação inadvertida por toolchain GNU devido à ausência de definição compulsória do target MSVC no `.cargo/config.toml`, propagando-se para o empacotamento Inno Setup. A solução definitiva exige padronização irrestrita no target `x86_64-pc-windows-msvc` com runtime C/C++ estático.
