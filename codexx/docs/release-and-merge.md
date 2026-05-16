# Codexx Release And Merge Notes

This fork keeps the public install surface separate from the stock OpenAI CLI:

- package name: `codexx`
- installed commands: `codexx`, `codexxx`, `codex-v5`, `codex-v6`, `codex-v7`, `codexx-v5`, `codexx-v6`, `codexx-v7`
- no installed `codex` command

The native binary inside platform payloads can still be named `codex`; the npm
bin map and standalone installers expose only the names above.

This separation is only for command installation and package storage. Runtime
sessions and generated handoff files stay on the standard `CODEX_HOME` /
`.codex` conventions so an independently installed stock `codex` can resume
threads created through the Codexx aliases.

For a requirements-level inventory of the forked behavior and changed files,
read [Codexx change requirements](./change-requirements.md).

For the local checkout layout, clone and work in one repository only. Do not use
`codex-openai-fork` with a nested `upstream` folder. See
[Codexx single-repo workflow](./single-repo-workflow.md).

## Precompiled Platforms

The npm release staging path builds one meta package plus native payloads for:

- Linux x64 and arm64
- Windows x64 and arm64
- macOS x64 and arm64

For `npm install -g codexx` to work without a local Rust toolchain, publish the
meta package and all platform payload versions produced by:

```powershell
python scripts/stage_npm_packages.py --release-version 0.130.0 --package codex
```

The meta package depends on platform payloads through npm alias optional
dependencies such as `codexx-linux-x64 -> npm:codexx@0.130.0-linux-x64`.

## Release Checklist

1. Build native release artifacts with the upstream `rust-release.yml` workflow
   or an equivalent local build.
2. Stage npm tarballs:

   ```powershell
   python scripts/stage_npm_packages.py --release-version 0.130.0 --package codex
   ```

3. Inspect the staged meta package. It should contain only these bins:

   ```json
   {
     "codexx": "bin/codex.js",
     "codexxx": "bin/codex.js",
     "codex-v5": "bin/codex-v5.js",
     "codex-v6": "bin/codex-v6.js",
     "codex-v7": "bin/codex-v7.js",
     "codexx-v5": "bin/codex-v5.js",
     "codexx-v6": "bin/codex-v6.js",
     "codexx-v7": "bin/codex-v7.js"
   }
   ```

4. Publish every produced tarball. The platform payload versions must exist
   before users install the meta package.
5. If publishing GitHub release assets, include the `codex-npm-<platform>.tgz`
   files. The standalone installers look for those asset names in
   `Tahlor/codex` by default. Set `CODEXX_RELEASE_REPO=owner/repo` to use a
   different release repository.

## Merge Onto A New Upstream Release

Keep the fork changes as a small branch on top of an upstream release tag. To
forward-port in the single Codexx repository, add OpenAI's repo as a remote
named `openai`:

```powershell
gitt remote add openai git@github.com:openai/codex.git
gitt fetch openai --tags
```

Then run:

```powershell
.\codexx\scripts\merge-openai-release.ps1 `
  -ReleaseTag rust-v0.131.0 `
  -UpstreamRemote openai `
  -BaseTag rust-v0.130.0 `
  -PatchRef codexx-goal-profiles-rust-v0.130.0 `
  -WorkBranch codexx-goal-profiles-rust-v0.131.0
```

The helper fetches upstream tags, creates the requested work branch, and
cherry-picks the Codexx patch range with `rerere` enabled and Git's
renormalizing merge strategy. It defaults to the local `gitt` wrapper. On
machines that intentionally use stock Git, pass `-GitCommand git`.

Resolve conflicts if any, then run:

```powershell
cargo check -p codex-cli --bin codex --message-format short
cargo check -p codex-tui --tests --message-format short
cargo check -p codex-cli --tests --message-format short
cargo check -p codex-exec --lib --message-format short
node --check codex-cli\bin\codex.js
node --check codex-cli\bin\codex-v5.js
node --check codex-cli\bin\codex-v6.js
node --check codex-cli\bin\codex-v7.js
```

If the patch branch name changes, pass `-PatchRef` and `-BaseTag`, or pass an
explicit `-PatchRange`. If the target release tag is already available locally
and this machine's GitHub fetch path is blocked, add `-NoFetch`.

## 2026-05-12 Merge Probe

There is no newer stable release than `rust-v0.130.0` as of this probe, but
GitHub lists `rust-v0.131.0-alpha.9` as a pre-release. Direct Git fetches and
codeload archives were blocked on this machine, so the probe used GitHub's
`rust-v0.130.0...rust-v0.131.0-alpha.9.diff` compare output against the files
that overlap the Codexx patch range.

The committed Codexx range forward-ported after resolving two straightforward
TUI conflicts:

- `codex-rs/tui/src/chatwidget/tests/helpers.rs`: keep the upstream
  `ChatWidgetInit` shape and add `initial_user_message_parse_slash: false`.
- `codex-rs/tui/src/lib.rs`: preserve both startup hooks review and Codexx
  fresh-resume goal initialization before calling `App::run`.

The current uncommitted Rust source changes also applied on top of that probe.
`codex-rs/Cargo.lock` conflicted only as release-version metadata; regenerate
or keep the new upstream lockfile during a real release merge instead of
forcing the old `0.130.0` lockfile versions forward.
