# Novel installer for Windows — downloads a prebuilt binary from GitHub releases.
#
# Usage (PowerShell):
#   irm https://raw.githubusercontent.com/taidge/novel/main/scripts/install.ps1 | iex
#
# Environment variables:
#   $env:NOVEL_VERSION   Pin a specific version (e.g. "v0.1.0"). Default: latest.
#   $env:NOVEL_DIR       Install directory. Default: $HOME\.novel\bin.
#   $env:NOVEL_REPO      Override the GitHub repo. Default: taidge/novel.

$ErrorActionPreference = "Stop"

function Write-Info($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Warn($msg) { Write-Host "warning: $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

$Repo       = if ($env:NOVEL_REPO) { $env:NOVEL_REPO } else { "taidge/novel" }
$InstallDir = if ($env:NOVEL_DIR)  { $env:NOVEL_DIR }  else { Join-Path $HOME ".novel\bin" }
$BinName    = "novel.exe"

# Detect architecture
$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default { Write-Err "unsupported arch: $($env:PROCESSOR_ARCHITECTURE)" }
}
$target = "$arch-pc-windows-msvc"

# Resolve version
if ($env:NOVEL_VERSION) {
    $Tag = $env:NOVEL_VERSION
} else {
    Write-Info "resolving latest version"
    try {
        $resp = Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/$Repo/releases/latest" -MaximumRedirection 0 -ErrorAction SilentlyContinue
    } catch {
        $resp = $_.Exception.Response
    }
    if (-not $resp) {
        # Fall back to API
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $Tag = $release.tag_name
    } else {
        $location = $resp.Headers.Location
        if ($location -is [array]) { $location = $location[0] }
        $Tag = ($location -split '/')[-1]
    }
}
$Version = $Tag.TrimStart('v')

$asset = "novel-v$Version-$target.zip"
$url   = "https://github.com/$Repo/releases/download/$Tag/$asset"

Write-Info "target: $target"
Write-Info "version: $Tag"
Write-Info "downloading: $url"

$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "novel-install-$([guid]::NewGuid())") -Force
try {
    $archivePath = Join-Path $tmp $asset
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archivePath
    } catch {
        Write-Err "failed to download $asset. Does this release include a build for $target?"
    }

    Write-Info "extracting"
    Expand-Archive -Path $archivePath -DestinationPath $tmp -Force
    $stage   = Join-Path $tmp "novel-v$Version-$target"
    $binPath = Join-Path $stage $BinName
    if (-not (Test-Path $binPath)) { Write-Err "archive did not contain $BinName" }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path $binPath -Destination (Join-Path $InstallDir $BinName) -Force

    Write-Info "installed $BinName to $InstallDir\$BinName"

    # Suggest PATH update if not already present
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not ($userPath -split ';' | Where-Object { $_ -eq $InstallDir })) {
        Write-Warn "$InstallDir is not on your User PATH."
        Write-Warn "Add it permanently with:"
        Write-Host ""
        Write-Host "    [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`" + [Environment]::GetEnvironmentVariable('Path','User'), 'User')"
        Write-Host ""
    } else {
        & (Join-Path $InstallDir $BinName) --version
    }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
