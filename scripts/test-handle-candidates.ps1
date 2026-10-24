param([string]$Zig = $env:WYRLEX_ZIG)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
if (-not $Zig) { $Zig = Join-Path $env:LOCALAPPDATA 'Programs\Zig\zig-x86_64-windows-0.17.0\zig.exe' }
$evidence = Join-Path $repo 'docs\evidence\handle-generation'
$scratch = Join-Path $repo 'artifacts\handle-generation'
New-Item -ItemType Directory -Force $evidence, $scratch | Out-Null
$source = Get-Content (Join-Path $repo 'kernel\src\abi.zig') -Raw
$transition = 'slot.generation = if (slot.generation == generation_max) 0 else slot.generation + 1;'
if (($source.Split($transition).Count - 1) -ne 1) { throw 'Expected exactly one B generation transition' }
$namespaceCheck = 'if (handle >> 62 != 0) return null;'
if (($source.Split($namespaceCheck).Count - 1) -ne 1) { throw 'Expected exactly one namespace check' }
$results = @()
foreach ($candidate in @('A', 'B')) {
    $dir = Join-Path $scratch $candidate
    New-Item -ItemType Directory -Force $dir | Out-Null
    Copy-Item -Path (Join-Path $repo 'kernel\src\*') -Destination $dir -Recurse -Force
    $text = $source
    if ($candidate -eq 'A') {
        $text = $text.Replace($namespaceCheck, "if (handle & (@as(u64, 1) << 63) != 0) return null;`n    $namespaceCheck")
        $text = $text.Replace($transition, @'
slot.generation = if (slot.generation == 0x7fffffff) 0 else slot.generation + 1;
    slot.generation = if (slot.generation == 0x3fffffff) 0 else slot.generation + 1;
'@)
    }
    [IO.File]::WriteAllText((Join-Path $dir 'abi.zig'), $text)
    foreach ($mode in @('Debug', 'ReleaseSafe', 'ReleaseFast')) {
        $log = Join-Path $evidence "candidate-$($candidate.ToLower())-$mode.log"
        & $Zig test (Join-Path $dir 'abi.zig') -O $mode --cache-dir (Join-Path $repo 'target\zig-handle-tests') --test-filter 'handle ' *> $log
        $exitCode = $LASTEXITCODE
        $body = Get-Content $log -Raw
        if ($candidate -eq 'B' -and ($exitCode -ne 0 -or $body -notmatch 'All 10 tests passed')) { throw "B failed: $log" }
        if ($candidate -eq 'A' -and ($exitCode -eq 0 -or $body -notmatch '4 failed')) { throw "A did not exhibit the expected four invariant failures: $log" }
        $results += [ordered]@{ candidate=$candidate; mode=$mode; exit_code=$exitCode; verdict=$(if ($candidate -eq 'A') {'invariant_failure'} else {'pass'}); log=[IO.Path]::GetFileName($log) }
        Write-Output "$candidate $mode exit=$exitCode"
    }
}
[ordered]@{ zig=(& $Zig version); results=$results; source_sha256=(Get-FileHash (Join-Path $repo 'kernel\src\abi.zig') -Algorithm SHA256).Hash } | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $evidence 'results.json')
