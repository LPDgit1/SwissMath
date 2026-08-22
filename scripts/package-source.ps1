[CmdletBinding()]
param(
    [string]$BundleName = 'SwissMath-v0.5-source'
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($BundleName) -or $BundleName -match '[\\/:]' -or $BundleName -match '\.\.') {
    throw 'BundleName non valido: usare un nome relativo senza slash, backslash o ..'
}

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$releaseRoot = Join-Path $projectRoot 'release'
$stageParent = Join-Path $projectRoot 'work\source-package'
$stageRoot = Join-Path $stageParent $BundleName
$archivePath = Join-Path $releaseRoot ($BundleName + '.zip')
$rootPrefix = $projectRoot.TrimEnd('\') + '\'

function Assert-InProject([string]$path) {
    $full = [System.IO.Path]::GetFullPath($path)
    if (-not $full.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Percorso fuori dal progetto: $full"
    }
    return $full
}

$null = Assert-InProject $stageParent
$null = Assert-InProject $stageRoot
$null = Assert-InProject $archivePath

New-Item -ItemType Directory -Path $releaseRoot -Force | Out-Null
if (Test-Path -LiteralPath $stageParent) {
    Remove-Item -LiteralPath $stageParent -Recurse -Force
}
New-Item -ItemType Directory -Path $stageRoot -Force | Out-Null

$excludedDirectoryNames = @('target', 'work', 'release', 'dist', 'node_modules', '.git', 'gen')
$excludedExtensions = @('.exe', '.msi', '.pdb', '.lib', '.exp', '.dll', '.zip', '.tmp', '.log')

Get-ChildItem -LiteralPath $projectRoot -Recurse -File -Force | ForEach-Object {
    $file = $_
    if (-not $file.FullName.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "File fuori dal progetto: $($file.FullName)"
    }

    $relative = $file.FullName.Substring($rootPrefix.Length)
    $parts = $relative -split '[\\/]'
    if ($parts | Where-Object { $excludedDirectoryNames -contains $_ }) { return }
    if ($excludedExtensions -contains $file.Extension.ToLowerInvariant()) { return }
    if ($file.Name -in @('Thumbs.db', '.DS_Store')) { return }

    $destination = Join-Path $stageRoot $relative
    $destinationParent = Split-Path -Parent $destination
    New-Item -ItemType Directory -Path $destinationParent -Force | Out-Null
    Copy-Item -LiteralPath $file.FullName -Destination $destination -Force
}

if (Test-Path -LiteralPath $archivePath) {
    Remove-Item -LiteralPath $archivePath -Force
}
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::Open($archivePath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    Get-ChildItem -LiteralPath $stageRoot -Recurse -File -Force | ForEach-Object {
        $relative = $_.FullName.Substring($stageRoot.Length).TrimStart('\', '/') -replace '\\', '/'
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
            $zip,
            $_.FullName,
            ($BundleName + '/' + $relative),
            [System.IO.Compression.CompressionLevel]::Optimal
        ) | Out-Null
    }
}
finally {
    $zip.Dispose()
}

$zip = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    $entries = @($zip.Entries | ForEach-Object { $_.FullName -replace '\\', '/' })
    $prefix = ($BundleName + '/')
    if ($entries.Count -eq 0 -or ($entries | Where-Object { -not $_.StartsWith($prefix, [System.StringComparison]::Ordinal) })) {
        throw "Archivio non conforme: deve avere un solo prefisso $prefix."
    }
    if ($entries | Where-Object { $_ -match '(^|/)(target|work|release|node_modules|gen|\.git)(/|$)' -or $_ -match '\.(exe|msi|pdb|dll|lib|exp|zip)$' }) {
        throw 'Archivio non conforme: contiene artefatti esclusi.'
    }
}
finally {
    $zip.Dispose()
}

Write-Output "Creato: $archivePath"
