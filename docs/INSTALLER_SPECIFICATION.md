# VISOR — Especificação Técnica do Instalador Inno Setup
## Fase 6 / Passo 2: Arquitetura, Diretivas e Ciclo de Vida de Instalação/Desinstalação

---

## 1. Visão Geral e Requisitos de Segurança

O instalador oficial do VISOR foi projetado para oferecer uma experiência fluida, moderna e sem fricção para usuários do Windows 10 e Windows 11.

### Princípios Fundamentais:
1. **Instalação sem UAC (`PrivilegesRequired=lowest`):**
   - Por padrão, o instalador não solicita privilégios de administrador nem aciona prompts do Controle de Conta de Usuário (UAC).
   - O aplicativo é instalado no diretório de programas do usuário (`%LOCALAPPDATA%\Programs\Visor`), garantindo que qualquer usuário padrão consiga instalar e atualizar o VISOR em sua máquina sem auxílio de suporte técnico.
2. **Desinstalação Não-Destrutiva:**
   - O desinstalador remove exclusivamente os arquivos de programa, atalhos e chaves de registro criadas pelo próprio VISOR.
   - **As preferências pessoais (`%LOCALAPPDATA%\Visor\settings.json`) são preservadas por padrão**, a menos que o usuário solicite explicitamente sua exclusão.
3. **Respeito Estrito ao Sistema de Associações:**
   - O instalador registra o VISOR como candidato qualificado (`OpenWithProgids` e `Capabilities`).
   - Nenhuma chave `UserChoice` é modificada ou corrompida.

---

## 2. Diretrizes e Parâmetros do Inno Setup

| Diretiva | Valor Definido | Justificativa Técnica |
| :--- | :--- | :--- |
| `AppId` | `{{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}}` | Identificador GUID permanente do VISOR (estabelecido desde a v0.2.0 para garantir atualização in-place contínua). |
| `AppName` | `Visor` | Nome oficial do produto. |
| `AppVersion` | `0.2.2` | Versão SemVer sincronizada com o `Cargo.toml`. |
| `AppPublisher` | `Projeto Visor` | Informação do desenvolvedor / editor. |
| `DefaultDirName` | `{autopf}\Visor` | Expande para `{localappdata}\Programs\Visor` em modo de usuário. |
| `PrivilegesRequired` | `lowest` | Instalação por usuário sem necessidade de privilégios de administrador. |
| `PrivilegesRequiredOverridesAllowed` | `dialog` | Permite que o instalador ofereça a opção "Instalar para todos os usuários" caso executado como Admin. |
| `ArchitecturesInstallIn64BitMode` | `x64compatible` | Força a instalação como binário nativo de 64 bits em sistemas x64. |
| `Compression` | `lzma2/ultra64` | Máxima taxa de compressão (reduz instalador para ~7 a 8 MB). |
| `SolidCompression` | `yes` | Otimização de dicionário compartilhado entre arquivos. |
| `UninstallDisplayIcon` | `{app}\visor.exe` | Exibe o ícone de alta resolução do VISOR no painel de desinstalação. |
| `DisableProgramGroupPage` | `yes` | Simplifica o assistente removendo páginas desnecessárias. |

---

## 3. Estrutura do Script Inno Setup (`installer/visor.iss`)

Abaixo está a especificação completa e documentada do script que será implementado no Passo 3:

```iss
; ==============================================================================
; VISOR — Script Oficial de Instalação Inno Setup 6
; ==============================================================================

#define MyAppName "VISOR"
#define MyAppVersion "0.2.2"
#define MyAppPublisher "Projeto Visor"
#define MyAppURL "https://github.com/Tsujiu/visor"
#define MyAppExeName "visor.exe"

[Setup]
; AppId permanente e exclusivo estabelecido desde a versão inicial
AppId={{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\Visor
DefaultGroupName=VISOR
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Visor-Setup-v{#MyAppVersion}-x64
SetupIconFile=..\assets\moldura.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
ChangesAssociations=yes
Uninstallable=yes
VersionInfoDescription=Instalador do VISOR
VersionInfoProductName=VISOR
VersionInfoVersion=0.2.2.0

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "Criar um atalho na Área de Trabalho"; GroupDescription: "Atalhos:"; Flags: unchecked
Name: "registerassoc"; Description: "Registrar o VISOR como aplicativo candidato para abrir imagens"; GroupDescription: "Integração com o Windows:"; Flags: checkedonce

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Registro como candidato oficial via aplicativo centralizado (sem UAC, sem UserChoice)
Filename: "{app}\{#MyAppExeName}"; Parameters: "--registrar-padrao"; StatusMsg: "Registrando o VISOR como aplicativo candidato..."; Tasks: registerassoc; Flags: runhidden waituntilterminated skipifsilent
; Executar o aplicativo apos conclusao se o usuario desejar
Filename: "{app}\{#MyAppExeName}"; Description: "Abrir o VISOR"; Flags: postinstall nowait skipifsilent unchecked

[Code]
// Remocao seletiva de registros exclusivos do VISOR sem afetar outros aplicativos nem configuracoes pessoais
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Exts: array[0..48] of String;
  I: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    // 1. Remover ProgID proprio
    RegDeleteKeyIncludingSubkeys(HKEY_CURRENT_USER, 'Software\Classes\Visor.Image');

    // 2. Remover capacidades proprias
    RegDeleteKeyIncludingSubkeys(HKEY_CURRENT_USER, 'Software\Visor');

    // 3. Remover valor de RegisteredApplications
    RegDeleteValue(HKEY_CURRENT_USER, 'Software\RegisteredApplications', 'VISOR');
    RegDeleteValue(HKEY_CURRENT_USER, 'Software\RegisteredApplications', 'Visor');

    // 4. Remover registro de Visor.Image em OpenWithProgids para as 49 extensoes candidatas
    Exts[0] := 'png'; Exts[1] := 'apng'; Exts[2] := 'jpg'; Exts[3] := 'jpeg'; Exts[4] := 'jpe';
    Exts[5] := 'jfif'; Exts[6] := 'webp'; Exts[7] := 'avif'; Exts[8] := 'gif'; Exts[9] := 'bmp';
    Exts[10] := 'dib'; Exts[11] := 'ico'; Exts[12] := 'tiff'; Exts[13] := 'tif'; Exts[14] := 'tga';
    Exts[15] := 'icb'; Exts[16] := 'vda'; Exts[17] := 'vst'; Exts[18] := 'dds'; Exts[19] := 'ktx2';
    Exts[20] := 'exr'; Exts[21] := 'hdr'; Exts[22] := 'jxl'; Exts[23] := 'svg'; Exts[24] := 'svgz';
    Exts[25] := 'qoi'; Exts[26] := 'pnm'; Exts[27] := 'pbm'; Exts[28] := 'pgm'; Exts[29] := 'ppm';
    Exts[30] := 'pam'; Exts[31] := 'ff'; Exts[32] := 'farbfeld'; Exts[33] := 'cr2'; Exts[34] := 'nef';
    Exts[35] := 'arw'; Exts[36] := 'dng'; Exts[37] := 'orf'; Exts[38] := 'rw2'; Exts[39] := 'pef';
    Exts[40] := 'raf'; Exts[41] := 'srw'; Exts[42] := 'crw'; Exts[43] := 'kdc'; Exts[44] := 'mrw';
    Exts[45] := 'nrw'; Exts[46] := 'raw'; Exts[47] := 'x3f'; Exts[48] := 'emf';

    for I := 0 to 48 do
    begin
      RegDeleteValue(HKEY_CURRENT_USER, 'Software\Classes\.' + Exts[I] + '\OpenWithProgids', 'Visor.Image');
    end;
  end;
end;
```

---

## 4. Ciclo de Vida: Instalação, Atualização e Desinstalação

### A. Fluxo de Instalação Inicial:
1. O usuário executa `Visor-Setup-v0.2.2-x64.exe`.
2. O assistente é exibido no idioma do sistema (Português ou Inglês).
3. Os arquivos são copiados para `%LOCALAPPDATA%\Programs\Visor`.
4. Os atalhos são criados no Menu Iniciar e, se selecionado, na Área de Trabalho.
5. Se a opção de registro estiver selecionada, executa `visor.exe --registrar-padrao` de forma invisível.
6. A tela final oferece o checkbox para iniciar o VISOR imediatamente.

### B. Fluxo de Atualização (Instalação sobre Versão Anterior):
1. O Inno Setup detecta a instalação existente via `AppId`.
2. Substitui os binários sem alterar o diretório escolhido pelo usuário.
3. Não solicita confirmação de caminhos redundantes.
4. O arquivo `%LOCALAPPDATA%\Visor\settings.json` permanece **100% intacto**.
5. Na primeira inicialização da versão atualizada, o sistema de schema do VISOR migra automaticamente eventuais campos novos com valores padrão seguros.

### C. Fluxo de Desinstalação:
1. O usuário aciona a desinstalação via "Configurações -> Aplicativos -> Aplicativos Instalados" ou pelo atalho do Inno Setup.
2. O desinstalador remove o executável, documentação e atalhos.
3. Remove as chaves registradas em `HKCU\Software\Classes\Visor.Image`, `HKCU\Software\Visor` e `HKCU\Software\RegisteredApplications\VISOR`.
4. **Proteção de Configurações:** O desinstalador pergunta explicitamente se o usuário deseja remover a pasta de configurações. Se o usuário escolher "Não" (opção padrão), suas configurações permanecem salvas para uso futuro.

---

## 5. Matriz de Compatibilidade e Verificação

| Cenário de Teste | Ambiente Alvo | Critério de Aceitação |
| :--- | :--- | :--- |
| **Instalação em Conta Padrão (Sem Admin)** | Windows 10 / 11 x64 | Instala sem solicitar credenciais de administrador nem exibir aviso UAC. |
| **Instalação em Conta com Admin** | Windows 10 / 11 x64 | Permite instalar tanto para o usuário atual quanto para todos os usuários. |
| **Execução Portátil Direta** | Pen drive / pasta arbitrária | Abre normalmente sem depender de chaves no registro. |
| **Atualização In-Place** | Versão anterior existente | Atualiza executável sem apagar preferências nem exigir desinstalação prévia. |
| **Desinstalação Padrão** | Windows 10 / 11 x64 | Remove binários e atalhos; preserva `settings.json` na resposta "Não". |
| **Remoção de Registro na Desinstalação** | Registro `HKCU` | Limpa ProgID e capacidades sem afetar associações de outros programas. |
