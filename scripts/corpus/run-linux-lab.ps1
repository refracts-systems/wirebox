param(
    [string]$Distro = 'Ubuntu-26.04',
    [ValidateRange(1, 20)][int]$Repetitions = 3
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$runId = 'linux-forwarding-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [Guid]::NewGuid().ToString('N').Substring(0, 8)
$inputDir = Join-Path $repo "target/corpus-input/$runId"
New-Item -ItemType Directory -Path $inputDir | Out-Null
Push-Location $repo
try {
    & cargo build --locked -p wyrlex-headless
    if ($LASTEXITCODE -ne 0) { throw 'Headless build failed' }
    $exe = Join-Path $repo 'target/debug/wyrlex-headless.exe'
    foreach ($case in @('S01', 'S02', 'S03', 'S04', 'R01', 'N01', 'N02')) {
        $output = & $exe --corpus-demo $case
        if ($LASTEXITCODE -ne 0) { throw "Simulator export failed: $case" }
        $output | Out-File -Encoding utf8 (Join-Path $inputDir "$case.json")
    }
    $sources = @('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'crates/wirebox-kernel/build.rs', 'fixtures/scenarios/icmp-static-v1.json')
    $sources += @(Get-ChildItem kernel/src/*.zig, crates/*/src/*.rs | ForEach-Object { [IO.Path]::GetRelativePath($repo, $_.FullName).Replace('\', '/') })
    $hashes = @($sources | Sort-Object -Unique | ForEach-Object {
        @{ path = $_; sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $_).Hash.ToLowerInvariant() }
    })
    $provenance = @{
        committed_baseline = (& git rev-parse HEAD).Trim()
        working_tree_dirty = [bool](& git status --porcelain)
        simulator_command = 'wyrlex-headless.exe --corpus-demo S01|S02|S03|S04|R01|N01|N02'
        simulator_platform = 'Windows x64 MSVC'
        rust_version = (& rustc --version).Trim()
        zig_version = '0.17.0 (enforced by Cargo build script)'
        binary_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $exe).Hash.ToLowerInvariant()
        sources = $hashes
    }
    $provenance | ConvertTo-Json -Depth 8 | Out-File -Encoding utf8 (Join-Path $inputDir 'engine-provenance.json')
    # Forward slashes avoid backslashes being consumed by WSL's command parsing.
    $linuxRepoOutput = & wsl -d $Distro -- wslpath -a $repo.Replace('\', '/')
    if ($LASTEXITCODE -ne 0 -or -not $linuxRepoOutput) { throw 'Cannot resolve repository in WSL' }
    $linuxRepo = $linuxRepoOutput.Trim()
    $linuxInputOutput = & wsl -d $Distro -- wslpath -a $inputDir.Replace('\', '/')
    if ($LASTEXITCODE -ne 0 -or -not $linuxInputOutput) { throw 'Cannot resolve input directory in WSL' }
    $linuxInput = $linuxInputOutput.Trim()
    & wsl -d $Distro -u root -- python3 "$linuxRepo/scripts/corpus/linux_lab.py" run --repo $linuxRepo --input $linuxInput --run-id $runId --repetitions $Repetitions --distro-name $Distro
    if ($LASTEXITCODE -ne 0) { throw 'Linux corpus run failed; inspect the new run directory' }
    & wsl -d $Distro -- python3 "$linuxRepo/scripts/corpus/validate.py" --repo $linuxRepo
    if ($LASTEXITCODE -ne 0) { throw 'Corpus validation failed' }
} finally {
    Pop-Location
}
