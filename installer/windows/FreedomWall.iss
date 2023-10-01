; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!

#define MyAppName "FreedomWall"
#define MyAppVersion "2.0.0"
#define MyAppPublisher "tasuren"
#define MyAppURL "http://freedomwall.f5.si"
#define MyAppExeName "FreedomWall.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{1EEF9391-5AB1-422E-80A9-45676191BB05}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
InfoAfterFile=after_install.txt
; Remove the following line to run in administrative install mode (install for all users.)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=".\"
OutputBaseFilename=FreedomWall
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"; LicenseFile: "..\..\LICENSE"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"; LicenseFile: ".\LICENSE-ja"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\src\locales\en.yml"; DestDir: "{app}\src"; Flags: ignoreversion
Source: "..\..\src\locales\ja.yml"; DestDir: "{app}\src"; Flags: ignoreversion
Source: "..\..\pages\_*.html"; DestDir: "{app}\pages"; Flags: ignoreversion
Source: "..\..\pages\not_found.html"; DestDir: "{app}\pages"; Flags: ignoreversion
Source: "..\..\pages\style.css"; DestDir: "{app}\pages"; Flags: ignoreversion
Source: "..\..\pages\freedomwall\*.js"; DestDir: "{app}\pages\freedomwall"; Flags: ignoreversion
Source: "..\..\templates\*"; DestDir: "{app}\templates"; Flags: ignoreversion recursesubdirs
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
