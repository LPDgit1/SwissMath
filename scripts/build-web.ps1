[CmdletBinding()]
param(
    [switch]$Offline,
    [string]$CargoPath,
    [string]$WasmBindgenPath
)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifest = Join-Path $projectRoot 'apps\web\Cargo.toml'
$dist = Join-Path $projectRoot 'dist\web'
$wasmTarget = Join-Path $projectRoot 'apps\web\target\wasm32-unknown-unknown\release\swissmath_web.wasm'

if (-not $CargoPath) {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargoCommand) {
        throw 'cargo non trovato nel PATH. Specificare -CargoPath oppure installare Rust stable.'
    }
    $CargoPath = $cargoCommand.Source
}
if (-not $WasmBindgenPath) {
    $bindgenCommand = Get-Command wasm-bindgen -ErrorAction SilentlyContinue
    if (-not $bindgenCommand) {
        throw 'wasm-bindgen non trovato nel PATH. Specificare -WasmBindgenPath oppure installare il CLI compatibile.'
    }
    $WasmBindgenPath = $bindgenCommand.Source
}
if (-not (Test-Path -LiteralPath $CargoPath)) { throw "cargo non trovato: $CargoPath" }
if (-not (Test-Path -LiteralPath $WasmBindgenPath)) { throw "wasm-bindgen non trovato: $WasmBindgenPath" }

if (Test-Path -LiteralPath $dist) {
    Remove-Item -LiteralPath $dist -Recurse -Force
}
New-Item -ItemType Directory -Force -Path (Join-Path $dist 'pkg') | Out-Null

$cargoArgs = @('build', '--manifest-path', $manifest, '--target', 'wasm32-unknown-unknown', '--release')
if ($Offline) { $cargoArgs += '--offline' }
& $CargoPath @cargoArgs
if ($LASTEXITCODE -ne 0) { throw "Build WASM fallita ($LASTEXITCODE)." }

& $WasmBindgenPath --version
if ($LASTEXITCODE -ne 0) { throw "Impossibile leggere la versione di wasm-bindgen ($LASTEXITCODE)." }
& $WasmBindgenPath $wasmTarget --target web --out-dir (Join-Path $dist 'pkg')
if ($LASTEXITCODE -ne 0) { throw "Generazione wasm-bindgen fallita ($LASTEXITCODE)." }

Copy-Item (Join-Path $projectRoot 'apps\web\web\index.html') (Join-Path $dist 'index.html') -Force
Copy-Item (Join-Path $projectRoot 'apps\web\web\app.js') (Join-Path $dist 'app.js') -Force
Copy-Item (Join-Path $projectRoot 'apps\web\web\styles.css') (Join-Path $dist 'styles.css') -Force
Get-ChildItem (Join-Path $dist 'pkg') -Filter '*.d.ts' -File -ErrorAction SilentlyContinue | Remove-Item -Force

foreach ($required in @('index.html', 'app.js', 'styles.css', 'pkg\swissmath_web.js', 'pkg\swissmath_web_bg.wasm')) {
    $path = Join-Path $dist $required
    if (-not (Test-Path -LiteralPath $path)) { throw "Asset deploy mancante: $required" }
}

Write-Output "SwissMath Web v0.4 pronto: $dist"
Get-ChildItem -LiteralPath $dist -Recurse -File | Select-Object FullName, Length
