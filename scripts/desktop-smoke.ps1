param([switch]$UpdateDocs, [switch]$Sine)
$ErrorActionPreference = 'Stop'
if ($Sine -and $UpdateDocs) { throw 'Sine smoke records its own stage 8 capture; do not replace historical screenshots.' }
$workspacePath = Split-Path -Parent $PSScriptRoot
$executablePath = Join-Path $workspacePath 'target/debug/rust-daw-desktop.exe'
if (-not (Test-Path -LiteralPath $executablePath)) { throw 'Build desktop debug before running this smoke.' }
$previousArguments = [Environment]::GetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', 'Process')
$desktopSmokeProcess = $null
$portProbe = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 9223)
try { $portProbe.Start() } catch { throw 'Local debug port 9223 is occupied. Close the other debug session first.' } finally { $portProbe.Stop() }
try {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9223'
    $desktopSmokeProcess = Start-Process -FilePath $executablePath -WorkingDirectory $workspacePath -WindowStyle Hidden -PassThru
    Push-Location -LiteralPath $workspacePath
    try {
        if ($Sine) { & node (Join-Path $PSScriptRoot 'desktop-smoke.mjs') 9223 --sine }
        elseif ($UpdateDocs) { & node (Join-Path $PSScriptRoot 'desktop-smoke.mjs') 9223 --update-docs }
        else { & node (Join-Path $PSScriptRoot 'desktop-smoke.mjs') 9223 }
        if ($LASTEXITCODE -ne 0) { throw 'Desktop smoke failed.' }
    } finally { Pop-Location }
    [void] $desktopSmokeProcess.CloseMainWindow()
    if (-not $desktopSmokeProcess.WaitForExit(10000)) { throw 'Desktop did not close within 10 seconds.' }
    if ($desktopSmokeProcess.ExitCode -ne 0) { throw "Desktop exited with code $($desktopSmokeProcess.ExitCode)." }
    Write-Output 'Native window closed successfully.'
} finally {
    [Environment]::SetEnvironmentVariable('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS', $previousArguments, 'Process')
    if ($null -ne $desktopSmokeProcess -and -not $desktopSmokeProcess.HasExited) {
        [void] $desktopSmokeProcess.CloseMainWindow()
        if (-not $desktopSmokeProcess.WaitForExit(5000)) { Stop-Process -Id $desktopSmokeProcess.Id }
    }
}
