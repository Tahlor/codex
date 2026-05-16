# READ THIS: CODEXX CHANGE REQUIREMENTS

This file documents the Codexx branch requirements introduced on top of the
upstream `rust-v0.130.0` source snapshot. It covers the committed branch delta
from upstream base `58573da` through `9a16e62`, plus the current uncommitted
working-tree changes present on 2026-05-12.

## Scope

The Codexx modification changes four user-facing areas:

- package and installer branding from stock `codex` / `@openai/codex` to
  `codexx`;
- compatibility launchers for `codex-v5`, `codex-v6`, `codex-v7`,
  `codexx-v5`, `codexx-v6`, and `codexx-v7`;
- native goal-profile behavior for root CLI, `exec`, and interactive TUI
  resume handoffs;
- release and merge workflow support for carrying the fork over newer upstream
  Codex tags.

The requirements below are written as the expected behavior of this branch.

## Distribution Requirements

### REQ-DIST-001: Public package identity

The npm meta package MUST be named `codexx`, not `@openai/codex`.

Changed artifacts:

- `codex-cli/package.json`
- `codex-cli/scripts/build_npm_package.py`
- `README.md`
- `codex-rs/README.md`

Acceptance evidence:

- `codex-cli/package.json` uses `"name": "codexx"`.
- README install examples use `npm install -g codexx`.
- package staging constants use `CODEX_NPM_NAME = "codexx"`.

### REQ-DIST-002: Installed commands

The public installed command set MUST be:

- `codexx`
- `codexxx`
- `codex-v5`
- `codex-v6`
- `codex-v7`
- `codexx-v5`
- `codexx-v6`
- `codexx-v7`

The fork MUST NOT install a public `codex` command through the Codexx npm
package or standalone installers.

Changed artifacts:

- `codex-cli/package.json`
- `codex-cli/bin/codex.js`
- `codex-cli/bin/codex-v5.js`
- `codex-cli/bin/codex-v6.js`
- `scripts/install/install.sh`
- `scripts/install/install.ps1`

Acceptance evidence:

- npm `bin` maps only the eight Codexx commands above.
- Unix and Windows installers expose the same primary and alias commands.
- release completeness checks require all alias binaries.

### REQ-DIST-003: Platform payload package names

The launcher and package staging MUST use Codexx platform payload names:

- `codexx-linux-x64`
- `codexx-linux-arm64`
- `codexx-darwin-x64`
- `codexx-darwin-arm64`
- `codexx-win32-x64`
- `codexx-win32-arm64`

Changed artifacts:

- `codex-cli/bin/codex.js`
- `codex-cli/scripts/build_npm_package.py`

Acceptance evidence:

- missing optional dependency messages tell users to reinstall `codexx`.
- package staging emits Codexx-named platform payload package metadata.

### REQ-DIST-004: Standalone installer storage and release source

Standalone installers MUST default to Codexx-specific locations and release
sources:

- home directory: `$CODEXX_HOME` or `.codexx`;
- release repository: `Tahlor/codex`, overridable with `CODEXX_RELEASE_REPO`;
- visible install command: `codexx`.

Changed artifacts:

- `scripts/install/install.sh`
- `scripts/install/install.ps1`

Acceptance evidence:

- install scripts use `CODEXX_HOME` and `.codexx`.
- release API URLs use `CODEXX_RELEASE_REPO` or `Tahlor/codex`.
- installer prompts and success messages say Codexx.
- runtime session storage still uses standard `CODEX_HOME` / `~/.codex` so a
  separately installed stock `codex` can see Codexx-created sessions.

### REQ-DIST-005: Update discovery and commands

Update checks, update prompts, and generated update commands MUST point at the
Codexx package and release locations.

Changed artifacts:

- `codex-rs/install-context/src/lib.rs`
- `codex-rs/tui/src/npm_registry.rs`
- `codex-rs/tui/src/update_action.rs`
- `codex-rs/tui/src/update_prompt.rs`
- `codex-rs/tui/src/updates.rs`
- `codex-rs/tui/tooltips.txt`

Acceptance evidence:

- npm registry checks use `https://registry.npmjs.org/codexx`.
- update commands install `codexx@latest`.
- release prompts and latest-release checks use `Tahlor/codex`.
- standalone install-context paths recognize `.codexx` package layouts and
  release directory shape.

## CLI And Profile Requirements

### REQ-CLI-001: Global goal option surface

The root CLI and `exec` command MUST accept one shared goal option surface:

- `--mode v5|v6`
- `--goal`
- `--no-goal`
- `--fresh-resume`
- `--fresh-resume-context-in-first-message`
- `--status`
- `--status-file <file>`
- `--no-status`
- `--context-file <file>`
- `--handoff-dir <dir>`
- `-t, --turns <n>`
- `-n, --next <prompt>`
- `-r, --repeat`
- `-b, --carry`
- `-e, --early-stopping`
- `--stop-token <token>`
- `--retries <n>`
- `--first-goal`
- `--last-goal`
- `--no-auto-recover`

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/cli/src/main.rs`

Acceptance evidence:

- `GoalCliOptions` defines the shared flags.
- root CLI inherits goal options into `exec` subcommands.
- tests cover parsing, inheritance, zero-turn rejection, empty stop-token
  rejection, and resolver behavior rejects conflicting first/last goal flags.

### REQ-CLI-002: Alias profile selection

Launching the binary through `codex-v5` or `codexx-v5` MUST select `--mode v5`
unless the user already supplied `--mode`. Launching through `codex-v6` or
`codexx-v6` MUST select `--mode v6` unless the user already supplied `--mode`.
Launching through `codex-v7` or `codexx-v7` MUST select `--mode v7`, which is
v6 fresh-resume behavior plus first-message resume context.

Changed artifacts:

- `codex-cli/bin/codex.js`
- `codex-rs/cli/src/main.rs`

Acceptance evidence:

- JavaScript launcher prepends `--mode v5` or `--mode v6` based on argv0.
- JavaScript launcher honors an explicit user `--mode`.
- Rust CLI maps alias executable names to `GoalMode::V5` and `GoalMode::V6`.

### REQ-CLI-003: Help and continuation wording

User-visible help MUST use `codexx` as the packaged command name, while
continuation hints MUST use stock-compatible `codex resume` so sessions remain
easy to resume from a separately installed OpenAI Codex CLI.

Changed artifacts:

- `codex-rs/core/src/util.rs`
- `codex-rs/core/src/util_tests.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/exec/src/cli.rs`
- `codex-rs/cli/src/marketplace_cmd.rs`
- `codex-rs/cli/src/mcp_cmd.rs`

Acceptance evidence:

- root and exec usage strings show `codexx`.
- plugin marketplace help tests expect `codexx plugin ...`.
- continuation hints say stock-compatible `codex resume <thread-id>`; the same
  thread id can also be passed to Codexx aliases when installed.

## Goal Profile Requirements

### REQ-GOAL-001: Goal feature enablement

When a profile or goal run creates a real goal, the CLI MUST enable the goals
feature by default, unless the user explicitly set `features.goals=...`.

Changed artifacts:

- `codex-rs/cli/src/main.rs`
- `codex-rs/exec/src/lib.rs`

Acceptance evidence:

- root interactive mode appends `features.goals=true` when needed.
- exec mode extends config overrides with `features.goals=true` when
  `goal_options.goal` is true.
- tests verify explicit user feature overrides are preserved.

### REQ-GOAL-002: Interactive profile prompt handling

Interactive v5/v6 startup prompts MUST be parsed as goal slash commands without
double-prefixing prompts that already start with `/goal`.

Changed artifacts:

- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/cli.rs`

Acceptance evidence:

- prompts become `/goal <prompt>` when needed.
- existing `/goal` prompts are trimmed and preserved.
- promptless interactive profile startup is a no-op.
- `codexx-v6 "goal"` alias startup is covered so the quoted goal is parsed as
  a slash command rather than sent as an ordinary chat message.

### REQ-GOAL-003: `--no-goal` escape hatch

`--no-goal` MUST disable profile-driven goal creation and fresh-resume handoff
behavior.

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/cli/src/main.rs`

Acceptance evidence:

- resolved goal options set `goal = false` when `--no-goal` is present.
- v6 resume tests show `--no-goal` leaves `interactive.fresh_resume` unset.

## Exec Requirements

### REQ-EXEC-001: App-server-backed goal turns

When goal mode is enabled, `exec` MUST create or update the app-server thread
goal before starting a user turn.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`

Acceptance evidence:

- `set_thread_goal_if_enabled` sends `ThreadGoalSet` with active status.
- initial user turns pass a goal objective separate from the full handoff
  prompt.

### REQ-EXEC-002: Non-interactive turn loop

`exec` MUST support bounded follow-up goal turns:

- `--turns N` runs at most `N` successful goal turns;
- `--next` supplies a follow-up prompt;
- `--repeat` repeats the original prompt;
- `--carry` combines original and follow-up prompts;
- default follow-up text is used when neither `--next` nor `--repeat` is set.

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/exec/src/cli_tests.rs`
- `codex-rs/exec/src/lib_tests.rs`

Acceptance evidence:

- `GoalCliOptions::resolve` rejects `--turns 0`.
- `follow_up_prompt` builds repeat, carry, next, and default prompts.
- tests cover turn-loop flag parsing and follow-up prompt behavior.

### REQ-EXEC-003: Early stopping

When `--early-stopping` is set, `exec` MUST stop after a completed turn whose
final agent message contains the configured stop token. The default stop token
MUST be `<NOTHING LEFT TO DO>`.

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/exec/src/lib.rs`

Acceptance evidence:

- empty stop tokens are rejected.
- `should_stop_after_turn` checks the final agent message for the configured
  token.

### REQ-EXEC-004: Automatic fresh-thread recovery

When automatic recovery is enabled and a goal turn fails, `exec` MUST retry in a
fresh thread up to `--retries N`, without counting failed attempts as successful
turns.

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/exec/src/lib.rs`

Acceptance evidence:

- auto recovery is enabled by v5/v6 profiles or explicit `--retries`.
- `start_user_turn_with_recovery` starts a new thread after failed turns.
- recovery attempts write latest bounded context before retrying.

## Handoff And Fresh Resume Requirements

### REQ-HANDOFF-001: Default handoff root

Generated Codexx status and recovery context files MUST default to
`.codex/sessions/<thread-id>/`, matching stock Codex workspace conventions so
resume handoff artifacts remain cross-tool compatible. Codexx-specific
`.codexx` paths are for standalone package installation storage, not runtime
session or handoff state.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/exec/src/lib_tests.rs`
- `codex-rs/tui/src/lib.rs`

Acceptance evidence:

- both exec and TUI use `DEFAULT_GOAL_HANDOFF_ROOT = ".codex"`.
- tests expect `.codex/sessions/.../STATUS.md` and
  `.codex/sessions/.../RECOVERY_CONTEXT.md`.

### REQ-HANDOFF-002: Status file behavior

Status files MUST be session-scoped by default for profile runs, auto recovery,
turn loops, and explicit `--status`, unless suppressed by `--no-status`.
Explicit `--status-file` MUST override the default.

Changed artifacts:

- `codex-rs/exec/src/cli.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/cli/src/main.rs`

Acceptance evidence:

- resolved goal options compute `default_status_file`.
- TUI fresh resume copies source status into continuation status when using
  default paths.
- `--no-status` tests suppress profile default status files.

### REQ-HANDOFF-003: Recovery context behavior

Recovery context files MUST be bounded, session-scoped, and referenced from
generated prompts. Automatic recovery MUST overwrite the latest context instead
of appending transcript history indefinitely.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`

Acceptance evidence:

- `prepare_goal_handoff` and `prepare_fresh_resume_handoff` create
  `RECOVERY_CONTEXT.md`.
- prompts include instructions to read the bounded context path.
- `write_latest_recovery_context` overwrites the context file on recovery.

### REQ-HANDOFF-004: Fresh resume objective recovery

Fresh resume MUST recover the source objective in this order:

- if `--first-goal` is set, use the first non-empty stored goal from rollout
  history;
- if `--last-goal` is set, use the last non-empty stored goal from rollout
  history, falling back to the current app-server goal where supported;
- otherwise, use the current app-server goal, then the last stored rollout goal;
- interactive TUI fresh resume may fall back to the thread name when no stored
  goal exists. It MUST NOT use raw thread preview text as the recovered
  objective because previews can be generated continuation prompts rather than
  the user's actual goal.
- first/last rollout recovery MUST ignore generated continuation scaffolds such
  as default follow-up objectives and short session-reference objectives. If a
  stored goal wraps a `Recovered goal:` section, recovery MUST use the inner
  objective rather than the scaffold text.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/cli/src/main.rs`

Acceptance evidence:

- exec and TUI parse `ThreadGoalUpdated` rollout lines.
- tests cover first/last goal recovery and TUI name/short-reference fallback.
- tests cover generated scaffold filtering and recovered-goal unwrapping.
- conflicting `--first-goal` and `--last-goal` are rejected.

### REQ-HANDOFF-005: Bounded goal objective length

Generated fresh-resume goal objectives MUST stay within
`MAX_THREAD_GOAL_OBJECTIVE_CHARS`. If the recovered objective is too long, the
goal objective MUST fall back to a shorter session-reference objective while
still pointing to bounded context when possible.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/exec/src/lib_tests.rs`
- `codex-rs/tui/src/lib.rs`

Acceptance evidence:

- `bounded_fresh_resume_goal_objective` exists in exec and TUI paths.
- tests verify normal objectives mention recovered goal and context.
- tests verify overlong objectives are shortened and remain within the protocol
  limit.

### REQ-HANDOFF-006: Interactive v6 fresh resume

Interactive v6 resume MUST start a fresh continuation thread instead of loading
the previous thread directly. The new thread MUST receive:

- an active thread goal;
- a startup prompt explaining this is a fresh continuation;
- a bounded recovery context file;
- an optional durable status file.

Before starting the fresh continuation thread, the CLI MUST print a
stock-compatible `codex resume <thread-id>` command for the source session so
the old session can be recovered if continuation startup crashes.

Changed artifacts:

- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/cli.rs`
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/lib.rs`

Acceptance evidence:

- v6 resume is converted into `FreshResumeStartup`.
- `App` starts a new thread, prepares the handoff, sets the thread goal, and
  sends the generated startup prompt.
- With `--fresh-resume-context-in-first-message` or `--mode v7`, the startup
  prompt carries the resume context and asks the model to reply exactly with
  `acknowledge`, while the active goal is set to the recovered objective
  without context-file instructions.
- tests cover the source-session restart message formatting.
- tests cover explicit fresh-resume prompt content and handoff file creation.

### REQ-HANDOFF-007: Old-session restart hints

Any Codexx path that starts a replacement thread from an existing source or
failed thread MUST print the old session before the replacement starts.

Changed artifacts:

- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/app.rs`

Acceptance evidence:

- exec fresh resume emits `Previous session before fresh restart:
  codex resume <thread-id>` before `thread/start`.
- exec automatic recovery emits the same message for the failed thread before
  starting the retry thread.
- interactive v6 fresh resume emits the same message for the source thread
  before starting the continuation thread.
- tests cover message formatting in exec and TUI paths.

## Release And Merge Requirements

### REQ-REL-001: Codexx release documentation

Release documentation MUST describe the Codexx package identity, installed
command set, platform payloads, and publication checklist.

Changed artifacts:

- `codexx/docs/release-and-merge.md`

Acceptance evidence:

- release notes list the package name and eight installed commands.
- release checklist names the expected npm bin map.
- release notes document Codexx platform payload aliases.

### REQ-REL-002: Forward-port helper

The merge helper MUST support forward-porting the Codexx patch range onto a new
upstream tag and provide actionable conflict guidance.

Changed artifacts:

- `codexx/scripts/merge-openai-release.ps1`
- `codexx/docs/release-and-merge.md`

Acceptance evidence:

- helper validates `ReleaseTag`, `BaseTag`, `PatchRef`, and `PatchRange`.
- helper supports `-NoFetch` for environments with blocked fetches.
- helper enables `rerere` and cherry-picks with `--rerere-autoupdate` and
  `-Xrenormalize`.
- helper lists conflicted files and prints `git cherry-pick --continue` /
  `git cherry-pick --abort` guidance.

### REQ-REL-003: Lockfile version metadata

The branch lockfile MUST reflect the workspace package version currently used
by the forked source tree. During a real upstream release merge, lockfile
conflicts SHOULD be resolved by regenerating or keeping the new upstream
lockfile version metadata rather than forcing stale version values forward.

Changed artifacts:

- `codex-rs/Cargo.lock`
- `codexx/docs/release-and-merge.md`

Acceptance evidence:

- current `Cargo.lock` package metadata uses `0.130.0` for workspace crates.
- merge notes call out lockfile conflicts as release-version metadata.

## Test Coverage Requirements

The branch MUST keep focused tests near changed behavior.

Changed test artifacts include:

- `codex-rs/cli/src/main.rs`
- `codex-rs/exec/src/cli_tests.rs`
- `codex-rs/exec/src/lib_tests.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/chatwidget/tests/goal_validation.rs`
- `codex-rs/tui/src/chatwidget/tests/helpers.rs`
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`
- `codex-rs/tui/src/chatwidget/tests/plan_mode.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`
- `codex-rs/tui/src/chatwidget/tests/status_and_layout.rs`
- `codex-rs/tui/src/snapshots/codex_tui__update_prompt__tests__update_prompt_modal.snap`
- `codex-rs/core/src/util_tests.rs`

Required validation commands for this change set:

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

The merge checklist in `codexx/docs/release-and-merge.md` intentionally repeats
these commands so release validation and change validation stay aligned.

## Change Map

- Packaging and installed command names:
  `README.md`, `codex-rs/README.md`, `codex-cli/package.json`,
  `codex-cli/bin/codex.js`, `codex-cli/bin/codex-v5.js`,
  `codex-cli/bin/codex-v6.js`, `codex-cli/bin/codex-v7.js`,
  `codex-cli/scripts/build_npm_package.py`,
  `codex-cli/scripts/README.md`, `scripts/install/install.sh`,
  `scripts/install/install.ps1`.
- Root CLI profile routing:
  `codex-rs/cli/src/main.rs`, `codex-rs/cli/src/marketplace_cmd.rs`,
  `codex-rs/cli/src/mcp_cmd.rs`.
- Exec goal profiles, handoff files, turn loops, and recovery:
  `codex-rs/exec/src/cli.rs`, `codex-rs/exec/src/lib.rs`,
  `codex-rs/exec/src/cli_tests.rs`, `codex-rs/exec/src/lib_tests.rs`.
- TUI fresh resume and goal validation:
  `codex-rs/tui/src/app.rs`, `codex-rs/tui/src/cli.rs`,
  `codex-rs/tui/src/lib.rs`, `codex-rs/tui/src/chatwidget.rs`, TUI tests and
  snapshots.
- Installer context and update wording:
  `codex-rs/core/src/util.rs`, `codex-rs/core/src/util_tests.rs`,
  `codex-rs/install-context/src/lib.rs`, `codex-rs/tui/src/npm_registry.rs`,
  `codex-rs/tui/src/update_action.rs`, `codex-rs/tui/src/update_prompt.rs`,
  `codex-rs/tui/src/updates.rs`, `codex-rs/tui/tooltips.txt`.
- Release maintenance:
  `codexx/docs/change-requirements.md`,
  `codexx/docs/release-and-merge.md`, `codexx/scripts/merge-openai-release.ps1`,
  `codex-rs/Cargo.lock`.
