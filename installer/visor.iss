; ==============================================================================
; VISOR — Script Oficial de Instalação Inno Setup 6
; ==============================================================================

#define MyAppName "VISOR"
#define MyAppVersion "1.0.2"
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
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "polish"; MessagesFile: "compiler:Languages\Polish.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[CustomMessages]
brazilianportuguese.TaskDesktopIcon=Criar um atalho na Área de Trabalho
brazilianportuguese.TaskRegisterAssoc=Registrar o VISOR como aplicativo candidato para abrir imagens
brazilianportuguese.TaskGroupShortcuts=Atalhos:
brazilianportuguese.TaskGroupWindows=Integração com o Windows:
brazilianportuguese.StatusRegister=Registrando o VISOR no Windows...
brazilianportuguese.RunProgram=Abrir o VISOR

english.TaskDesktopIcon=Create a desktop shortcut
english.TaskRegisterAssoc=Register VISOR as a candidate image viewer in Windows
english.TaskGroupShortcuts=Shortcuts:
english.TaskGroupWindows=Windows Integration:
english.StatusRegister=Registering VISOR with Windows...
english.RunProgram=Launch VISOR

spanish.TaskDesktopIcon=Crear un acceso directo en el Escritorio
spanish.TaskRegisterAssoc=Registrar VISOR como visor de imágenes en Windows
spanish.TaskGroupShortcuts=Accesos directos:
spanish.TaskGroupWindows=Integración con Windows:
spanish.StatusRegister=Registrando VISOR en Windows...
spanish.RunProgram=Abrir VISOR

french.TaskDesktopIcon=Créer un raccourci sur le Bureau
french.TaskRegisterAssoc=Enregistrer VISOR comme visionneuse d'images dans Windows
french.TaskGroupShortcuts=Raccourcis :
french.TaskGroupWindows=Intégration Windows :
french.StatusRegister=Enregistrement de VISOR dans Windows...
french.RunProgram=Lancer VISOR

german.TaskDesktopIcon=Desktop-Verknüpfung erstellen
german.TaskRegisterAssoc=VISOR als Bildbetrachter in Windows registrieren
german.TaskGroupShortcuts=Verknüpfungen:
german.TaskGroupWindows=Windows-Integration:
german.StatusRegister=VISOR wird in Windows registriert...
german.RunProgram=VISOR starten

italian.TaskDesktopIcon=Crea un collegamento sul Desktop
italian.TaskRegisterAssoc=Registra VISOR come visualizzatore di immagini in Windows
italian.TaskGroupShortcuts=Collegamenti:
italian.TaskGroupWindows=Integrazione con Windows:
italian.StatusRegister=Registrazione di VISOR in Windows...
italian.RunProgram=Avvia VISOR

japanese.TaskDesktopIcon=デスクトップにショートカットを作成する
japanese.TaskRegisterAssoc=VISOR を Windows の画像ビューアーとして登録する
japanese.TaskGroupShortcuts=ショートカット:
japanese.TaskGroupWindows=Windows との連携:
japanese.StatusRegister=VISOR を Windows に登録しています...
japanese.RunProgram=VISOR を起動する

polish.TaskDesktopIcon=Utwórz skrót na Pulpicie
polish.TaskRegisterAssoc=Zarejestruj VISOR jako przeglądarkę obrazów w systemie Windows
polish.TaskGroupShortcuts=Skróty:
polish.TaskGroupWindows=Integracja z systemem Windows:
polish.StatusRegister=Rejestrowanie VISOR w systemie Windows...
polish.RunProgram=Uruchom VISOR

russian.TaskDesktopIcon=Создать ярлык на Рабочем столе
russian.TaskRegisterAssoc=Зарегистрировать VISOR в системе для просмотра изображений
russian.TaskGroupShortcuts=Ярлыки:
russian.TaskGroupWindows=Интеграция с Windows:
russian.StatusRegister=Регистрация VISOR в Windows...
russian.RunProgram=Запустить VISOR

[Tasks]
Name: "desktopicon"; Description: "{cm:TaskDesktopIcon}"; GroupDescription: "{cm:TaskGroupShortcuts}"; Flags: unchecked
Name: "registerassoc"; Description: "{cm:TaskRegisterAssoc}"; GroupDescription: "{cm:TaskGroupWindows}"; Flags: checkedonce

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README-PORTABLE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Registro como candidato oficial via aplicativo centralizado (sem UAC, sem UserChoice)
Filename: "{app}\{#MyAppExeName}"; Parameters: "--registrar-padrao"; StatusMsg: "{cm:StatusRegister}"; Tasks: registerassoc; Flags: runhidden waituntilterminated skipifsilent
; Executar o aplicativo apos conclusao se o usuario desejar
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:RunProgram}"; Flags: postinstall nowait skipifsilent unchecked

[Code]
// Remocao seletiva de registros exclusivos do VISOR sem afetar outros aplicativos nem configuracoes pessoais
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Exts: array[0..96] of String;
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

    // 4. Remover registro de Visor.Image em OpenWithProgids para todas as extensoes candidatas
    Exts[0] := 'png'; Exts[1] := 'apng'; Exts[2] := 'jpg'; Exts[3] := 'jpeg'; Exts[4] := 'jpe';
    Exts[5] := 'jfif'; Exts[6] := 'gif'; Exts[7] := 'bmp'; Exts[8] := 'dib'; Exts[9] := 'tif';
    Exts[10] := 'tiff'; Exts[11] := 'webp'; Exts[12] := 'avif'; Exts[13] := 'avis'; Exts[14] := 'jxl';
    Exts[15] := 'ico'; Exts[16] := 'cur'; Exts[17] := 'ani'; Exts[18] := 'tga'; Exts[19] := 'icb';
    Exts[20] := 'vda'; Exts[21] := 'vst'; Exts[22] := 'pcx'; Exts[23] := 'dcx'; Exts[24] := 'qoi';
    Exts[25] := 'hdr'; Exts[26] := 'pic'; Exts[27] := 'exr'; Exts[28] := 'jp2'; Exts[29] := 'j2k';
    Exts[30] := 'j2c'; Exts[31] := 'jpx'; Exts[32] := 'psd'; Exts[33] := 'psb'; Exts[34] := 'xcf';
    Exts[35] := 'ora'; Exts[36] := 'ff'; Exts[37] := 'farbfeld'; Exts[38] := 'xbm'; Exts[39] := 'ppm';
    Exts[40] := 'pgm'; Exts[41] := 'pbm'; Exts[42] := 'pnm'; Exts[43] := 'pam'; Exts[44] := 'sgi';
    Exts[45] := 'rgb'; Exts[46] := 'rgba'; Exts[47] := 'bw'; Exts[48] := 'ras'; Exts[49] := 'wbmp';
    Exts[50] := 'dds'; Exts[51] := 'ktx'; Exts[52] := 'ktx2'; Exts[53] := 'astc'; Exts[54] := 'basis';
    Exts[55] := 'pvr'; Exts[56] := 'atf'; Exts[57] := 'vtf'; Exts[58] := 'blp'; Exts[59] := 'wal';
    Exts[60] := 'spr'; Exts[61] := 'txd'; Exts[62] := 'tex'; Exts[63] := 'svg'; Exts[64] := 'svgz';
    Exts[65] := 'emf'; Exts[66] := 'wmf'; Exts[67] := 'dxf'; Exts[68] := 'cdr'; Exts[69] := 'mng';
    Exts[70] := 'lottie'; Exts[71] := 'tgs'; Exts[72] := 'raw'; Exts[73] := 'dng'; Exts[74] := 'cr2';
    Exts[75] := 'crw'; Exts[76] := 'nef'; Exts[77] := 'nrw'; Exts[78] := 'arw'; Exts[79] := 'srf';
    Exts[80] := 'sr2'; Exts[81] := 'raf'; Exts[82] := 'orf'; Exts[83] := 'rw2'; Exts[84] := 'pef';
    Exts[85] := '3fr'; Exts[86] := 'iiq'; Exts[87] := 'erf'; Exts[88] := 'kdc'; Exts[89] := 'dcr';
    Exts[90] := 'mos'; Exts[91] := 'mrw'; Exts[92] := 'x3f'; Exts[93] := 'bay'; Exts[94] := 'cap';
    Exts[95] := 'cr3'; Exts[96] := 'srw';

    for I := 0 to 96 do
    begin
      RegDeleteValue(HKEY_CURRENT_USER, 'Software\Classes\.' + Exts[I] + '\OpenWithProgids', 'Visor.Image');
    end;
  end;
end;
