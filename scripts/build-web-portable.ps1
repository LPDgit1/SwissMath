[CmdletBinding()]
param(
    [switch]$Offline,
    [string]$CargoPath,
    [string]$WasmBindgenPath,
    [string]$RustcPath
)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$buildScript = Join-Path $PSScriptRoot 'build-web.ps1'
$launcherSource = Join-Path $projectRoot 'tools\web_launcher.rs'
$releaseRoot = Join-Path $projectRoot 'release'
$output = Join-Path $releaseRoot 'SwissMath-Web-Portable.exe'

$buildArguments = @{}
if ($Offline) { $buildArguments.Offline = $true }
if ($CargoPath) { $buildArguments.CargoPath = $CargoPath }
if ($WasmBindgenPath) { $buildArguments.WasmBindgenPath = $WasmBindgenPath }
& $buildScript @buildArguments

if (-not $RustcPath) {
    $rustcCommand = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $rustcCommand) {
        throw 'rustc non trovato nel PATH. Specificare -RustcPath oppure installare Rust stable.'
    }
    $RustcPath = $rustcCommand.Source
}
if (-not (Test-Path -LiteralPath $RustcPath -PathType Leaf)) {
    throw "rustc non trovato: $RustcPath"
}

New-Item -ItemType Directory -Path $releaseRoot -Force | Out-Null
& $RustcPath --edition 2021 -D warnings -C opt-level=3 -C strip=symbols -C target-feature=+crt-static $launcherSource -o $output
if ($LASTEXITCODE -ne 0) { throw "Build del launcher portabile fallita ($LASTEXITCODE)." }
if (-not (Test-Path -LiteralPath $output -PathType Leaf)) {
    throw 'Launcher portabile non generato.'
}

$hash = (Get-FileHash -LiteralPath $output -Algorithm SHA256).Hash
$size = (Get-Item -LiteralPath $output).Length
Write-Output "SwissMath Web portabile pronto: $output"
Write-Output "Dimensione: $size byte"
Write-Output "SHA256: $hash"
