param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('add', 'remove')]
  [string]$Action,

  [Parameter(Mandatory = $true)]
  [string]$CliDir
)

$ErrorActionPreference = 'Stop'

$current = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @()
if (-not [string]::IsNullOrWhiteSpace($current)) {
  $entries = $current -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
}

$normalizedTarget = $CliDir.Trim().Trim('"').TrimEnd('\\').ToLowerInvariant()
$nextEntries = New-Object System.Collections.Generic.List[string]
$seen = [System.Collections.Generic.HashSet[string]]::new()

foreach ($entry in $entries) {
  $normalized = $entry.Trim().Trim('"').TrimEnd('\\').ToLowerInvariant()
  if ([string]::IsNullOrWhiteSpace($normalized)) { continue }
  if ($normalized -eq $normalizedTarget) { continue }
  if ($seen.Add($normalized)) {
    $nextEntries.Add($entry.Trim().Trim('"'))
  }
}

$status = 'already-present'
if ($Action -eq 'add') {
  if ($seen.Add($normalizedTarget)) {
    $nextEntries.Add($CliDir)
    $status = 'updated'
  }
} elseif ($entries.Count -ne $nextEntries.Count) {
  $status = 'updated'
}

$newPath = if ($nextEntries.Count -eq 0) { $null } else { $nextEntries -join ';' }
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
Write-Output $status
