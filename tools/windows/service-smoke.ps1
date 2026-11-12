# Run only in the disposable ScrumeVPN Windows VM, from elevated PowerShell.
# Installs the status-only service under SYSTEM; does not change networking.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Binary,
    [Parameter(Mandatory=$true)][switch]$DisposableLab
)
$ErrorActionPreference = 'Stop'
if (-not $DisposableLab) { throw 'DisposableLab required' }
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Elevated guest PowerShell required' }
if (Get-Service -Name ScrumeVPNSvc -ErrorAction SilentlyContinue) { throw 'Existing service detected; no replacement performed' }
$source = (Resolve-Path -LiteralPath $Binary).Path
$parent = Join-Path $env:ProgramFiles 'ScrumeVPN-Lab'
if (Test-Path -LiteralPath $parent) { throw 'Existing lab directory detected; choose a clean VM snapshot' }
New-Item -ItemType Directory -Path $parent | Out-Null
# Disable inherited write permissions; SID-based ACEs are localization-independent.
& icacls.exe $parent /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' '*S-1-5-32-545:(OI)(CI)RX' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Lab directory ACL failed' }
$installed = Join-Path $parent 'ScrumeVPNSvc.exe'
Copy-Item -LiteralPath $source -Destination $installed
$hash = (Get-FileHash -LiteralPath $installed -Algorithm SHA256).Hash
$created = $false
$service = $null
$report = $null
try {
    New-Service -Name ScrumeVPNSvc -DisplayName 'ScrumeVPN Lab (status only)' -BinaryPathName ('"' + $installed + '"') -StartupType Manual | Out-Null
    $created = $true
    $service = Get-Service -Name ScrumeVPNSvc
    $service.Start()
    $service.WaitForStatus('Running', [TimeSpan]::FromSeconds(10))
    $json = & $installed --status
    if ($LASTEXITCODE -ne 0) { throw 'Status client failed' }
    $status = $json | ConvertFrom-Json
    if ($status.result.state -ne 'UNENROLLED' -or $status.result.tunnel_available -ne $false) { throw 'Unexpected status response' }
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $service.Stop()
    $service.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(10))
    $watch.Stop()
    $report = [pscustomobject]@{
        status='PASS'; scope='SCM and console status IPC only; not TUN or vault';
        os_arch=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();
        process_arch=[Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString();
        binary_sha256=$hash; stop_ms=$watch.ElapsedMilliseconds
    }
} finally {
    if ($created) {
        if ($null -eq $service) { $service = Get-Service -Name ScrumeVPNSvc }
        try {
            $service.Refresh()
            if ($service.Status -ne 'Stopped') {
                $service.Stop()
                $service.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(10))
            }
        } finally { $service.Dispose() }
        & sc.exe delete ScrumeVPNSvc | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'Lab service delete failed; inspect VM before continuing' }
        $deadline = [DateTime]::UtcNow.AddSeconds(5)
        do {
            & sc.exe query ScrumeVPNSvc | Out-Null
            $queryCode = $LASTEXITCODE
            if ($queryCode -eq 1060) { break }
            if ($queryCode -ne 0) { throw 'Lab service cleanup query failed' }
            Start-Sleep -Milliseconds 100
        } while ([DateTime]::UtcNow -lt $deadline)
        if ($queryCode -ne 1060) { throw 'Lab service is still registered; cleanup not confirmed' }
    }
    # Keep lab binaries/evidence for inspection; do not delete unrelated files.
}
# PASS is emitted only after successful stop and confirmed SCM deletion.
$report | Add-Member -NotePropertyName cleanup -NotePropertyValue 'SERVICE_REMOVED'
$report | ConvertTo-Json
