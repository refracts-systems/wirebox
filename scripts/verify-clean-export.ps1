param()
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$export = Join-Path ([IO.Path]::GetTempPath()) ('wyrlex-clean-source-' + [guid]::NewGuid().ToString('N'))
$previousTarget = $env:CARGO_TARGET_DIR

Push-Location $repo
try {
    # Include uncommitted source too: this repository has no initial commit yet.
    $files = @(& git ls-files --cached --others --exclude-standard | Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate repository source' }
    New-Item -ItemType Directory -Path $export | Out-Null
    foreach ($file in $files) {
        $destination = Join-Path $export $file
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
        Copy-Item -LiteralPath (Join-Path $repo $file) -Destination $destination
    }
    Write-Output "Fresh source export: $export"
    # Dependency/compiler caches are intentionally reused. No source-side Zig .lib is copied.
    $env:CARGO_TARGET_DIR = Join-Path $repo 'target'
    Set-Location $export
    & cargo build --workspace --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Fresh-source build failed' }
    & cargo run --locked --offline -p wyrlex-headless -- --probe
    if ($LASTEXITCODE -ne 0) { throw 'Fresh-source ABI probe failed' }
    & cargo run --locked --offline -p wyrlex-headless -- --check-fixtures
    if ($LASTEXITCODE -ne 0) { throw 'Fresh-source fixture validation failed' }
    & cargo run --locked --offline -p wyrlex-headless -- --check-runtime
    if ($LASTEXITCODE -ne 0) { throw 'Fresh-source runtime validation failed' }
    & cargo run --locked --offline -p wyrlex-headless -- --check-network
    if ($LASTEXITCODE -ne 0) { throw 'Fresh-source network validation failed' }
    Write-Output 'Fresh-source build passed with cached dependencies; not a cold-cache or remote CI run.'
} finally {
    $env:CARGO_TARGET_DIR = $previousTarget
    Pop-Location
}
