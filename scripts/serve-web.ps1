[CmdletBinding()]
param(
    [ValidateRange(1024, 65535)]
    [int]$Port = 8765,
    [switch]$NoBrowser
)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$webRootPath = Join-Path $projectRoot 'dist\web'

if (-not (Test-Path -LiteralPath $webRootPath -PathType Container)) {
    throw 'Bundle Web non trovato. Eseguire prima scripts\build-web.ps1.'
}

$webRoot = (Resolve-Path $webRootPath).Path.TrimEnd('\')
$webPrefix = $webRoot + '\'
$requiredFiles = @(
    'index.html',
    'app.js',
    'styles.css',
    'pkg\swissmath_web.js',
    'pkg\swissmath_web_bg.wasm'
)

foreach ($requiredFile in $requiredFiles) {
    if (-not (Test-Path -LiteralPath (Join-Path $webRoot $requiredFile) -PathType Leaf)) {
        throw "Bundle Web incompleto: manca $requiredFile. Eseguire scripts\build-web.ps1."
    }
}

function Get-ContentType([string]$path) {
    switch ([System.IO.Path]::GetExtension($path).ToLowerInvariant()) {
        '.html' { return 'text/html; charset=utf-8' }
        '.js'   { return 'text/javascript; charset=utf-8' }
        '.css'  { return 'text/css; charset=utf-8' }
        '.wasm' { return 'application/wasm' }
        '.json' { return 'application/json; charset=utf-8' }
        default { return 'application/octet-stream' }
    }
}

function Send-Response(
    [System.IO.Stream]$stream,
    [int]$statusCode,
    [string]$reason,
    [byte[]]$body,
    [string]$contentType,
    [bool]$headOnly
) {
    $headers = "HTTP/1.1 $statusCode $reason`r`n" +
        "Content-Type: $contentType`r`n" +
        "Content-Length: $($body.Length)`r`n" +
        "Cache-Control: no-cache`r`n" +
        "Connection: close`r`n`r`n"
    $headerBytes = [System.Text.Encoding]::ASCII.GetBytes($headers)
    $stream.Write($headerBytes, 0, $headerBytes.Length)
    if (-not $headOnly -and $body.Length -gt 0) {
        $stream.Write($body, 0, $body.Length)
    }
    $stream.Flush()
}

$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $Port)
$url = "http://127.0.0.1:$Port/"

try {
    $listener.Start()
    Write-Host "SwissMath Web disponibile su $url"
    Write-Host 'Premere Ctrl+C per arrestare il server locale.'

    if (-not $NoBrowser) {
        Start-Process $url
    }

    while ($true) {
        $client = $listener.AcceptTcpClient()
        try {
            $stream = $client.GetStream()
            $reader = [System.IO.StreamReader]::new(
                $stream,
                [System.Text.Encoding]::ASCII,
                $false,
                1024,
                $true
            )
            $requestLine = $reader.ReadLine()
            while ($null -ne ($headerLine = $reader.ReadLine()) -and $headerLine.Length -gt 0) { }

            if ([string]::IsNullOrWhiteSpace($requestLine)) {
                continue
            }

            $parts = $requestLine.Split(' ')
            if ($parts.Length -lt 2 -or $parts[0] -notin @('GET', 'HEAD')) {
                $body = [System.Text.Encoding]::UTF8.GetBytes('Metodo non consentito.')
                Send-Response $stream 405 'Method Not Allowed' $body 'text/plain; charset=utf-8' $false
                continue
            }

            $headOnly = $parts[0] -eq 'HEAD'
            $requestPath = ($parts[1] -split '\?', 2)[0]
            $decodedPath = [System.Uri]::UnescapeDataString($requestPath)
            if ($decodedPath -eq '/') { $decodedPath = '/index.html' }

            $relativePath = $decodedPath.TrimStart('/').Replace('/', '\')
            $candidate = [System.IO.Path]::GetFullPath((Join-Path $webRoot $relativePath))
            if (-not $candidate.StartsWith($webPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
                $body = [System.Text.Encoding]::UTF8.GetBytes('Percorso non consentito.')
                Send-Response $stream 403 'Forbidden' $body 'text/plain; charset=utf-8' $headOnly
                continue
            }

            if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
                $body = [System.Text.Encoding]::UTF8.GetBytes('Risorsa non trovata.')
                Send-Response $stream 404 'Not Found' $body 'text/plain; charset=utf-8' $headOnly
                continue
            }

            $body = [System.IO.File]::ReadAllBytes($candidate)
            Send-Response $stream 200 'OK' $body (Get-ContentType $candidate) $headOnly
        }
        catch {
            Write-Warning $_.Exception.Message
        }
        finally {
            $client.Dispose()
        }
    }
}
finally {
    $listener.Stop()
}
