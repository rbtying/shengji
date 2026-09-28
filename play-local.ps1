# Start (or restart) the local game server with computer players, then open it in the browser.
#
#   pwsh -File play-local.ps1            # restart the server and open the game
#   pwsh -File play-local.ps1 -Stop      # stop the server
#   pwsh -File play-local.ps1 -Build     # rebuild frontend and server first
#   pwsh -File play-local.ps1 -NoBrowser # don't open a browser tab
param([switch]$Stop, [switch]$Build, [switch]$NoBrowser)

$root = $PSScriptRoot
$exe  = Join-Path $root 'target\release\shengji.exe'
$data = Join-Path $root 'local-data'
$logs = Join-Path $root 'local-logs'
New-Item -ItemType Directory -Force $data, $logs | Out-Null

# Only one server can own port 3030, so stop any shengji server that's running.
Get-Process shengji -ErrorAction SilentlyContinue |
    ForEach-Object { Write-Host "Stopping server (PID $($_.Id))"; Stop-Process -Id $_.Id -Force }
if ($Stop) { return }

if ($Build -or -not (Test-Path $exe)) {
    Push-Location (Join-Path $root 'frontend')
    npx --yes yarn@1 install --frozen-lockfile
    npx --yes yarn@1 build
    Pop-Location
    # frontend/dist is embedded into the server binary at compile time.
    cargo build --release --bin shengji --manifest-path (Join-Path $root 'Cargo.toml')
}

$env:DUMP_PATH    = Join-Path $data 'shengji_state.json'
$env:MESSAGE_PATH = Join-Path $data 'shengji_messages.json'
$p = Start-Process -FilePath $exe -WorkingDirectory (Join-Path $root 'backend') -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput (Join-Path $logs 'server.out.log') `
    -RedirectStandardError  (Join-Path $logs 'server.err.log')
Write-Host "Server running (PID $($p.Id)) at http://localhost:3030; logs in $logs"

if (-not $NoBrowser) {
    Start-Sleep -Seconds 1
    Start-Process 'http://localhost:3030'
}
