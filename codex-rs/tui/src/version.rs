/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-facing fork build label. Keep `CODEX_CLI_VERSION` semver-only because
/// update checks and client metadata parse or transmit it.
pub const CODEXX_CLI_VERSION_DISPLAY: &str = "codexx-local";
