param(
    [Parameter(Mandatory = $true)]
    [string]$ReleaseTag,

    [string]$UpstreamRemote = "origin",

    [string]$PatchRef = "codexx-goal-profiles-rust-v0.130.0",

    [string]$BaseTag = "rust-v0.130.0",

    [string]$PatchRange,

    [string]$WorkBranch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Run-Git {
    param([string[]]$Args)
    Write-Host "+ git $($Args -join ' ')"
    & git @Args
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Args -join ' ') failed with exit code $LASTEXITCODE"
    }
}

if ($ReleaseTag -notmatch "^rust-v") {
    $ReleaseTag = "rust-v$ReleaseTag"
}

if ([string]::IsNullOrWhiteSpace($WorkBranch)) {
    $WorkBranch = "codexx/$ReleaseTag"
}

if ([string]::IsNullOrWhiteSpace($PatchRange)) {
    $PatchRange = "$BaseTag..$PatchRef"
}

$dirty = (& git status --porcelain)
if (-not [string]::IsNullOrWhiteSpace($dirty)) {
    throw "Working tree is dirty. Commit or stash changes before merging a new release."
}

Run-Git @("fetch", $UpstreamRemote, "--tags")
Run-Git @("switch", "-C", $WorkBranch, $ReleaseTag)
Run-Git @("cherry-pick", $PatchRange)

Write-Host ""
Write-Host "Codexx patch range applied on $ReleaseTag."
Write-Host "Run checks, update docs/version references if needed, then push $WorkBranch."
