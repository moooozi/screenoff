#include "version.iss"
#define MyAppName "ScreenOff"
#define MyAppPublisher "M Zidane"
#define MyAppURL "https://github.com/mzidane/screenoff"
#define MyAppExeName "screenoff.exe"

[Setup]
AppId=dev.zidane.screenoff
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} v{#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=.
OutputBaseFilename=ScreenOff-Setup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern dark includetitlebar
SetupIconFile=icons\app_icon.ico 
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible


[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/f /im screenoff.exe"; RunOnceId: "KillApp"; Flags: runhidden

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent