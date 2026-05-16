# Implementation Plan

This plan ports the useful `codex_v2` wrapper behavior, especially the v5 and
v6 modes, into a local OpenAI Codex fork as a small native extension layer. The
fork should expose the behavior through normal `codex` flags first. `codex-v5`
and `codex-v6` should become compatibility aliases for documented flag
combinations, not independent implementations.

## Success Criteria

- Prompt-bearing commands can be run as real Codex goals instead of text that
  merely starts with `/goal`.
- A session can resume old work in a fresh session with bounded context/status
  files, including `codex resume <session>` handoffs and automatic recovery from
  remote compact/context failures.
- Non-interactive runs can queue repeated follow-up prompts and stop after a
  bounded number of successful turns or an early-stop condition.
- v5 and v6 compatibility commands continue to exist, but only as aliases or
  profiles over the same native flag surface.
- Local changes stay isolated enough that future upstream Codex releases can be
  merged with routine conflict resolution.

## Feature Distillation

The three required features are the right core:

1. Real goal starts for every prompt-bearing command.
2. Fresh-session resume/recovery with compact context and status files.
3. Autoprompting/recursive non-interactive work over a bounded turn loop.

Important additions from the existing v5/v6 wrappers:

- Workspace and thread ownership checks before any automatic recovery uses old
  state, rollout data, or log lines. This prevents one Codex session from
  recovering with another workspace's goal.
- Bounded handoff files instead of transcript-sized prompts.
- Old-session resume hints before replacement starts. Any fresh resume or
  automatic recovery that starts a new thread should first print a
  stock-compatible `codex resume <thread-id>` command for the source/failed
  session, so a crash during restart still leaves a recovery handle in
  scrollback or JSONL output.
- Clean output controls, especially agent-message-only stdout for app-server
  runs and optional capture files for diagnostics, commands, token usage, and
  raw protocol traffic.
- Explicit first/last meaningful goal selection when resuming a session, while
  skipping resume-only meta goals, generated follow-up scaffolds, and short
  session-reference objectives.
- Treat thread previews as display metadata only. If a source thread has no
  stored goal, generated handoffs may use a thread name or a short
  session-reference objective, but must not reuse raw preview text as the
  recovered goal.
- Strict retry accounting: recovery retries do not count as successful turns.
- Compatibility shims for `codex-v5` and `codex-v6`, plus escape hatches to run
  close to upstream native behavior.

## Proposed Native Flag Surface

Use one native option model, then define v5/v6 as profiles over it.

Core goal flags:

- `--goal` or default-on in this fork: send the prompt through real goal
  creation.
- `--no-goal`: escape hatch for exact upstream prompt behavior.
- `--goal-transport app-server|native`: app-server goal path for direct goal
  turns; native path for ordinary upstream execution where supported.
- `--profile v5|v6`: internal profile selector used by aliases and tests.

Resume/recovery flags:

- `--fresh-resume`: resume old work by starting a new goal session instead of
  loading the old session directly.
- `--context-file <file>`: write/read bounded recovery context.
- `--status`: tell the agent to use the default session-scoped status file.
- `--status-file <file>`: tell the agent to keep durable status current.
- `--no-status`: suppress default profile status files when an explicit status
  file is not provided.
- `--handoff-dir <dir>`: default parent for context/status/session-chain files.
- `--first-goal` and `--last-goal`: select the first or last meaningful stored
  goal from a source session.
- `--auto-recover` and `--no-auto-recover`: enable/disable compact/context
  failure recovery.
- `--retries <n>`: maximum recovery restarts per planned turn.
- `--context-chars <n>`: cap handoff transcript excerpts.

Autoprompt flags:

- `-t, --turns <n>`: run up to `n` successful goal turns.
- `-n, --next <prompt>`: follow-up goal after the first successful turn.
- `-r, --repeat`: repeat the original prompt for follow-up turns.
- `-b, --carry`: combine the original prompt with the follow-up prompt.
- `-e, --early-stopping`: allow stop-token based completion.
- `--stop-token <token>`: defaults to `<NOTHING LEFT TO DO>`.

Output/capture flags:

- `--clean-output`: print agent-message text only where the transport supports
  it.
- `--output-file <file>` and `--output-mode all|agent|commands|diagnostics|usage`.
- `--save-all <file>`, `--save-agent <file>`, and `--save-commands <file>`.

Compatibility profiles:

- `codex-v5 ...` maps to `codex --profile v5 --goal
  --goal-transport app-server --clean-output --auto-recover ...`.
- `codex-v6 ...` maps to `codex --profile v6 --goal --goal-transport native
  --fresh-resume --auto-recover ...`, with generated recovery/resume handoffs
  using the app-server goal path.

## Fork Architecture

Keep upstream-facing edits small and route behavior through new modules.

- Add one CLI option struct for the new flags, for example
  `GoalRunOptions`, and convert aliases/profiles into that struct early.
- Add a `goal_runner` module that owns goal creation, thread start/resume,
  turn execution, stdout filtering, and retry accounting.
- Add a `handoff` module that owns context/status/session-chain file paths,
  bounded summaries, first/last meaningful goal selection, and prompt assembly.
- Add a `recovery` module that owns compact/context failure detection,
  ownership checks, and recovery decisions.
- Keep thin adapter code at existing upstream call sites. Prefer calling
  upstream APIs over changing auth, model/provider, sandbox, protocol, or TUI
  internals.
- If upstream lacks a needed event, add the smallest possible internal event or
  adapter and document it in `docs/upstream-map.md`.

## Phase 0: Source Acquisition And Branch Shape

Goal: place upstream OpenAI Codex source at `./upstream` and establish a merge
workflow.

Accepted sources:

- `git clone https://github.com/openai/codex.git upstream`
- source archive for tag `rust-v0.130.0`
- an authenticated or mirrored clone approved by the user

Validation:

- `upstream/LICENSE` exists and is Apache-2.0.
- `upstream/codex-rs/Cargo.toml` exists.
- `upstream/codex-rs/cli` exists.
- `upstream/package.json` or release metadata identifies the selected target
  tag.

Branch policy:

- Keep an unmodified upstream branch or directory.
- Put local changes on a dedicated branch such as `tarchibald/goal-continuity`.
- Commit wrapper-spec translation, native flag plumbing, recovery, and packaging
  as separate commits so future upstream merges can isolate conflicts.

## Phase 1: Upstream Map

Goal: locate exact Rust APIs for the behavior we need before modifying them.

Deliver `docs/upstream-map.md` with:

- CLI parsing and command dispatch entrypoints.
- Thread start, thread resume, and goal creation APIs.
- App-server `thread/goal/set` implementation path.
- Native TUI/exec turn execution and stdout rendering path.
- State database and rollout JSONL locations and schemas.
- Compact/context failure reporting path, including remote compact task errors.
- Minimal build/test command that passes before local modifications.

## Phase 2: Flag And Profile Plumbing

Goal: add the standard flags without changing behavior yet.

Deliverables:

- Native parser support for the flags listed above.
- `--profile v5` and `--profile v6` expanding into explicit option defaults.
- `codex-v5` and `codex-v6` shims or binary aliases that call the same codepath.
- Dry-run/debug output that shows the resolved profile and options for tests.

Acceptance:

- Aliases do not have their own behavior branches beyond profile selection.
- Existing upstream commands still work with no new flags.
- `--no-goal` and `--no-auto-recover` provide escape hatches.

## Phase 3: Real Goal Starts

Goal: make prompt-bearing commands start as real goals.

Deliverables:

- Root prompt and `exec` prompt use internal goal creation when `--goal` is on.
- Resume/fork handoffs use real goal creation for generated objectives.
- No implementation relies on argv text beginning with `/goal`.
- Clean stdout is available for app-server goal runs.

Acceptance:

- Tests prove the persisted first user message is a real goal turn.
- Tool output is muted from stdout by default in clean-output mode.
- Failure to initialize, start a thread, set a goal, or complete a turn exits as
  a failed goal turn.

## Phase 4: Fresh Resume And Recovery Handoffs

Goal: resume old work in a new session with bounded files.

Deliverables:

- `codex resume <session> --fresh-resume --first-goal|--last-goal` creates a new
  goal session from the source session's meaningful goal.
- The source session's `codex resume <thread-id>` command is printed before the
  new goal session is started.
- Context/status/session-chain files are written under the active workspace,
  defaulting to `.codex/sessions/<session>/` or another documented fork path.
- Promptless recovery is allowed only with verified same-workspace thread
  context.
- Generated recovery prompts point at files and omit full transcripts.
- Objective fallback never uses raw thread preview text; previews may contain
  generated handoff prompts or transcript fragments instead of the user's goal.

Acceptance:

- Other-workspace compact/context failures are ignored.
- Threadless global compact/context log lines do not trigger automatic recovery.
- Recovery fails closed when ownership cannot be proven.
- No-goal source threads resume with a title/name-derived or short
  session-reference objective, not copied preview text.

## Phase 5: Automatic Compact/Context Recovery

Goal: recover from active-session compact/context failures without losing the
goal.

Deliverables:

- Detect remote compact task errors and local context/compaction failures.
- Stop or detach the failed run cleanly.
- Write bounded recovery context.
- Print the failed session's `codex resume <thread-id>` command.
- Start a fresh goal session with the recovered or original goal.
- Keep recovery retries from counting as successful turns.

Acceptance:

- A compact failure during a supervised run starts a fresh goal session.
- Recovery uses the explicit resumed thread when present.
- Recovery diagnostics exclude unrelated global log noise.
- Exhausted retries produce a clear failure with the relevant muted diagnostics.

## Phase 6: Autoprompt Turn Loop

Goal: support bounded recursive work in non-interactive sessions.

Deliverables:

- `--turns`, `--next`, `--repeat`, `--carry`, `--early-stopping`, and
  `--stop-token`.
- Turn loop runs only for non-interactive or explicitly supervised sessions.
- Status/context files are referenced on every generated follow-up goal.
- The next prompt can be queued before the run starts and repeated until the
  turn limit or early-stop condition.

Acceptance:

- `--turns N` counts successful goal turns only.
- Recovery restarts do not decrement the remaining successful turn budget.
- Early-stop tokens stop after the current turn and only when the turn
  completed usefully.

## Phase 7: Capture, Usage, And Compatibility

Goal: preserve practical wrapper affordances that are not part of the core
continuity loop but matter for operations.

Deliverables:

- Capture modes for raw traffic, agent text, commands, diagnostics, and usage.
- Optional token usage reporting if upstream exposes equivalent data.
- Compatibility with old v5/v6 flag names where practical, implemented as
  aliases to native flags.
- Documentation showing old wrapper commands and their new `codex` equivalents.

Acceptance:

- Existing `codex_v2` v5/v6 acceptance cases are either translated to Rust tests
  or explicitly marked as obsolete with a reason.
- Capture files are bounded where they are used as recovery context.

## Phase 8: Packaging And Update Workflow

Goal: install the fork without blocking future upstream merges.

Deliverables:

- Install script for the forked `codex` binary plus optional `codex-v5` and
  `codex-v6` aliases.
- PATH verification and rollback instructions.
- Merge checklist for a future upstream release:
  - update upstream branch/tag;
  - run upstream tests before replaying local patches;
  - resolve conflicts only in the thin adapter and local extension modules;
  - rerun goal/recovery/autoprompt acceptance tests;
  - update `docs/upstream-map.md` when touched upstream APIs move.

## Test Plan

Translate the existing `codex_v2/test/codex-v2.test.js` coverage into Rust
unit/integration tests where possible, prioritizing:

- v5 concise turn-loop defaults and clean stdout.
- v5 first/last goal recovery from rollout/state.
- v5 first/last goal recovery skipping generated continuation scaffolds.
- v5 status-file instructions on initial and follow-up goals.
- v5 recovery attempts not counting as successful turns.
- v5 resume compact failure starting a fresh goal session.
- v6 native-first default behavior and default YOLO policy if retained.
- v6 fresh resume with session-scoped status/context/chain files.
- v6 compact recovery for inherited-stdio root sessions.
- v6 ignoring compact/context errors from another workspace.
- v6 omitting generated recovery prompts from later recovery context.
- v6 refusing to seed fallback objectives from raw thread preview text.

## Non-Goals For First Cut

- Do not modify auth behavior.
- Do not fork model/provider logic.
- Do not alter sandbox policy semantics beyond existing v5/v6 defaults.
- Do not replace upstream native TUI internals unless a small event hook is
  required for owned recovery.
- Do not publish a GitHub fork until the local source build and acceptance tests
  are proven.
