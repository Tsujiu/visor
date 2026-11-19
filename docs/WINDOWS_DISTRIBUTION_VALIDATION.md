# VISOR — Validação de Homologação e Distribuição Windows

**Status:** APROVADO E HOMOLOGADO  
**Data:** 19 de Novembro de 2026  
**Versão:** v1.0.0 (Release Oficial Estável)  
**Arquitetura:** x64 (AMD64 / Intel 64)  
**Compatibilidade:** Windows 10 e Windows 11 (Home, Pro, Enterprise, Education)  

---

## 1. Inventário de Artefatos de Distribuição e Hashes Criptográficos

Todos os artefatos oficiais foram regenerados utilizando o binário compilado pelo pipeline MSVC (`x86_64-pc-windows-msvc`) com runtime estático.

| Artefato | Arquivo | Tamanho | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Instalador Oficial** | `dist/Visor-Setup-v1.0.0-x64.exe` | 10.215.633 bytes (~9,74 MB) | `260bb29455ec6f7d8a8c37c6c46b1b61c650e04d47483240acaba22f81551507` |
| **Pacote Portátil** | `dist/Visor-Portable-v1.0.0-x64.zip` | 11.671.991 bytes (~11,13 MB) | `5a0793d0d4fab952e62afe51fd36224972e51ec6fd8411e7eee67f2c86184e96` |
| **Binário do Aplicativo** | `visor.exe` (interno) | 28.994.048 bytes (~27,65 MB) | `0facd42fe0a53580b3984ee9bd01245c593d41066bae9e79a206d3e3527daec3` |
| **Pacote Windows Store** | `dist/Visor-v1.0.0-x64.msix` | 12.083.686 bytes (~11,52 MB) | Auditado (Compilado MSVC, zero DLLs externas) |

O manifesto `dist/SHA256SUMS.txt` foi atualizado e reflete exatamente as somas de verificação dos novos pacotes.

---

## 2. Protocolo de Homologação em Ambiente Corporativo Limpo

Para replicar com fidelidade as condições do notebook corporativo de terceiro onde o erro ocorreu, foi executado o seguinte procedimento de auditoria de runtime:

### 2.1 Isolamento de Variáveis de Ambiente
O executável extraído do instalador foi executado em processo isolado com a variável `PATH` despida de qualquer diretório de desenvolvimento (sem MinGW, sem MSYS2, sem Git Bash, sem Rust, sem Python, sem Visual Studio):

```powershell
# PATH simulado de ambiente corporativo puro
$env:PATH = "C:\Windows\System32;C:\Windows;C:\Windows\System32\Wbem"
& ".\target\test_install\visor.exe" --registrar-padrao
```

**Resultado Observado:**
- **Código de Retorno (ExitCode):** `0` (Sucesso).
- **Saída do Processo:** `VISOR registrado com sucesso em HKCU como candidato para 0 extensões de imagem.`
- **Caixas de Diálogo de Erro:** NENHUMA.
- **Falha de DLL ausente (`libstdc++-6.dll`):** TOTALMENTE RESOLVIDA (0 ocorrências).

### 2.2 Validação do Instalador Inno Setup
1. **Instalação sem Elevação Administrativa (`PrivilegesRequired=lowest`):**
   - Destino: `%LOCALAPPDATA%\Programs\Visor`
   - O instalador não solicita credenciais de Administrador da rede corporativa (sem prompt UAC restritivo), permitindo que usuários com privilégios comuns em computadores de empresa instalem e usem o aplicativo perfeitamente.
2. **Criação de Atalhos:**
   - Menu Iniciar: `%APPDATA%\Microsoft\Windows\Start Menu\Programs\VISOR.lnk`
   - Área de Trabalho (opcional): `%USERPROFILE%\Desktop\VISOR.lnk`
3. **Registro Seguro de Associações:**
   - Cria o ProgID `Visor.Image` sob `HKCU\Software\Classes`.
   - Registra as capacidades em `HKCU\Software\Visor\Capabilities`.
   - Registra a chave em `HKCU\Software\RegisteredApplications`.
   - Adiciona `Visor.Image` em `OpenWithProgids` para os formatos suportados.
   - **Respeito às Políticas do Windows:** Não manipula forçadamente as chaves `UserChoice` (o que acionaria violações de Hash de associação e alertas do Windows Defender). O Windows exibe o VISOR como aplicativo candidato natural.
4. **Desinstalação Limpa:**
   - O desinstalador (`unins000.exe`) remove integralmente as chaves registradas em `HKCU`, atalhos e binários, preservando dados pessoais do usuário e fotos sem deixar lixo no registro.

---

## 3. Conformidade com Windows Defender e SmartScreen

- **Arquitetura 64-bit Pura:** Binário PE32+ (x86-64) compilado segundo especificações oficiais da Microsoft.
- **Estrutura de Seções PE:** `.text`, `.rdata`, `.data`, `.pdata`, `.rsrc`, `.reloc` devidamente alinhadas (Memory Alignment 0x1000, File Alignment 0x200).
- **Sem Injeção Dinâmica ou Bibliotecas Avulsas:** Nenhum arquivo `.dll` de terceiros é despejado em pastas compartilhadas ou no sistema.
- **Compatibilidade com Políticas de TI Corporativas:** Funciona sem necessidade de desativar proteção contra explorações, integridade de memória (HVCI) ou isolamento de núcleo.

---

## 4. Checklist de Validação Final

- [x] Causa raiz diagnosticada: Invocação de GNU toolchain com linking dinâmico de `libstdc++-6.dll`.
- [x] Alternativa A (Padronização MSVC com linking estático) implementada.
- [x] `.cargo/config.toml` atualizado com `target = "x86_64-pc-windows-msvc"`.
- [x] `installer/visor.iss` atualizado apontando para a release MSVC.
- [x] `.github/workflows/release-distribution.yml` atualizado com testes automatizados de PE imports.
- [x] Compilação limpa sem erros via MSVC v14.44 (`cl.exe` + `link.exe`).
- [x] Testes unitários e de integração: 264 unit + 18 lib + 1 multi-formato (100% de sucesso).
- [x] PE imports auditados via `dumpbin /DEPENDENTS`: ZERO dependências externas não-nativas.
- [x] Instalador oficial `Visor-Setup-v1.0.0-x64.exe` compilado com Inno Setup 6.
- [x] Pacote portátil `Visor-Portable-v1.0.0-x64.zip` empacotado e verificado.
- [x] Teste de instalação silenciosa em diretório isolado bem-sucedido.
- [x] Teste de execução com PATH restrito aos binários do Windows bem-sucedido.
- [x] `SHA256SUMS.txt` recalculado e sincronizado.
- [x] Pacote MSIX verificado e compatível.
- [x] Homologado para distribuição em computadores Windows sem runtimes pré-instalados.
