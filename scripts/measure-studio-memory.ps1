param(
    [Parameter(Mandatory)][int]$StudioProcessId,
    [ValidateRange(1, 3600)][int]$Seconds = 1800,
    [string]$OutputPath = 'artifacts/studio-memory.json'
)
$ErrorActionPreference = 'Stop'
$process = Get-Process -Id $StudioProcessId
if ($process.ProcessName -ne 'wyrlex-studio') { throw 'PID must identify wyrlex-studio' }
$started = $process.StartTime.ToUniversalTime()
$samples = [System.Collections.Generic.List[object]]::new()
$watch = [Diagnostics.Stopwatch]::StartNew()
$failure = $null
try {
    while ($watch.Elapsed.TotalSeconds -lt $Seconds) {
        $process.Refresh()
        if ($process.HasExited) { throw 'Studio exited during observation' }
        $samples.Add([ordered]@{
            seconds = $watch.Elapsed.TotalSeconds
            working_set_bytes = $process.WorkingSet64
            private_bytes = $process.PrivateMemorySize64
            responding = $process.Responding
        })
        Start-Sleep -Seconds 1
    }
} catch { $failure = $_.Exception.Message }
$fullPath = [IO.Path]::GetFullPath($OutputPath)
[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($fullPath)) | Out-Null
$peak = ($samples | ForEach-Object { $_['working_set_bytes'] } | Measure-Object -Maximum).Maximum
[ordered]@{
    scope = 'Process memory and Windows responding flag only; not input latency or scripted workflow acceptance'
    process_id = $StudioProcessId
    process_started_utc = $started
    requested_seconds = $Seconds
    elapsed_seconds = $watch.Elapsed.TotalSeconds
    peak_working_set_bytes = $peak
    within_1gib = ($null -ne $peak -and $peak -le 1GB)
    failure = $failure
    samples = $samples
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $fullPath -Encoding utf8
if ($failure) { throw $failure }
Write-Output "Observation saved to $fullPath. G4 requires workload and latency evidence separately."
