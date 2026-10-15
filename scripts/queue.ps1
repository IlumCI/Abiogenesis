# Runs experiment arms detached: seeds 1..10 per arm, at most $Max worlds at a time
# (counting every abiogenesis process on the machine, so queues side by side share the cap).
#
#   queue.ps1 -Arms "A=--no-tasks+--ocean+oceans\enwik8|B=--no-tasks+--ocean+genomes" -Done AB_DONE
#
# Run args are joined by + with no spaces: Start-Process splits spaced values apart.
# Copy the exe somewhere first (-Exe) if you'll rebuild while a batch runs.
param(
    [string]$Arms,
    [string]$Done = 'DONE',
    [string]$Exe = "$PSScriptRoot\..\target\release\abiogenesis.exe",
    [int]$Epochs = 20000,
    [int]$Max = 12
)
$wd = (Resolve-Path "$PSScriptRoot\..").Path
$proc = [IO.Path]::GetFileNameWithoutExtension($Exe)
$jobs = @()
foreach ($s in 1..10) {
    foreach ($arm in $Arms.Split('|')) {
        $name, $extra = $arm.Split('=', 2)
        $jobs += ,@("${name}_$s", $s, ($extra -split '\+' | Where-Object { $_ }))
    }
}
New-Item -ItemType Directory -Force "$wd\runs" | Out-Null
$running = @()
foreach ($j in $jobs) {
    while (@(Get-Process "$proc*" -ErrorAction SilentlyContinue).Count -ge $Max) { Start-Sleep -Seconds 15 }
    $name = $j[0]
    Remove-Item -Recurse -Force "$wd\runs\$name" -ErrorAction SilentlyContinue
    $a = @('run', '--epochs', "$Epochs", '--seed', "$($j[1])", '--economy', 'energy', '--out', "runs\$name") + $j[2]
    $p = Start-Process -FilePath $Exe -ArgumentList $a -WorkingDirectory $wd -RedirectStandardOutput "$wd\runs\$name.out" -RedirectStandardError "$wd\runs\$name.err" -WindowStyle Hidden -PassThru
    $p.PriorityClass = 'BelowNormal'
    $running += $p
}
$running | Wait-Process
Set-Content -Path "$wd\runs\$Done" -Value (Get-Date)
