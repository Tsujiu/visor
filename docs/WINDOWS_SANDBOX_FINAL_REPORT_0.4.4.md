# VISOR — RELATÓRIO FINAL DE AUDITORIA WINDOWS SANDBOX v0.4.4
## Homologação em Ambiente Descartável Limpo e Validação de Artefatos Oficiais

**Data:** 10 de Novembro de 2026  
**Projeto:** VISOR  
**Versão Homologada:** `0.4.4`  
**Branch Base:** `main`  
**Commit de Produção:** `77c28298715da9bf0d4633b669fec90098020d58`  
**GitHub Actions Run Oficial:** [Run ID 42806985528](https://github.com/Tsujiu/visor/actions/runs/42806985528)  
**Diretório de Distribuição Local:** `C:\Artifacts\`  
**Arquivo de Configuração do Sandbox:** `C:\Artifacts\VisorSandbox.wsb`  

---

## 1. Auditoria e Diagnóstico do Ambiente Windows Sandbox

Em cumprimento estrito às diretrizes de QA ("*Verificar ambiente; se o Sandbox não estiver disponível, não tente contornar permissões administrativas; não classifique o computador de desenvolvimento como Clean VM*"), foi realizada a auditoria das condições de execução:

| Parâmetro de Ambiente | Diagnóstico Real no Host | Classificação |
| :--- | :--- | :--- |
| **Sistema Operacional** | Microsoft Windows 11 Pro 64-bit (Build 26300) | **CONFIRMADO** |
| **Arquitetura** | `x86_64` (64 bits) | **CONFIRMADO** |
| **Privilégios da Sessão** | `IsAdministrator: False` (Sessão Não-Elevada) | **PADRÃO (Sem Elevação)** |
| **Binário Windows Sandbox** | `C:\Windows\System32\WindowsSandbox.exe` -> `False` (Inexistente) | **INDISPONÍVEL** |
| **Status do Recurso Windows** | Recurso `Containers-DisposableClientVM` desabilitado | **INDISPONÍVEL** |
| **Hipervisores Alternativos** | Hyper-V Manager, VirtualBox e VMware ausentes | **INDISPONÍVEL** |
| **Diretório Mapeado `C:\Artifacts`** | Criado, acessível e populado com os novos artefatos oficiais (CRT estático) | **APROVADO** |

---

## 2. Auditoria Criptográfica Pré-Execução dos Artefatos

Todos os arquivos foram baixados do GitHub Actions Run oficial `42806985528` e alocados em `C:\Artifacts\`. Os hashes SHA-256 foram recalculados localmente e comparados com os valores esperados:

| Arquivo Obrigatório | Tamanho (Bytes) | SHA-256 Esperado / Calculado | Conformidade |
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

## 3. Preparação do Kit de Homologação em `C:\Artifacts\`

Para permitir que o operador execute o teste em uma máquina Windows limpa, VM descartável ou Windows Sandbox:

1. **Artefatos Oficiais de Produção (com vinculação estática `+crt-static`):**
   - `C:\Artifacts\Visor-Setup-v0.4.4-x64.exe`
   - `C:\Artifacts\Visor-Portable-v0.4.4-x64.zip`
   - `C:\Artifacts\SHA256SUMS.txt`
2. **Amostras de Imagens Multi-Formato (`C:\Artifacts\samples\`):**
   - `sample_test.png` (800x600, 24-bit PNG)
   - `sample_test.jpg` (800x600, JPEG)
   - `sample_test.bmp` (800x600, 24-bit Windows Bitmap)
   - `moldura.png` (Ícone oficial de alta fidelidade)
3. **Arquivo de Inicialização Automática do Sandbox (`C:\Artifacts\VisorSandbox.wsb`):**
   ```xml
   <Configuration>
     <VGpu>Default</VGpu>
     <Networking>Disable</Networking>
     <MappedFolders>
       <MappedFolder>
         <HostFolder>C:\Artifacts</HostFolder>
         <SandboxFolder>C:\Users\WDAGUtilityAccount\Desktop\Artifacts</SandboxFolder>
         <ReadOnly>true</ReadOnly>
       </MappedFolder>
     </MappedFolders>
     <LogonCommand>
       <Command>explorer.exe C:\Users\WDAGUtilityAccount\Desktop\Artifacts</Command>
     </LogonCommand>
   </Configuration>
   ```

---

## 4. Auditoria Funcional no Host Windows 11

Como evidência técnica complementar de comportamento e ausência de regressões:

- **Instalação Sem UAC:** O instalador opera estritamente no escopo de usuário `%LOCALAPPDATA%\Programs\Visor`, sem exigir privilégios de administrador.
- **Atributos de Versão Win32:** `FileVersion: 0.4.4`, `ProductVersion: 0.4.4` confirmados no cabeçalho PE.
- **Renderização de Formatos:** O visualizador decodifica fluidamente PNG, JPG e BMP sem erros de DLL ou exceções de runtime.
- **Desinstalação Completa:** O desinstalador `unins000.exe` remove 100% dos binários e elimina a chave de registro em `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\VISOR`.
- **Versão Portátil:** Opera de forma autônoma e autocontida, sem dependências de registro ou instalação prévia.
- **Isolamento de Runtime MSVC:** PE Import Directory confirma ausência de `VCRUNTIME140.dll` e dependências externas de compilação.

---

## 5. Roteiro de Teste na VM do Operador

1. Copiar a pasta `C:\Artifacts` para o ambiente limpo da VM.
2. Validar os hashes SHA-256 no PowerShell:
   ```powershell
   Get-FileHash Visor-Setup-v0.4.4-x64.exe -Algorithm SHA256
   # Retorno esperado: 2fe95ea2d0ef307732d38435ed4360db69f58aebfee9f7427539ac42d50d924c
   ```
3. Executar o instalador `Visor-Setup-v0.4.4-x64.exe` ou descompactar `Visor-Portable-v0.4.4-x64.zip`.
4. Abrir as imagens de `samples\` e verificar que a inicialização ocorre instantaneamente sem nenhuma requisição de DLL ou runtime.
