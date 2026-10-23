param(
    [switch]$CoreOnly,
    [ValidateSet('Debug', 'ReleaseSafe', 'ReleaseFast')]
    [string[]]$ZigModes = @('Debug', 'ReleaseSafe')
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot

function Invoke-Checked {
    param([string]$Exe, [string[]]$ToolArgs)
    & $Exe @ToolArgs
    if ($LASTEXITCODE -ne 0) { throw "$Exe failed with exit code $LASTEXITCODE" }
}

$zig = if ($env:WIREBOX_ZIG) { $env:WIREBOX_ZIG } else {
    $installed = Join-Path $env:LOCALAPPDATA 'Programs\Zig\zig-x86_64-windows-0.17.0\zig.exe'
    if (Test-Path -LiteralPath $installed) { $installed } else { 'zig' }
}

Push-Location $repo
try {
    Invoke-Checked -Exe $zig -ToolArgs @('version')
    Invoke-Checked -Exe 'rustc' -ToolArgs @('--version')
    Invoke-Checked -Exe 'cargo' -ToolArgs @('fmt', '--all', '--', '--check')
    Invoke-Checked -Exe $zig -ToolArgs @('fmt', '--check', 'kernel/src')
    New-Item -ItemType Directory -Force -Path 'target' | Out-Null
    foreach ($mode in $ZigModes) {
        # ReleaseSafe checks optimizer-sensitive behavior without discarding safety checks.
        Invoke-Checked -Exe $zig -ToolArgs @('test', 'kernel/src/abi.zig', '-O', $mode, '--cache-dir', 'target/zig-tests')
    }
    Invoke-Checked -Exe $zig -ToolArgs @('cc', '-target', 'x86_64-windows-msvc', '-std=c11', '-c', 'kernel/tests/layout.c', '-Ikernel/include', '-o', 'target/abi-layout.obj')
    $packages = if ($CoreOnly) { @('-p', 'wirebox-kernel', '-p', 'wirebox-headless') } else { @('--workspace') }
    Invoke-Checked -Exe 'cargo' -ToolArgs (@('test', '--locked') + $packages)
    Invoke-Checked -Exe 'cargo' -ToolArgs (@('clippy', '--locked') + $packages + @('--all-targets', '--', '-D', 'warnings'))
    Invoke-Checked -Exe 'cargo' -ToolArgs @('run', '--locked', '-p', 'wirebox-headless', '--', '--probe')
    Invoke-Checked -Exe 'cargo' -ToolArgs @('run', '--locked', '-p', 'wirebox-headless', '--', '--check-fixtures')
    Invoke-Checked -Exe 'cargo' -ToolArgs @('run', '--locked', '-p', 'wirebox-headless', '--', '--scheduler-demo')
    Invoke-Checked -Exe 'cargo' -ToolArgs @('run', '--locked', '-p', 'wirebox-headless', '--', '--check-runtime')
    Invoke-Checked -Exe 'cargo' -ToolArgs @('run', '--locked', '-p', 'wirebox-headless', '--', '--check-network')
    if (-not $CoreOnly) {
        Invoke-Checked -Exe 'cargo' -ToolArgs @('build', '--locked', '-p', 'wirebox-studio')
    }
    Write-Output 'Wirebox verification passed. Native UI interaction is a separate manual/Computer Use check.'
} finally {
    Pop-Location
}
