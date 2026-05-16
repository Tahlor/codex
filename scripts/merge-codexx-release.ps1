param(
    [Parameter(Mandatory = $true)]
    [string]$ReleaseTag,

    [string]$UpstreamRemote = "origin",

    [string]$PatchRef = "codexx-goal-profiles-rust-v0.130.0",

    [string]$BaseTag = "rust-v0.130.0",

    [string]$PatchRange,

    [string]$WorkBranch,

    [switch]$NoFetch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Run-Git {
    param([string[]]$GitArgs)
    Write-Host "+ git $($GitArgs -join ' ')"
    & git @GitArgs
    if ($LASTEXITCODE -ne 0) {
        throw "git $($GitArgs -join ' ') failed with exit code $LASTEXITCODE"
    }
}

function Invoke-Git {
    param([string[]]$GitArgs)
    Write-Host "+ git $($GitArgs -join ' ')"
    & git @GitArgs
}

function Test-GitCommit {
    param([string]$Ref)
    & git rev-parse --verify --quiet "$Ref^{commit}" *> $null
    return $LASTEXITCODE -eq 0
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

if (-not $NoFetch) {
    Run-Git @("fetch", $UpstreamRemote, "--tags")
} else {
    Write-Host "Skipping fetch because -NoFetch was supplied."
}
if (-not (Test-GitCommit $ReleaseTag)) {
    throw "Release tag '$ReleaseTag' was not found after fetching '$UpstreamRemote'."
}
if (-not (Test-GitCommit $BaseTag)) {
    throw "Base tag '$BaseTag' was not found."
}
if (-not (Test-GitCommit $PatchRef)) {
    throw "Patch ref '$PatchRef' was not found."
}

$patchCount = (& git rev-list --count $PatchRange).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "Patch range '$PatchRange' is not valid."
}
if ($patchCount -eq "0") {
    throw "Patch range '$PatchRange' did not resolve to any commits."
}

Run-Git @("config", "rerere.enabled", "true")
Run-Git @("config", "rerere.autoupdate", "true")
Run-Git @("switch", "-C", $WorkBranch, $ReleaseTag)

Invoke-Git @(
    "cherry-pick",
    "--rerere-autoupdate",
    "-Xrenormalize",
    $PatchRange
)
$cherryPickExit = $LASTEXITCODE
if ($cherryPickExit -ne 0) {
    Write-Host ""
    Write-Warning "Codexx patch range did not apply cleanly."
    $conflicts = @(& git diff --name-only --diff-filter=U)
    if ($conflicts.Count -gt 0) {
        Write-Host "Conflicted files:"
        $conflicts | ForEach-Object { Write-Host "  $_" }
    }
    Write-Host ""
    Write-Host "Resolve conflicts, run 'git add <files>', then 'git cherry-pick --continue'."
    Write-Host "To abandon this attempt, run 'git cherry-pick --abort'."
    throw "git cherry-pick failed with exit code $cherryPickExit"
}

Write-Host ""
Write-Host "Codexx patch range applied on $ReleaseTag."
Write-Host "Run checks, update docs/version references if needed, then push $WorkBranch."
