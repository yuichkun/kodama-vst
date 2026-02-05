#define AppName "Kodama"

[Setup]
AppName={#AppName}
AppVersion={#MyAppVersion}
DefaultDirName={commoncf}\VST3
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir={#OutputDir}
OutputBaseFilename=Kodama-{#MyAppVersion}-Windows
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Files]
Source: "{#SourceDir}\VST3\Kodama.vst3\*"; DestDir: "{commoncf}\VST3\Kodama.vst3"; Flags: ignoreversion recursesubdirs createallsubdirs
