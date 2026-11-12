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
try {
    New-Service -Name ScrumeVPNSvc -DisplayName 'ScrumeVPN Lab (status only)' -BinaryPathName ('"' + $installed + '"') -StartupType Manual | Out-Null
    $created = $true
    Start-Service -Name ScrumeVPNSvc
    $service = Get-Service -Name ScrumeVPNSvc
    $service.WaitForStatus('Running', [TimeSpan]::FromSeconds(10))
    $json = & $installed --status
    if ($LASTEXITCODE -ne 0) { throw 'Status client failed' }
    $status = $json | ConvertFrom-Json
    if ($status.result.state -ne 'UNENROLLED' -or $status.result.tunnel_available -ne $false) { throw 'Unexpected status response' }
    $watch = [Diagnostics.Stopwatch]::StartNew()
    Stop-Service -Name ScrumeVPNSvc
    (Get-Service -Name ScrumeVPNSvc).WaitForStatus('Stopped', [TimeSpan]::FromSeconds(10))
    $watch.Stop()
    [pscustomobject]@{
        status='PASS'; scope='SCM and console status IPC only; not TUN or vault';
        os_arch=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();
        process_arch=[Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString();
        binary_sha256=$hash; stop_ms=$watch.ElapsedMilliseconds
    } | ConvertTo-Json
} finally {
    if ($created) {
        Stop-Service -Name ScrumeVPNSvc -ErrorAction SilentlyContinue
        & sc.exe delete ScrumeVPNSvc | Out-Null
    }
    # Keep lab binaries/evidence for inspection; do not delete unrelated files.
}
