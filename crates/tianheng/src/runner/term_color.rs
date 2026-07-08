//! Zero-dependency ANSI styling for the human `check` render. Hand-rolled on purpose: a colour
//! crate would trip 天衡's own `restrict_dependencies_to(guibiao, hunyi, louke, serde_json)`
//! self-law, so the shell carries its own handful of SGR constants instead.
//!
//! Colour is **presentation only** — it never changes the verdict, is applied solely to the
//! default human report (never `--format json` / `sarif`, the machine surfaces), and is gated to
//! an interactive terminal that has not set `NO_COLOR`. A pipe, a file, a CI log, or a captured
//! test string is not a terminal, so it stays byte-identical to the un-styled report.
use std::io::IsTerminal;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";

/// Whether the human render wraps fields in ANSI. `Copy` so it threads cheaply through the pure
/// text producer.
#[derive(Clone, Copy)]
pub(crate) struct Style {
    active: bool,
}

impl Style {
    /// Never colour — the byte-stable form the pure text producer and its unit tests use.
    pub(crate) const PLAIN: Style = Style { active: false };

    /// Always colour — used by the interactive path and asserted directly in tests.
    pub(crate) const ACTIVE: Style = Style { active: true };

    /// Colour when standard error is an interactive terminal and `NO_COLOR` is unset or empty (the
    /// widely-honoured convention: per no-color.org, only a **non-empty** `NO_COLOR` suppresses
    /// colour). Everything non-interactive — a pipe, a redirect, a CI log — resolves to
    /// [`Style::PLAIN`], so the machine-facing byte stream never carries escape codes.
    pub(crate) fn detect() -> Style {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        if std::io::stderr().is_terminal() && !no_color {
            Style::ACTIVE
        } else {
            Style::PLAIN
        }
    }

    fn wrap(self, codes: &str, text: &str) -> String {
        if self.active {
            format!("{codes}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    /// The reason — the repair direction — emphasised so it leads the eye.
    pub(crate) fn reason(self, text: &str) -> String {
        self.wrap(BOLD, text)
    }

    /// An enforce-severity header: a failure (red).
    pub(crate) fn enforce(self, text: &str) -> String {
        self.wrap(RED, text)
    }

    /// A warn-severity header: an advisory (yellow), distinct from a failure at a glance.
    pub(crate) fn warn(self, text: &str) -> String {
        self.wrap(YELLOW, text)
    }

    /// A constitution/usage error — a diagnostic voice (red), the exit-2 sibling of a violation.
    pub(crate) fn error(self, text: &str) -> String {
        self.wrap(RED, text)
    }
}
