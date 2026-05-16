# v5 Direct Spec

`codex-v5` should be implemented as a compatibility alias/profile over native
Codex flags. The profile uses Codex app-server internals directly instead of
spawning another `codex` process, but its behavior should be expressed through
the shared native option model described in `implementation-plan.md`.

## Invariants

- Prompt-bearing work is encoded as a real goal turn.
- Goal creation uses the same internal path as app-server `thread/goal/set`; no
  user objective may depend on textual `/goal` prompt emulation.
- Stdout defaults to agent-message text only.
- Tool output and raw protocol diagnostics are not printed unless explicitly
  requested.
- Recovery state is bounded, file-backed, and session-scoped.
- Recovery retries do not count as successful turns.

## Required User-Facing Behavior

- `codex --profile v5 "goal"`, `codex-v5 "goal"`, and `codexx-v5 "goal"`
  resolve to the same internal options.
- `codex-v5 "goal"` starts a goal turn.
- `codex-v5 exec "goal"` starts a non-interactive goal turn.
- `codex-v5 -t N` runs up to `N` successful turns.
- `codex-v5 --next "goal"` supplies the next-turn objective.
- `codex-v5 resume <thread-id> --first-goal` recovers the first stored goal.
- `codex-v5 resume <thread-id> --last-goal` recovers the last meaningful stored
  goal.
- First/last goal recovery must treat thread previews as display metadata only.
  It must not seed a recovered objective from raw preview text.
- `--status` enables the default session-scoped status file, `--no-status`
  suppresses profile defaults, and `--status-file` chooses an explicit file.
- `--status-file`, `--context-file`, and capture flags write bounded files in
  the active workspace without sharing default paths across sessions.
- Generated default files live under `.codex/sessions/<thread-id>/` so stock
  `codex` and Codexx aliases share the same workspace handoff convention.
- Automatic recovery overwrites the session `RECOVERY_CONTEXT.md` with the
  original objective plus the latest bounded handoff instead of accumulating
  every previous context.
- Automatic recovery prints the old session's stock-compatible
  `codex resume <thread-id>` command before starting a replacement thread.

## Recovery Rules

- v5 uses explicit app-server/thread state, not the global native TUI log.
- Generic generated goals such as `Continue the active goal.`, default
  follow-up objectives, and short session-reference scaffolds are ignored when
  selecting first/last meaningful goals.
- If a stored goal is a generated fresh-resume scaffold that wraps a
  `Recovered goal:` section, v5 should recover the inner goal rather than
  treating the scaffold text as the user's objective.
- If recovery cannot prove the intended source thread, v5 must stop with
  diagnostics instead of selecting another thread.
- Failure to initialize, create a thread, set a goal, or complete a turn is a
  failed goal turn.
- Recovery retries keep only the latest bounded context in the session recovery
  file.

## Likely Rust Touchpoints

After upstream source is available, start by inspecting:

- `codex-rs/cli`
- `codex-rs/app-server`
- `codex-rs/app-server-protocol`
- `codex-rs/core`
- `codex-rs/state`
- `codex-rs/rollout`
- `codex-rs/thread-store`

Expected implementation shape:

- Add a v5 command or mode in the CLI command parser.
- Reuse internal app-server goal plumbing instead of JSON-RPC over stdio.
- Add bounded recovery/status file writers as Rust modules.
- Add tests around app-server goal creation, recovery retry accounting, and
  stdout filtering.
- Add tests proving first/last goal recovery skips generated scaffolds and never
  falls back to raw thread preview text.
