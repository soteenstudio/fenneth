# Fenneth installer for Windows (PowerShell 5.1+ or PowerShell 7).
#
#   irm https://github.com/__REPO__/releases/latest/download/install.ps1 | iex
#
# Environment variables:
#   FENNETH_VERSION         Release tag to install, e.g. v0.2.0 (default: latest)
#   FENNETH_INSTALL         Install root; the binary goes to <root>\bin
#                           (default: %LOCALAPPDATA%\Fenneth)
#   FENNETH_NO_MODIFY_PATH  Set to 1 to leave your user PATH untouched
#   FENNETH_REPO            GitHub "owner/repo" to download from
#   FENNETH_BASE_URL        URL, or an existing local directory, to take the
#                           release files from instead of GitHub

$ErrorActionPreference = 'Stop'
try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch { }
$ProgressPreference = 'SilentlyContinue'

$Repo = if ($env:FENNETH_REPO) { $env:FENNETH_REPO } else { '__REPO__' }
$Version = if ($env:FENNETH_VERSION) { $env:FENNETH_VERSION } else { 'latest' }
$BaseUrl = $env:FENNETH_BASE_URL
$Root = if ($env:FENNETH_INSTALL) { $env:FENNETH_INSTALL } else { Join-Path $env:LOCALAPPDATA 'Fenneth' }
$BinDir = Join-Path $Root 'bin'

if (-not $BaseUrl) {
    if ($Repo -like '__REPO__*') {
        throw 'This copy of the installer does not know its GitHub repo; set $env:FENNETH_REPO = "owner/repo".'
    }
    if ($Version -eq 'latest') {
        $BaseUrl = "https://github.com/$Repo/releases/latest/download"
    } else {
        $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
    }
}

# Windows on ARM64 runs the x64 build through emulation.
$Target = 'x86_64-pc-windows-msvc'
$Asset = "fenneth-$Target.zip"

$Tmp = Join-Path ([IO.Path]::GetTempPath()) ("fenneth-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Tmp | Out-Null

function Get-Release([string]$Name, [string]$Dest) {
    if (Test-Path -LiteralPath $BaseUrl -PathType Container) {
        Copy-Item -LiteralPath (Join-Path $BaseUrl $Name) -Destination $Dest
    } else {
        Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$Name" -OutFile $Dest
    }
}

try {
    Write-Host "downloading $Asset ($Version)..."
    $Zip = Join-Path $Tmp $Asset
    Get-Release $Asset $Zip

    $ShaFile = Join-Path $Tmp "$Asset.sha256"
    $haveSha = $true
    try { Get-Release "$Asset.sha256" $ShaFile } catch { $haveSha = $false }
    if ($haveSha) {
        $expected = ((Get-Content -LiteralPath $ShaFile -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Zip).Hash.ToLowerInvariant()
        if ($expected -ne $actual) {
            throw "checksum mismatch for $Asset (expected $expected, got $actual)"
        }
    } else {
        Write-Warning 'no checksum file published; skipping verification'
    }

    Expand-Archive -LiteralPath $Zip -DestinationPath $Tmp -Force
    $Exe = Join-Path $Tmp "fenneth-$Target\fenneth.exe"
    if (-not (Test-Path -LiteralPath $Exe)) { throw 'unexpected archive layout' }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -LiteralPath $Exe -Destination (Join-Path $BinDir 'fenneth.exe') -Force
} finally {
    Remove-Item -LiteralPath $Tmp -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "installed: $(Join-Path $BinDir 'fenneth.exe')"
& (Join-Path $BinDir 'fenneth.exe') --version

# ---- PATH ----------------------------------------------------------------------------------------
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @()
if ($userPath) { $entries = $userPath -split ';' | Where-Object { $_ } }
if ($entries -notcontains $BinDir) {
    if ($env:FENNETH_NO_MODIFY_PATH -eq '1') {
        Write-Host "add this directory to your PATH:  $BinDir"
    } else {
        $newPath = (@($entries) + $BinDir) -join ';'
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Host "added $BinDir to your user PATH (open a new terminal to pick it up)"
    }
}
if (($env:Path -split ';') -notcontains $BinDir) { $env:Path = "$env:Path;$BinDir" }

# ---- C compiler ---------------------------------------------------------------------------------------
$haveCc = $false
foreach ($cc in 'gcc', 'clang', 'cc') {
    if (Get-Command $cc -ErrorAction SilentlyContinue) { $haveCc = $true; break }
}
if (-not $haveCc) {
    Write-Warning "no C compiler found; 'fenneth run/build' needs gcc or clang ('fenneth emit/check' work without)."
    Write-Host '  easiest option (MinGW-w64 gcc, self-contained):'
    Write-Host '    winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT'
    Write-Host '  alternative: MSYS2 (https://www.msys2.org), then:  pacman -S mingw-w64-ucrt-x86_64-gcc'
    Write-Host '  (open a new terminal afterwards so the compiler is on PATH)'
}

Write-Host 'done. try:  Set-Content hi.fen ''fn main() -> i32 { print("hi"); return 0; }''; fenneth hi.fen'
