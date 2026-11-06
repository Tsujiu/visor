; Instalador por usuário. Não pede administrador e registra o Visor só neste usuário.

#define MyAppName "Visor"
#define MyAppVersion "0.2.1"
#define MyAppPublisher "MuTsuJii"
#define MyAppURL "https://github.com/Tsujiu/visor"
#define MyAppExeName "visor.exe"

[Setup]
AppId={{B7E4A1C2-9F30-4D5E-8A61-0C4E2F8A91D3}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\Visor
DefaultGroupName=Visor
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Visor-Setup-0.2.1
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
VersionInfoDescription=Instalador do Visor
VersionInfoProductName=Visor
VersionInfoVersion=0.2.1.0

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "Criar um atalho na área de trabalho"; GroupDescription: "Atalhos:"; Flags: unchecked
Name: "register"; Description: "Usar o Visor para abrir imagens"; GroupDescription: "Associação:"; Flags: checkedonce

[Files]
Source: "..\target\release\visor.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Visor"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\Visor"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Parameters: "--registrar-padrao"; StatusMsg: "Registrando o Visor como visualizador de imagens..."; Tasks: register; Flags: runhidden waituntilterminated skipifsilent
Filename: "{app}\{#MyAppExeName}"; Description: "Abrir o Visor"; Flags: postinstall nowait skipifsilent unchecked

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\Visor"
