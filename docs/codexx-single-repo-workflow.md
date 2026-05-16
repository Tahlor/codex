# Codexx Single-Repo Workflow

Codexx should be one Git checkout, not a wrapper repo containing another repo.

## Canonical Repository

Use this as the only Codexx source checkout:

```powershell
gitt clone -b codexx-goal-profiles-rust-v0.130.0 git@github.com:Tahlor/codex.git C:\Users\tarchibald\github\codexx
Set-Location C:\Users\tarchibald\github\codexx
```

On machines without the local `gitt` wrapper, use `git` in the same command
positions.

Do not clone or work from `codex-openai-fork`, and do not create an `upstream`
subdirectory. The OpenAI project is tracked as a remote, not as a nested folder.

## Expected Remotes

For a fresh clone from `Tahlor/codex`, `origin` should be your Codexx fork:

```powershell
gitt remote -v
```

Add OpenAI as the upstream source remote:

```powershell
gitt remote add openai git@github.com:openai/codex.git
gitt fetch openai --tags
```

Expected shape:

```text
origin  git@github.com:Tahlor/codex.git
openai  git@github.com:openai/codex.git
```

## Run From Source

Until `codexx` is published as npm packages or release artifacts, install from
the source checkout:

```powershell
Set-Location C:\Users\tarchibald\github\codexx\codex-rs
cargo build -p codex-cli --bin codex
Set-Location C:\Users\tarchibald\github\codexx
npm link .\codex-cli
codexx-v6 --version
```

The JavaScript launcher checks the checkout's Rust debug build before falling
back to packaged platform payloads, so `npm link .\codex-cli` is enough for a
source-built local install.

## Merge A Future OpenAI Release

Keep Codexx as a small patch stack on top of an OpenAI release tag.

Example for a future `rust-v0.131.0` release:

```powershell
Set-Location C:\Users\tarchibald\github\codexx
gitt status --short
gitt fetch origin
gitt fetch openai --tags

.\scripts\merge-codexx-release.ps1 `
  -ReleaseTag rust-v0.131.0 `
  -UpstreamRemote openai `
  -BaseTag rust-v0.130.0 `
  -PatchRef codexx-goal-profiles-rust-v0.130.0 `
  -WorkBranch codexx-goal-profiles-rust-v0.131.0
```

The helper creates the work branch from the new OpenAI release tag and
cherry-picks the Codexx patch range `rust-v0.130.0..codexx-goal-profiles-rust-v0.130.0`.

If conflicts occur:

```powershell
gitt status --short
# edit conflicted files
gitt add <resolved-files>
gitt cherry-pick --continue
```

To abandon a failed forward-port:

```powershell
gitt cherry-pick --abort
gitt switch codexx-goal-profiles-rust-v0.130.0
gitt branch -D codexx-goal-profiles-rust-v0.131.0
```

## Required Checks Before Pushing

Run these from the single Codexx checkout:

```powershell
Set-Location C:\Users\tarchibald\github\codexx
node --check codex-cli\bin\codex.js
node --check codex-cli\bin\codex-v5.js
node --check codex-cli\bin\codex-v6.js
node --check codex-cli\bin\codex-v7.js

Set-Location C:\Users\tarchibald\github\codexx\codex-rs
cargo test -p codex-tui auto_fresh_restart --lib
cargo check -p codex-cli -p codex-tui -p codex-core -p codex-exec
```

Then commit and push:

```powershell
Set-Location C:\Users\tarchibald\github\codexx
gitt status --short
gitt add -A
gitt commit -m "Forward-port codexx to rust-v0.131.0"
gitt push -u origin codexx-goal-profiles-rust-v0.131.0
```

## Release Install Later

The easy install target is:

```powershell
npm install -g codexx
codexx-v6
```

That requires publishing the Codexx npm meta package and platform payloads. The
source workflow above is the current reliable path before publishing.
