; ==============================================================================
; VISOR — Script Oficial de Instalação Inno Setup 6
; ==============================================================================

#define MyAppName "VISOR"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Tsujiu"
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
VersionInfoVersion={#MyAppVersion}

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
