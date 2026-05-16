# v6 Direct Spec

`codex-v6` should be implemented as a compatibility alias/profile over native
Codex flags. The profile should preserve native behavior for normal starts while
making generated recovery/resume handoffs owned, scoped, and goal-correct.

## Invariants

- First normal run preserves native Codex behavior.
- Generated handoffs are real goal turns, not argv text that begins with
  `/goal`.
- A recovery event must be owned by the current process or an explicitly
  resumed thread before it can affect control flow.
- Other-workspace compact/context failures must not terminate the current run,
  enter diagnostics, create recovery files, or seed a new goal.
- Promptless root recovery requires a verified same-workspace thread context.
- Thread previews are display metadata, not authoritative goal sources.
  Generated recovery prompts and transcript fragments must not be recycled as
  fresh objectives.
- Missing ownership evidence must fail closed.

## Required User-Facing Behavior

- `codex --profile v6 "goal"`, `codex-v6 "goal"`, and `codexx-v6 "goal"`
  resolve to the same internal options.
- `codex-v6 "goal"` behaves like native Codex on first run.
- `codexx-v6 "goal"` and `codex-v6 "goal"` must convert the startup prompt
  into a real `/goal <goal>` slash command internally with slash parsing
  enabled. The quoted string must not be sent as an ordinary chat message.
- `codex-v6 exec "goal"` behaves like native Codex on first run.
- `codex-v6 resume <thread-id>` and `codexx-v6 resume <thread-id>` create a
  fresh interactive goal handoff from the source thread instead of relying on
  native resume compaction.
- Before starting a fresh continuation thread, the CLI prints the source
  session as a stock-compatible `codex resume <thread-id>` command so the old
  session remains recoverable if startup fails.
- `--first-goal` and `--last-goal` select first/last meaningful goals from the
  source thread.
- If the source thread has no stored goal and no explicit first/last goal was
  requested, the handoff still starts and derives its objective from the source
  thread title/name when present. If there is no title/name, the objective is a
  short session-reference objective such as "Resume what you were doing in
  Codex session <thread-id>."
- Raw thread preview text must not be used as a fallback objective because it
  can contain generated continuation prompts, assistant status text, stale tool
  choices, or other transcript fragments rather than the user's actual goal.
- `--status` enables the default session-scoped status file for ordinary goal
  runs, `--no-status` suppresses profile defaults, and `--status-file` chooses
  an explicit file.
- Generated files live under `.codex/sessions/<thread-id>/` or a
  user-specified handoff directory.
- Automatic recovery overwrites the session `RECOVERY_CONTEXT.md` with the
  original objective plus the latest bounded handoff instead of accumulating
  every previous context.
- Automatic recovery prints the old session's `codex resume <thread-id>` command
  before starting the replacement thread.
- Continuation hints use stock-compatible `codex resume ...`; `codex-v6`,
  `codexx`, and `codexx-v6` accept the same thread ids when installed.

## Recovery Ownership Rules

- Recovery should use process-local events if upstream exposes them.
- If reading persisted logs is still necessary, log lines must include a thread
  id.
- Explicit resume/fork thread ids are accepted.
- Otherwise, the thread id must resolve through state or rollout metadata to a
  cwd matching the wrapper workspace.
- Threadless global compact errors are ignored for automatic recovery.
- Recovery retries reuse the same session-scoped status/context paths and keep
  only the latest bounded context.

## Likely Rust Touchpoints

After upstream source is available, inspect:

- `codex-rs/tui`
- `codex-rs/core`
- `codex-rs/state`
- `codex-rs/rollout`
- `codex-rs/thread-store`
- `codex-rs/app-server`
- `codex-rs/app-server-protocol`
- `codex-rs/cli`

Expected implementation shape:

- Add explicit v6 command/mode entrypoint in CLI parsing.
- Route resume/fork handoffs through internal goal creation.
- Replace global-log scraping with process-owned compact/context events if
  available.
- If a log fallback remains, enforce workspace ownership before restart.
- Add tests for cross-workspace compact events, promptless root recovery, and
  generated handoff goal content.
- Add tests proving fallback objectives do not reuse raw thread previews.
