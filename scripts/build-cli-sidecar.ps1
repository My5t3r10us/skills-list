$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$binaryDir = Join-Path $repoRoot "src-tauri\binaries"

New-Item -ItemType Directory -Force -Path $binaryDir | Out-Null

try {
  $targetTriple = (rustc --print host-tuple).Trim()
} catch {
  $targetTriple = (rustc -Vv | Select-String "host:" | ForEach-Object { $_.Line.Split(" ")[1] }).Trim()
}

if (-not $targetTriple) {
  throw "Unable to determine the Rust target triple for the CLI sidecar."
}

cargo build -p skills-cli --release

$extension = if ($IsWindows -or $env:OS -eq "Windows_NT") { ".exe" } else { "" }
$source = Join-Path $repoRoot "target\release\skills-list$extension"
$destination = Join-Path $binaryDir "skills-list-$targetTriple$extension"

if (-not (Test-Path $source)) {
  throw "CLI binary was not built at $source"
}

Copy-Item -Force $source $destination
Write-Host "Prepared skills-list CLI sidecar: $destination"
