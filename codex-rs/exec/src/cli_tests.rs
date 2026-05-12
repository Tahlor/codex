use super::*;
use pretty_assertions::assert_eq;

#[test]
fn resume_parses_prompt_after_global_flags() {
    const PROMPT: &str = "echo resume-with-global-flags-after-subcommand";
    let cli = Cli::parse_from([
        "codex-exec",
        "resume",
        "--last",
        "--json",
        "--model",
        "gpt-5.2-codex",
        "--dangerously-bypass-approvals-and-sandbox",
        "--skip-git-repo-check",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        PROMPT,
    ]);

    assert!(cli.ephemeral);
    assert!(cli.ignore_user_config);
    assert!(cli.ignore_rules);
    let Some(Command::Resume(args)) = cli.command else {
        panic!("expected resume command");
    };
    let effective_prompt = args.prompt.clone().or_else(|| {
        if args.last {
            args.session_id.clone()
        } else {
            None
        }
    });
    assert_eq!(effective_prompt.as_deref(), Some(PROMPT));
}

#[test]
fn resume_accepts_output_last_message_flag_after_subcommand() {
    const PROMPT: &str = "echo resume-with-output-file";
    let cli = Cli::parse_from([
        "codex-exec",
        "resume",
        "session-123",
        "-o",
        "/tmp/resume-output.md",
        PROMPT,
    ]);

    assert_eq!(
        cli.last_message_file,
        Some(PathBuf::from("/tmp/resume-output.md"))
    );
    let Some(Command::Resume(args)) = cli.command else {
        panic!("expected resume command");
    };
    assert_eq!(args.session_id.as_deref(), Some("session-123"));
    assert_eq!(args.prompt.as_deref(), Some(PROMPT));
}

#[test]
fn parses_config_isolation_flags() {
    let cli = Cli::parse_from([
        "codex-exec",
        "--ignore-user-config",
        "--ignore-rules",
        "summarize",
    ]);

    assert!(cli.ignore_user_config);
    assert!(cli.ignore_rules);
}

#[test]
fn parses_goal_mode_and_turn_loop_flags() {
    let cli = Cli::parse_from([
        "codex-exec",
        "--mode",
        "v5",
        "--status-file",
        ".codex/STATUS.md",
        "--context-file",
        ".codex/RECOVERY_CONTEXT.md",
        "--turns",
        "3",
        "--next",
        "Continue from status",
        "ship it",
    ]);

    let resolved = cli.goal.resolve().expect("goal options should resolve");
    assert_eq!(resolved.mode, Some(GoalMode::V5));
    assert!(resolved.goal);
    assert_eq!(resolved.turns, 3);
    assert_eq!(resolved.next.as_deref(), Some("Continue from status"));
    assert_eq!(
        resolved.status_file,
        Some(PathBuf::from(".codex/STATUS.md"))
    );
    assert_eq!(
        resolved.context_file,
        Some(PathBuf::from(".codex/RECOVERY_CONTEXT.md"))
    );
}

#[test]
fn v6_mode_defaults_to_fresh_resume() {
    let cli = Cli::parse_from(["codex-exec", "--mode", "v6", "resume", "session-id"]);

    let resolved = cli.goal.resolve().expect("goal options should resolve");
    assert_eq!(resolved.mode, Some(GoalMode::V6));
    assert!(resolved.goal);
    assert!(resolved.fresh_resume);
    assert!(resolved.default_status_file);
}

#[test]
fn status_flag_enables_default_status_file_without_profile_mode() {
    let cli = Cli::parse_from(["codex-exec", "--status", "ship"]);

    let resolved = cli.goal.resolve().expect("goal options should resolve");
    assert!(resolved.default_status_file);
}

#[test]
fn no_status_suppresses_profile_default_status_file() {
    let cli = Cli::parse_from(["codex-exec", "--mode", "v5", "--no-status", "ship"]);

    let resolved = cli.goal.resolve().expect("goal options should resolve");
    assert!(!resolved.default_status_file);
}

#[test]
fn goal_options_reject_zero_turns() {
    let cli = Cli::parse_from(["codex-exec", "--turns", "0", "ship"]);

    let err = cli
        .goal
        .resolve()
        .expect_err("zero turns should be rejected");
    assert!(err.contains("--turns must be greater than 0"));
}

#[test]
fn goal_options_reject_empty_stop_token() {
    let cli = Cli::parse_from(["codex-exec", "--stop-token", "", "ship"]);

    let err = cli
        .goal
        .resolve()
        .expect_err("empty stop token should be rejected");
    assert!(err.contains("--stop-token must not be empty"));
}

#[test]
fn removed_full_auto_flag_reports_migration_path() {
    let cli = Cli::parse_from(["codex-exec", "--full-auto", "summarize"]);

    assert_eq!(
        cli.removed_full_auto_warning(),
        Some("warning: `--full-auto` is deprecated; use `--sandbox workspace-write` instead.")
    );
}
