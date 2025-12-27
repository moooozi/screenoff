#include "version.iss"
#define MyAppName "ScreenOff"
#define MyAppPublisher "M Zidane"
#define MyAppURL "https://github.com/mzidane/screenoff"
#define MyAppExeName "screenoff.exe"

[Setup]
AppId=dev.zidane.screenoff
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={userpf}\{#MyAppName}
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


[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent