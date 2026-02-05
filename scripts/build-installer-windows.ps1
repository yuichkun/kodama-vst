$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
$buildDir = Join-Path $projectRoot "build"
$installerDir = Join-Path $projectRoot "installer"
$stagingDir = Join-Path $installerDir "staging"
$vst3Path = Join-Path $buildDir "plugin/Kodama_artefacts/Release/VST3/Kodama.vst3"

$version = $env:KODAMA_VERSION
if ($version) {
    $version = $version -replace '^v', ''
}
if (-not $version -and $env:GITHUB_REF_NAME) {
    $version = $env:GITHUB_REF_NAME -replace '^v', ''
}
if (-not $version) {
    $cmakePath = Join-Path $projectRoot "CMakeLists.txt"
    $cmakeLine = Get-Content $cmakePath | Select-String -Pattern 'project\(Kodama VERSION ([0-9.]+)\)'
    if ($cmakeLine.Matches.Count -gt 0) {
        $version = $cmakeLine.Matches[0].Groups[1].Value
    }
}
if (-not $version) {
    $version = "0.1.0"
}

Write-Host "=== Kodama Windows Installer Build ==="
Write-Host "Version: $version"

if (-not (Test-Path $vst3Path)) {
    Write-Error "VST3 plugin not found at $vst3Path. Run 'npm run release:vst' first."
}

if (Test-Path $installerDir) {
    Remove-Item $installerDir -Recurse -Force
}

New-Item -ItemType Directory -Path $stagingDir | Out-Null
$stagingVst3 = Join-Path $stagingDir "VST3"
New-Item -ItemType Directory -Path $stagingVst3 | Out-Null
Copy-Item -Path $vst3Path -Destination $stagingVst3 -Recurse

$issPath = Join-Path $scriptDir "kodama-installer.iss"
$iscc = "iscc"

$arguments = @(
    "/DMyAppVersion=$version",
    "/DSourceDir=$stagingDir",
    "/DOutputDir=$installerDir",
    $issPath
)

Write-Host "Building installer with Inno Setup..."
& $iscc @arguments

Write-Host "=== Build Complete ==="
Write-Host "Installer output: $installerDir"
