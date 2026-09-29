[CmdletBinding()]
param(
    [string]$ExecutablePath = (Join-Path $env:LOCALAPPDATA 'Programs\DropCove\DropCove.exe'),
    [string]$DatabasePath = (Join-Path $env:LOCALAPPDATA 'DropCove\shelf.db'),
    [string]$ArtifactPath = (Join-Path (Get-Location) 'artifacts\stage1-performance.json'),
    [int]$BatchCount = 100,
    [int]$ItemsPerBatch = 10,
    [int]$HoldVisibleSeconds = 0
)

$ErrorActionPreference = 'Stop'

if ($BatchCount -lt 1 -or $ItemsPerBatch -lt 1 -or $HoldVisibleSeconds -lt 0) {
    throw 'BatchCount and ItemsPerBatch must be positive; HoldVisibleSeconds cannot be negative.'
}

if (-not (Test-Path -LiteralPath $ExecutablePath)) {
    throw "DropCove executable was not found: $ExecutablePath"
}

$existing = Get-Process -Name DropCove -ErrorAction SilentlyContinue
if ($null -ne $existing) {
    $pids = ($existing | ForEach-Object Id) -join ', '
    throw "DropCove is already running (PID $pids). Close it before running this measurement."
}

$python = Get-Command py.exe -ErrorAction SilentlyContinue
if ($null -eq $python) {
    $python = Get-Command python.exe -ErrorAction SilentlyContinue
}
if ($null -eq $python) {
    throw 'Python 3 is required only for the disposable SQLite seed.'
}

Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class DropCoveStage1Native
{
    public delegate bool EnumWindowsCallback(IntPtr windowHandle, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr windowHandle, out uint processId);

    public static IntPtr FindWindowForProcess(int processId)
    {
        IntPtr found = IntPtr.Zero;
        EnumWindows((windowHandle, _) =>
        {
            uint candidateProcessId;
            GetWindowThreadProcessId(windowHandle, out candidateProcessId);
            if (candidateProcessId == (uint)processId)
            {
                found = windowHandle;
                return false;
            }

            return true;
        }, IntPtr.Zero);
        return found;
    }
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr windowHandle);

    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr windowHandle, uint message, IntPtr wParam, IntPtr lParam);

}
'@

$runId = [Guid]::NewGuid().ToString('N')
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "DropCove-Stage1-$runId"
$fixtureRoot = Join-Path $tempRoot 'fixtures'
$seedScript = Join-Path $tempRoot 'seed.py'
$backupDatabase = Join-Path $tempRoot 'original-shelf.db'
$backupWal = Join-Path $tempRoot 'original-shelf.db-wal'
$backupShm = Join-Path $tempRoot 'original-shelf.db-shm'
$seededDatabase = Join-Path $tempRoot 'seeded-shelf.db'
$seededWal = Join-Path $tempRoot 'seeded-shelf.db-wal'
$seededShm = Join-Path $tempRoot 'seeded-shelf.db-shm'
$databaseDirectory = Split-Path -Parent $DatabasePath
$settingsPath = Join-Path $databaseDirectory 'settings.json'
if (-not (Test-Path -LiteralPath $settingsPath)) {
    throw "DropCove settings were not found: $settingsPath"
}

$hotKeyConfiguration = (Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json).HotKey
if ($null -eq $hotKeyConfiguration -or $null -eq $hotKeyConfiguration.VirtualKey) {
    throw "DropCove settings do not contain a usable HotKey definition: $settingsPath"
}
$process = $null
$report = $null
$databaseWasMoved = $false
$walWasMoved = $false
$shmWasMoved = $false
$seedAttempted = $false

function Move-IfPresent([string]$source, [string]$destination) {
    if (Test-Path -LiteralPath $source) {
        Move-Item -LiteralPath $source -Destination $destination
        return $true
    }

    return $false
}

function Wait-ForShelfWindow {
    param(
        [int]$ProcessId,
        [int]$TimeoutSeconds = 20
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $handle = [DropCoveStage1Native]::FindWindowForProcess($ProcessId)
        if ($handle -ne [IntPtr]::Zero) {
            return $handle
        }

        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for the DropCove window for PID $ProcessId."
}

function Wait-ForVisibility {
    param(
        [IntPtr]$WindowHandle,
        [bool]$Visible,
        [int]$TimeoutSeconds = 10
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if ([DropCoveStage1Native]::IsWindowVisible($WindowHandle) -eq $Visible) {
            return
        }

        Start-Sleep -Milliseconds 25
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for DropCove visibility=$Visible."
}

function Wait-ForProcessReady {
    param(
        [int]$ProcessId,
        [int]$TimeoutSeconds = 60
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $current = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
        if ($null -eq $current) {
            throw "DropCove exited before startup completed."
        }
        if ($current.Responding) {
            Start-Sleep -Milliseconds 500
            return
        }

        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)

    throw 'Timed out waiting for DropCove startup to become responsive.'
}


function Get-DropCoveCpuSamples {
    param(
        [int]$ProcessId,
        [int]$SampleCount = 5
    )

    $samples = @()
    $previous = Get-Process -Id $ProcessId
    $clock = [Diagnostics.Stopwatch]::StartNew()
    for ($sample = 0; $sample -lt $SampleCount; $sample++) {
        $start = $clock.Elapsed
        Start-Sleep -Seconds 1
        $current = Get-Process -Id $ProcessId
        $elapsed = $clock.Elapsed - $start
        $cpuMilliseconds = ($current.TotalProcessorTime - $previous.TotalProcessorTime).TotalMilliseconds
        $wallMilliseconds = $elapsed.TotalMilliseconds
        $samples += [Math]::Round(($cpuMilliseconds / $wallMilliseconds) * 100, 6)
        $previous = $current
    }

    return $samples
}

function Get-DropCoveWorkingSetSamples {
    param(
        [int]$ProcessId,
        [int]$SampleCount = 5
    )

    $samples = @()
    for ($sample = 0; $sample -lt $SampleCount; $sample++) {
        $current = Get-Process -Id $ProcessId
        $samples += $current.WorkingSet64 / 1MB
        if ($sample -lt ($SampleCount - 1)) {
            Start-Sleep -Seconds 1
        }
    }

    return $samples
}

function Wait-InteractiveHold {
    param(
        [int]$ProcessId,
        [int]$Seconds,
        [string]$Phase
    )

    if ($Seconds -eq 0) {
        return 0
    }

    Write-Output "${Phase}: $Seconds seconds. Exercise the DropCove window now."
    $unresponsiveSamples = 0
    for ($second = 0; $second -lt $Seconds; $second++) {
        $current = Get-Process -Id $ProcessId
        if (-not $current.Responding) {
            $unresponsiveSamples++
        }
        Start-Sleep -Seconds 1
    }

    return $unresponsiveSamples
}

try {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
    New-Item -ItemType Directory -Path $fixtureRoot -Force | Out-Null
    New-Item -ItemType Directory -Path $databaseDirectory -Force | Out-Null

    $previewPngBytes = [Convert]::FromBase64String('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=')
    for ($batch = 1; $batch -le $BatchCount; $batch++) {
        $batchDirectory = Join-Path $fixtureRoot ("Batch{0:D3}" -f $batch)
        New-Item -ItemType Directory -Path $batchDirectory -Force | Out-Null
        for ($item = 1; $item -le $ItemsPerBatch; $item++) {
            if ((($batch - 1) % 10) -eq 0 -and $item -eq 1) {
                [IO.File]::WriteAllBytes((Join-Path $batchDirectory 'Preview.png'), $previewPngBytes)
            }
            else {
                $filePath = Join-Path $batchDirectory ("Item{0:D2}.txt" -f $item)
                [IO.File]::WriteAllText($filePath, 'DropCove Stage 1 measurement fixture.')
            }
        }
    }

    @'
import os
import sqlite3
import sys
import uuid
from datetime import datetime, timezone


def dotnet_roundtrip_utc():
    now = datetime.now(timezone.utc)
    return now.strftime('%Y-%m-%dT%H:%M:%S.') + f'{now.microsecond:06d}0+00:00'


database_path, fixture_root, batch_count, items_per_batch = sys.argv[1:]
batch_count = int(batch_count)
items_per_batch = int(items_per_batch)
connection = sqlite3.connect(database_path)
try:
    connection.execute('PRAGMA foreign_keys = ON')
    connection.execute('PRAGMA journal_mode = WAL')
    connection.executescript('''
        CREATE TABLE IF NOT EXISTS shelf_batches (
            id TEXT PRIMARY KEY,
            created_at TEXT NOT NULL,
            position INTEGER NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS shelf_items (
            id TEXT PRIMARY KEY,
            batch_id TEXT NOT NULL REFERENCES shelf_batches(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            path TEXT NOT NULL,
            name TEXT NOT NULL,
            is_folder INTEGER NOT NULL CHECK (is_folder IN (0, 1)),
            is_pinned INTEGER NOT NULL CHECK (is_pinned IN (0, 1)),
            UNIQUE (batch_id, position)
        );
    ''')

    for batch_index in range(batch_count):
        batch_id = str(uuid.uuid4())
        connection.execute(
            'INSERT INTO shelf_batches (id, created_at, position) VALUES (?, ?, ?)',
            (batch_id, dotnet_roundtrip_utc(), batch_index - batch_count),
        )
        for item_index in range(items_per_batch):
            name = (
                'Preview.png'
                if batch_index % 10 == 0 and item_index == 0
                else f'Item{item_index + 1:02d}.txt'
            )
            path = os.path.join(
                fixture_root,
                f'Batch{batch_index + 1:03d}',
                name,
            )
            connection.execute(
                '''
                INSERT INTO shelf_items
                    (id, batch_id, position, path, name, is_folder, is_pinned)
                VALUES (?, ?, ?, ?, ?, 0, 0)
                ''',
                (
                    str(uuid.uuid4()),
                    batch_id,
                    item_index,
                    path,
                    os.path.basename(path),
                ),
            )

    connection.commit()
finally:
    connection.close()

print(f'seeded {batch_count} batches and {batch_count * items_per_batch} items')
'@ | Set-Content -LiteralPath $seedScript -Encoding UTF8

    $databaseWasMoved = Move-IfPresent $DatabasePath $backupDatabase
    $walWasMoved = Move-IfPresent "$DatabasePath-wal" $backupWal
    $shmWasMoved = Move-IfPresent "$DatabasePath-shm" $backupShm

    $seedAttempted = $true
    & $python.Source $seedScript $DatabasePath $fixtureRoot $BatchCount $ItemsPerBatch
    if ($LASTEXITCODE -ne 0) {
        throw "The disposable SQLite seed failed with exit code $LASTEXITCODE."
    }

    $process = Start-Process -FilePath $ExecutablePath -PassThru
    Wait-ForProcessReady -ProcessId $process.Id
    $windowHandle = Wait-ForShelfWindow -ProcessId $process.Id
    Wait-ForVisibility -WindowHandle $windowHandle -Visible $true
    Start-Sleep -Seconds 5

    $initialUnresponsiveSamples = 0
    $restartUnresponsiveSamples = 0
    if ($HoldVisibleSeconds -gt 0) {
        Write-Output 'Initial hold checklist: scroll top-bottom-top, expand/collapse, verify Preview.png thumbnail or async icon fallback, pin Item02.txt, remove Item03.txt, and start/cancel a drag.'
        $initialUnresponsiveSamples = Wait-InteractiveHold -ProcessId $process.Id -Seconds $HoldVisibleSeconds -Phase 'Initial interaction hold'
    }

    $visibleCpuSamples = Get-DropCoveCpuSamples -ProcessId $process.Id
    $visibleWorkingSetSamples = Get-DropCoveWorkingSetSamples -ProcessId $process.Id

    if ($HoldVisibleSeconds -gt 0) {
        [DropCoveStage1Native]::SendMessage($windowHandle, 0x8002, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
        try {
            Wait-Process -Id $process.Id -Timeout 10
        }
        catch {
            Stop-Process -Id $process.Id -Force
            Wait-Process -Id $process.Id -Timeout 10
        }

        $process = Start-Process -FilePath $ExecutablePath -PassThru
        Wait-ForProcessReady -ProcessId $process.Id
        $windowHandle = Wait-ForShelfWindow -ProcessId $process.Id
        Wait-ForVisibility -WindowHandle $windowHandle -Visible $true
        Write-Output 'Restart hold checklist: expand Batch001; verify Item02.txt remains pinned, Item03.txt remains absent, and Preview.png still resolves asynchronously.'
        $restartUnresponsiveSamples = Wait-InteractiveHold -ProcessId $process.Id -Seconds $HoldVisibleSeconds -Phase 'Restart verification hold'
    }
    [DropCoveStage1Native]::SendMessage($windowHandle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    Wait-ForVisibility -WindowHandle $windowHandle -Visible $false

    $hiddenIdleCpuSamples = Get-DropCoveCpuSamples -ProcessId $process.Id
    $hiddenIdleWorkingSetSamples = Get-DropCoveWorkingSetSamples -ProcessId $process.Id
    $summonLatencyMs = $null
    $summonStatus = 'blocked'
    $summonBlocker = 'Automated keyboard injection cannot provide valid elevated-to-unelevated global-hotkey evidence in this session; run the interactive hotkey smoke manually.'

    $report = [ordered]@{
        generatedUtc = [DateTime]::UtcNow.ToString('O')
        executablePath = $ExecutablePath
        databasePath = $DatabasePath
        hotKey = $hotKeyConfiguration
        batchCount = $BatchCount
        itemCount = $BatchCount * $ItemsPerBatch
        imageFixtureCount = [Math]::Ceiling($BatchCount / 10.0)
        holdVisibleSeconds = $HoldVisibleSeconds
        initialUnresponsiveSamples = $initialUnresponsiveSamples
        restartUnresponsiveSamples = $restartUnresponsiveSamples
        scaleCheckStatus = if ($HoldVisibleSeconds -gt 0) { 'manual confirmation required' } else { 'not run' }
        scaleCheckInstructions = 'Initial: scroll top-bottom-top, expand/collapse, verify Preview.png thumbnail or async fallback, pin Item02.txt, remove Item03.txt, start/cancel drag. Restart: verify pin/removal persistence and image resolution.'
        visibleCpuPercentRawAverage = [Math]::Round(($visibleCpuSamples | Measure-Object -Average).Average, 4)
        visibleCpuPercentPerLogicalProcessor = [Math]::Round((($visibleCpuSamples | Measure-Object -Average).Average / [Environment]::ProcessorCount), 4)
        hiddenIdleCpuPercentRawAverage = [Math]::Round(($hiddenIdleCpuSamples | Measure-Object -Average).Average, 4)
        hiddenIdleCpuPercentPerLogicalProcessor = [Math]::Round((($hiddenIdleCpuSamples | Measure-Object -Average).Average / [Environment]::ProcessorCount), 4)
        logicalProcessorCount = [Environment]::ProcessorCount
        cpuMeasurementMethod = 'Process.TotalProcessorTime delta over wall-clock samples; per-logical field divides the raw percentage once'
        visibleWorkingSetMbAverage = [Math]::Round(($visibleWorkingSetSamples | Measure-Object -Average).Average, 2)
        hiddenIdleWorkingSetMbAverage = [Math]::Round(($hiddenIdleWorkingSetSamples | Measure-Object -Average).Average, 2)
        summonLatencyMs = $summonLatencyMs
        summonStatus = $summonStatus
        summonBlocker = $summonBlocker
        visibleCpuSamples = $visibleCpuSamples
        hiddenIdleCpuSamples = $hiddenIdleCpuSamples
        visibleWorkingSetSamplesMb = $visibleWorkingSetSamples | ForEach-Object { [Math]::Round($_, 2) }
        hiddenIdleWorkingSetSamplesMb = $hiddenIdleWorkingSetSamples | ForEach-Object { [Math]::Round($_, 2) }
        visibleState = "$BatchCount Shelf Batches / $($BatchCount * $ItemsPerBatch) Shelf Items after normal visible launch"
        hiddenIdleState = "$BatchCount Shelf Batches / $($BatchCount * $ItemsPerBatch) Shelf Items after the close-to-hide action"
    }
}
finally {
    $canRestore = $true
    if ($null -ne $process) {
        $current = Get-Process -Id $process.Id -ErrorAction SilentlyContinue
        if ($null -ne $current) {
            $expectedPath = (Resolve-Path -LiteralPath $ExecutablePath).Path
            $identity = Get-CimInstance Win32_Process -Filter "ProcessId=$($process.Id)"
            $ownsProcess = $null -ne $identity -and
                [String]::Equals($identity.ExecutablePath, $expectedPath, [StringComparison]::OrdinalIgnoreCase)
            if (-not $ownsProcess) {
                Write-Warning "PID $($process.Id) no longer matches the requested executable; profile restoration was not attempted."
                $canRestore = $false
            }
            else {
                $windowHandle = [DropCoveStage1Native]::FindWindowForProcess($process.Id)
                if ($windowHandle -ne [IntPtr]::Zero) {
                    [DropCoveStage1Native]::SendMessage($windowHandle, 0x8002, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
                }

                try {
                    Wait-Process -Id $process.Id -Timeout 10
                }
                catch {
                    Write-Warning "PID $($process.Id) did not exit after the diagnostic shutdown request; terminating that launched process."
                    Stop-Process -Id $process.Id -Force
                    try {
                        Wait-Process -Id $process.Id -Timeout 10
                    }
                    catch {
                        Write-Warning "PID $($process.Id) could not be confirmed stopped; profile restoration was not attempted."
                        $canRestore = $false
                    }
                }
            }
        }
    }

    if ($canRestore) {
        if ($databaseWasMoved -or $seedAttempted) {
            Move-IfPresent $DatabasePath $seededDatabase | Out-Null
            Move-IfPresent "$DatabasePath-wal" $seededWal | Out-Null
            Move-IfPresent "$DatabasePath-shm" $seededShm | Out-Null
        }

        if ($databaseWasMoved) {
            Move-Item -LiteralPath $backupDatabase -Destination $DatabasePath
        }
        if ($walWasMoved) {
            Move-Item -LiteralPath $backupWal -Destination "$DatabasePath-wal"
        }
        if ($shmWasMoved) {
            Move-Item -LiteralPath $backupShm -Destination "$DatabasePath-shm"
        }

        if ($null -ne $report) {
            New-Item -ItemType Directory -Path (Split-Path -Parent $ArtifactPath) -Force | Out-Null
            $report | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $ArtifactPath -Encoding UTF8
            $report | Format-List
            Write-Output "Measurement report: $ArtifactPath"
        }

        if (Test-Path -LiteralPath $tempRoot) {
            Remove-Item -LiteralPath $tempRoot -Recurse -Force
        }
    }
}
