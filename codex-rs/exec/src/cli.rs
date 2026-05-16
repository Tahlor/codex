use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use clap::ValueEnum;
use codex_utils_cli::CliConfigOverrides;
use codex_utils_cli::SharedCliOptions;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    override_usage = "codexx exec [OPTIONS] [PROMPT]\n       codexx exec [OPTIONS] <COMMAND> [ARGS]"
)]
pub struct Cli {
    /// Action to perform. If omitted, runs a new non-interactive session.
    #[command(subcommand)]
    pub command: Option<Command>,

    #[clap(flatten)]
    pub goal: GoalCliOptions,

    #[clap(flatten)]
    pub shared: ExecSharedCliOptions,

    /// Allow running Codex outside a Git repository.
    #[arg(long = "skip-git-repo-check", global = true, default_value_t = false)]
    pub skip_git_repo_check: bool,

    /// Run without persisting session files to disk.
    #[arg(long = "ephemeral", global = true, default_value_t = false)]
    pub ephemeral: bool,

    /// Do not load `$CODEX_HOME/config.toml`; auth still uses `CODEX_HOME`.
    #[arg(long = "ignore-user-config", global = true, default_value_t = false)]
    pub ignore_user_config: bool,

    /// Do not load user or project execpolicy `.rules` files.
    #[arg(long = "ignore-rules", global = true, default_value_t = false)]
    pub ignore_rules: bool,

    /// Legacy compatibility trap for the removed `--full-auto` flag.
    #[arg(
        long = "full-auto",
        hide = true,
        global = true,
        default_value_t = false,
        conflicts_with = "dangerously_bypass_approvals_and_sandbox"
    )]
    pub removed_full_auto: bool,

    /// Path to a JSON Schema file describing the model's final response shape.
    #[arg(long = "output-schema", value_name = "FILE")]
    pub output_schema: Option<PathBuf>,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Specifies color settings for use in the output.
    #[arg(long = "color", value_enum, default_value_t = Color::Auto)]
    pub color: Color,

    /// Print events to stdout as JSONL.
    #[arg(
        long = "json",
        alias = "experimental-json",
        default_value_t = false,
        global = true
    )]
    pub json: bool,

    /// Specifies file where the last message from the agent should be written.
    #[arg(
        long = "output-last-message",
        short = 'o',
        value_name = "FILE",
        global = true
    )]
    pub last_message_file: Option<PathBuf>,

    /// Initial instructions for the agent. If not provided as an argument (or
    /// if `-` is used), instructions are read from stdin. If stdin is piped and
    /// a prompt is also provided, stdin is appended as a `<stdin>` block.
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,
}

impl std::ops::Deref for Cli {
    type Target = SharedCliOptions;

    fn deref(&self) -> &Self::Target {
        &self.shared.0
    }
}

impl std::ops::DerefMut for Cli {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared.0
    }
}

impl Cli {
    pub fn removed_full_auto_warning(&self) -> Option<&'static str> {
        if self.removed_full_auto {
            return Some(
                "warning: `--full-auto` is deprecated; use `--sandbox workspace-write` instead.",
            );
        }

        None
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct GoalCliOptions {
    /// Use a goal profile: v5 turn loops, v6 fresh-resume recovery.
    #[arg(long = "mode", value_enum, global = true)]
    pub mode: Option<GoalMode>,

    /// Create a real thread goal before sending a prompt.
    #[arg(long = "goal", global = true, default_value_t = false)]
    pub goal: bool,

    /// Do not create a goal automatically.
    #[arg(long = "no-goal", global = true, default_value_t = false)]
    pub no_goal: bool,

    /// Start a new goal thread from resume context.
    #[arg(long = "fresh-resume", global = true, default_value_t = false)]
    pub fresh_resume: bool,

    /// Put fresh-resume context in the first message and keep the goal objective clean.
    #[arg(
        long = "fresh-resume-context-in-first-message",
        alias = "resume-context-in-first-message",
        global = true,
        default_value_t = false
    )]
    pub fresh_resume_context_in_first_message: bool,

    /// Status file to read, create, and keep current.
    #[arg(long = "status-file", value_name = "FILE", global = true)]
    pub status_file: Option<PathBuf>,

    /// Use the default session-scoped status file.
    #[arg(long = "status", global = true, default_value_t = false)]
    pub status: bool,

    /// Disable the default session-scoped status file.
    #[arg(
        long = "no-status",
        global = true,
        default_value_t = false,
        conflicts_with_all = ["status", "status_file"]
    )]
    pub no_status: bool,

    /// Context file to read before continuing.
    #[arg(long = "context-file", value_name = "FILE", global = true)]
    pub context_file: Option<PathBuf>,

    /// Directory for default handoff files.
    #[arg(long = "handoff-dir", value_name = "DIR", global = true)]
    pub handoff_dir: Option<PathBuf>,

    /// Maximum successful goal turns to run.
    #[arg(short = 't', long = "turns", value_name = "N", global = true)]
    pub turns: Option<usize>,

    /// Follow-up prompt after the first successful turn.
    #[arg(short = 'n', long = "next", value_name = "PROMPT", global = true)]
    pub next: Option<String>,

    /// Use the original prompt for follow-up turns.
    #[arg(short = 'r', long = "repeat", global = true, default_value_t = false)]
    pub repeat: bool,

    /// Combine the original and follow-up prompts.
    #[arg(short = 'b', long = "carry", global = true, default_value_t = false)]
    pub carry: bool,

    /// Stop turn loops when the final answer has the stop token.
    #[arg(
        short = 'e',
        long = "early-stopping",
        global = true,
        default_value_t = false
    )]
    pub early_stopping: bool,

    /// Final-answer token meaning no useful work remains.
    #[arg(long = "stop-token", value_name = "TOKEN", global = true)]
    pub stop_token: Option<String>,

    /// Fresh-thread recovery attempts per planned turn.
    #[arg(long = "retries", value_name = "N", global = true)]
    pub retries: Option<usize>,

    /// Recover the first meaningful stored goal.
    #[arg(long = "first-goal", global = true, default_value_t = false)]
    pub first_goal: bool,

    /// Recover the last meaningful stored goal.
    #[arg(long = "last-goal", global = true, default_value_t = false)]
    pub last_goal: bool,

    /// Disable automatic fresh-thread recovery.
    #[arg(long = "no-auto-recover", global = true, default_value_t = false)]
    pub no_auto_recover: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum GoalMode {
    V5,
    V6,
    V7,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGoalCliOptions {
    pub mode: Option<GoalMode>,
    pub goal: bool,
    pub fresh_resume: bool,
    pub fresh_resume_context_in_first_message: bool,
    pub status_file: Option<PathBuf>,
    pub default_status_file: bool,
    pub context_file: Option<PathBuf>,
    pub handoff_dir: Option<PathBuf>,
    pub turns: usize,
    pub next: Option<String>,
    pub repeat: bool,
    pub carry: bool,
    pub early_stopping: bool,
    pub stop_token: String,
    pub retries: usize,
    pub first_goal: bool,
    pub last_goal: bool,
    pub auto_recover: bool,
}

impl GoalCliOptions {
    pub fn resolve(&self) -> Result<ResolvedGoalCliOptions, String> {
        if self.first_goal && self.last_goal {
            return Err("--first-goal and --last-goal cannot be used together".to_string());
        }

        let profile_goal = matches!(self.mode, Some(GoalMode::V5 | GoalMode::V6 | GoalMode::V7));
        let fresh_resume_context_in_first_message =
            self.fresh_resume_context_in_first_message || matches!(self.mode, Some(GoalMode::V7));
        let fresh_resume = (self.fresh_resume
            || fresh_resume_context_in_first_message
            || matches!(self.mode, Some(GoalMode::V6 | GoalMode::V7)))
            && !self.no_goal;
        let goal = (self.goal || profile_goal || fresh_resume) && !self.no_goal;
        let turns = self.turns.unwrap_or(1);
        if turns == 0 {
            return Err("--turns must be greater than 0".to_string());
        }
        if self
            .stop_token
            .as_deref()
            .is_some_and(|token| token.is_empty())
        {
            return Err("--stop-token must not be empty".to_string());
        }
        let retries = self.retries.unwrap_or(2);
        let repeat = self.repeat;
        let carry = self.carry || (!repeat && self.next.is_some());
        let auto_recover = !self.no_auto_recover
            && (matches!(self.mode, Some(GoalMode::V5 | GoalMode::V6 | GoalMode::V7))
                || self.retries.is_some());
        let default_status_file = !self.no_status
            && (self.status
                || self.handoff_dir.is_some()
                || auto_recover
                || turns > 1
                || matches!(self.mode, Some(GoalMode::V5 | GoalMode::V6 | GoalMode::V7)));

        Ok(ResolvedGoalCliOptions {
            mode: self.mode,
            goal,
            fresh_resume,
            fresh_resume_context_in_first_message,
            status_file: self.status_file.clone(),
            default_status_file,
            context_file: self.context_file.clone(),
            handoff_dir: self.handoff_dir.clone(),
            turns,
            next: self.next.clone(),
            repeat,
            carry,
            early_stopping: self.early_stopping,
            stop_token: self
                .stop_token
                .clone()
                .unwrap_or_else(|| "<NOTHING LEFT TO DO>".to_string()),
            retries,
            first_goal: self.first_goal,
            last_goal: self.last_goal,
            auto_recover,
        })
    }
}

#[derive(Debug, Default)]
pub struct ExecSharedCliOptions(SharedCliOptions);

impl ExecSharedCliOptions {
    pub fn into_inner(self) -> SharedCliOptions {
        self.0
    }
}

impl std::ops::Deref for ExecSharedCliOptions {
    type Target = SharedCliOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ExecSharedCliOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Args for ExecSharedCliOptions {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        mark_exec_global_args(SharedCliOptions::augment_args(cmd))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        mark_exec_global_args(SharedCliOptions::augment_args_for_update(cmd))
    }
}

impl FromArgMatches for ExecSharedCliOptions {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        SharedCliOptions::from_arg_matches(matches).map(Self)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        self.0.update_from_arg_matches(matches)
    }
}

fn mark_exec_global_args(cmd: clap::Command) -> clap::Command {
    cmd.mut_arg("model", |arg| arg.global(true))
        .mut_arg("dangerously_bypass_approvals_and_sandbox", |arg| {
            arg.global(true)
        })
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Resume a previous session by id or pick the most recent with --last.
    Resume(ResumeArgs),

    /// Run a code review against the current repository.
    Review(ReviewArgs),
}

#[derive(Args, Debug)]
struct ResumeArgsRaw {
    // Note: This is the direct clap shape. We reinterpret the positional when --last is set
    // so "codex resume --last <prompt>" treats the positional as a prompt, not a session id.
    /// Conversation/session id (UUID) or thread name. UUIDs take precedence if it parses.
    /// If omitted, use --last to pick the most recent recorded session.
    #[arg(value_name = "SESSION_ID")]
    session_id: Option<String>,

    /// Resume the most recent recorded session (newest) without specifying an id.
    #[arg(long = "last", default_value_t = false)]
    last: bool,

    /// Show all sessions (disables cwd filtering).
    #[arg(long = "all", default_value_t = false)]
    all: bool,

    /// Optional image(s) to attach to the prompt sent after resuming.
    #[arg(
        long = "image",
        short = 'i',
        value_name = "FILE",
        value_delimiter = ',',
        num_args = 1
    )]
    images: Vec<PathBuf>,

    /// Prompt to send after resuming the session. If `-` is used, read from stdin.
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    prompt: Option<String>,
}

#[derive(Debug)]
pub struct ResumeArgs {
    /// Conversation/session id (UUID) or thread name. UUIDs take precedence if it parses.
    /// If omitted, use --last to pick the most recent recorded session.
    pub session_id: Option<String>,

    /// Resume the most recent recorded session (newest) without specifying an id.
    pub last: bool,

    /// Show all sessions (disables cwd filtering).
    pub all: bool,

    /// Optional image(s) to attach to the prompt sent after resuming.
    pub images: Vec<PathBuf>,

    /// Prompt to send after resuming the session. If `-` is used, read from stdin.
    pub prompt: Option<String>,
}

impl From<ResumeArgsRaw> for ResumeArgs {
    fn from(raw: ResumeArgsRaw) -> Self {
        // When --last is used without an explicit prompt, treat the positional as the prompt
        // (clap can’t express this conditional positional meaning cleanly).
        let (session_id, prompt) = if raw.last && raw.prompt.is_none() {
            (None, raw.session_id)
        } else {
            (raw.session_id, raw.prompt)
        };
        Self {
            session_id,
            last: raw.last,
            all: raw.all,
            images: raw.images,
            prompt,
        }
    }
}

impl Args for ResumeArgs {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        ResumeArgsRaw::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        ResumeArgsRaw::augment_args_for_update(cmd)
    }
}

impl FromArgMatches for ResumeArgs {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        ResumeArgsRaw::from_arg_matches(matches).map(Self::from)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        *self = ResumeArgsRaw::from_arg_matches(matches).map(Self::from)?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct ReviewArgs {
    /// Review staged, unstaged, and untracked changes.
    #[arg(
        long = "uncommitted",
        default_value_t = false,
        conflicts_with_all = ["base", "commit", "prompt"]
    )]
    pub uncommitted: bool,

    /// Review changes against the given base branch.
    #[arg(
        long = "base",
        value_name = "BRANCH",
        conflicts_with_all = ["uncommitted", "commit", "prompt"]
    )]
    pub base: Option<String>,

    /// Review the changes introduced by a commit.
    #[arg(
        long = "commit",
        value_name = "SHA",
        conflicts_with_all = ["uncommitted", "base", "prompt"]
    )]
    pub commit: Option<String>,

    /// Optional commit title to display in the review summary.
    #[arg(long = "title", value_name = "TITLE", requires = "commit")]
    pub commit_title: Option<String>,

    /// Custom review instructions. If `-` is used, read from stdin.
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Color {
    Always,
    Never,
    #[default]
    Auto,
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
