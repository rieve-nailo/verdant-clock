#define AppName "Verdant Clock"
#define AppVersion GetEnv("APP_VERSION")
#define AppPublisher "Verdant Clock"
#define AppExeName "verdant-clock.exe"
#define ProjectRoot AddBackslash(SourcePath) + "..\.."

[Setup]
AppId={{F8AD2AC2-A9D4-41E2-A2BD-9F1A46BB736D}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\Verdant Clock
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#AppExeName}
OutputDir={#ProjectRoot}\dist
OutputBaseFilename=Verdant-Clock-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesInstallIn64BitMode=x64

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Files]
Source: "{#ProjectRoot}\target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: postinstall nowait skipifsilent
