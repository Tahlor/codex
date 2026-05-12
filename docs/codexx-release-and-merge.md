# Codexx Release And Merge Notes

This fork keeps the public install surface separate from the stock OpenAI CLI:

- package name: `codexx`
- installed commands: `codexx`, `codex-v5`, `codex-v6`, `codexx-v5`, `codexx-v6`
- no installed `codex` command

The native binary inside platform payloads can still be named `codex`; the npm
bin map and standalone installers expose only the names above.

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
     "codex-v5": "bin/codex-v5.js",
     "codex-v6": "bin/codex-v6.js",
     "codexx-v5": "bin/codex-v5.js",
     "codexx-v6": "bin/codex-v6.js"
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
forward-port:

```powershell
.\scripts\merge-codexx-release.ps1 -ReleaseTag rust-v0.131.0
```

The helper fetches upstream tags, creates `codexx/rust-v0.131.0`, and
cherry-picks the Codexx patch range. Resolve conflicts if any, then run:

```powershell
cargo check -p codex-cli --bin codex --message-format short
cargo check -p codex-tui --tests --message-format short
cargo check -p codex-cli --tests --message-format short
cargo check -p codex-exec --lib --message-format short
node --check codex-cli\bin\codex.js
node --check codex-cli\bin\codex-v5.js
node --check codex-cli\bin\codex-v6.js
```

If the patch branch name changes, pass `-PatchRef` and `-BaseTag`, or pass an
explicit `-PatchRange`.
