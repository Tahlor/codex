# Codexx Deployment Notes

Codexx currently has two install paths:

- Source install for local development and daily use before public artifacts
  are published.
- Published install through the `codexx` npm meta package plus platform payload
  packages.

## Source Install

Use the single checkout documented in
[single-repo workflow](../docs/single-repo-workflow.md). Build the Rust CLI,
then link the JavaScript launcher package:

```powershell
Set-Location C:\Users\tarchibald\github\codexx\codex-rs
cargo build -p codex-cli --bin codex
Set-Location C:\Users\tarchibald\github\codexx
npm link .\codex-cli
codexx-v6 --version
```

## Published Install

The target user-facing install is:

```powershell
npm install -g codexx
codexx-v6
```

That requires publishing the `codexx` meta package and every platform payload
version produced by the npm staging flow. See
[release and merge notes](../docs/release-and-merge.md) for the current
checklist.

## GitHub Release Assets

Standalone installer support expects GitHub release assets named like the
staged `codex-npm-<platform>.tgz` payloads. By default installers look at
`Tahlor/codex`; set `CODEXX_RELEASE_REPO=owner/repo` to use another release
repository.
