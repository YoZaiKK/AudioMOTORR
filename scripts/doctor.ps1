$ErrorActionPreference = 'Stop'
$missingCount = 0
$toolchainFile = Join-Path $PSScriptRoot '..\rust-toolchain.toml'
$toolchainText = Get-Content -LiteralPath $toolchainFile -Raw
if ($toolchainText -notmatch 'channel\s*=\s*"([^"]+)"') {
    throw 'Cannot determine the pinned Rust toolchain.'
}
$pinnedToolchain = $Matches[1]

Write-Output 'Rust DAW - Windows prerequisites (read-only)'

foreach ($toolName in @('rustc', 'cargo', 'node', 'npm')) {
    if (Get-Command $toolName -ErrorAction SilentlyContinue) {
        if ($toolName -in @('rustc', 'cargo')) {
            if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
                Write-Output "[UNKNOWN] rustup is needed to inspect $toolName without installing a toolchain."
                $missingCount++
                continue
            }
            $toolVersion = & rustup run $pinnedToolchain $toolName --version
        } else {
            $toolVersion = & $toolName --version
        }
        if ($LASTEXITCODE -eq 0) {
            Write-Output "[OK] $toolName : $toolVersion"
        } else {
            Write-Output "[MISSING] $toolName could not run."
            $missingCount++
        }
    } else {
        Write-Output "[MISSING] $toolName is not on PATH."
        $missingCount++
    }
}

if ($env:OS -ne 'Windows_NT') {
    Write-Output '[UNKNOWN] This script only verifies Windows prerequisites.'
    exit 1
}

$vswherePath = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (Test-Path -LiteralPath $vswherePath) {
    $compilerPaths = @(& $vswherePath -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find 'VC\Tools\MSVC\**\bin\Hostx64\x64\cl.exe')
    if ($compilerPaths.Count -gt 0) {
        Write-Output "[OK] MSVC x64 compiler: $($compilerPaths[0])"
    } else {
        Write-Output '[MISSING] MSVC x64 C++ tools were not found.'
        $missingCount++
    }
} else {
    Write-Output '[UNKNOWN] vswhere was not found; cannot verify MSVC reliably.'
    $missingCount++
}

$sdkRegistry = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots' -Name KitsRoot10 -ErrorAction SilentlyContinue
$sdkVerified = $false
if ($sdkRegistry -and $sdkRegistry.KitsRoot10) {
    $sdkIncludeRoot = Join-Path $sdkRegistry.KitsRoot10 'Include'
    if (Test-Path -LiteralPath $sdkIncludeRoot) {
        foreach ($sdkDirectory in Get-ChildItem -LiteralPath $sdkIncludeRoot -Directory) {
            $headerPath = Join-Path $sdkDirectory.FullName 'um\Windows.h'
            $libraryPath = Join-Path $sdkRegistry.KitsRoot10 "Lib\$($sdkDirectory.Name)\um\x64\kernel32.lib"
            if ((Test-Path -LiteralPath $headerPath) -and (Test-Path -LiteralPath $libraryPath)) {
                Write-Output "[OK] Windows SDK headers and x64 libraries: $($sdkDirectory.Name)"
                $sdkVerified = $true
                break
            }
        }
    }
}
if (-not $sdkVerified) {
    Write-Output '[UNKNOWN] Could not verify Windows SDK headers and x64 libraries.'
    $missingCount++
}

$webViewVersion = $null
$webViewClient = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
foreach ($registryRoot in @('HKCU:\Software\Microsoft\EdgeUpdate\Clients', 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients', 'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients')) {
    $webViewEntry = Get-ItemProperty -LiteralPath "$registryRoot\$webViewClient" -Name pv -ErrorAction SilentlyContinue
    if ($webViewEntry -and $webViewEntry.pv -and $webViewEntry.pv -ne '0.0.0.0') {
        $webViewVersion = $webViewEntry.pv
        break
    }
}
if ($webViewVersion) {
    Write-Output "[OK] WebView2 Runtime registered: $webViewVersion"
} else {
    Write-Output '[UNKNOWN] WebView2 Runtime was not found in the expected registry entries.'
    $missingCount++
}

Write-Output 'No tools were installed and no system settings were changed.'
Write-Output 'A successful native build and window smoke remain the final environment checks.'
if ($missingCount -gt 0) { exit 1 }
