//! Which region of a text a property is about, decided once and carried in the type.
//!
//! Every recognizer over text used to take a bare `&str`, and each one re-decided the region by hand. Six defects
//! across two reviews came from that, all one shape — *the corpus was taken to be the whole blob when the property
//! was about a distinguished part of it*:
//!
//!   * a **comment** made a file count as holding a projection (`projection_register.rs`);
//!   * an **HTML comment** satisfied "a reader can find this document", being invisible to a reader;
//!   * `expected_status` in a twin's **header comment** satisfied a property about executed text;
//!   * a `test -f` on a gate's path in the Definition of Done satisfied "this gate is invoked";
//!   * a signature block quoted in a **tag message** satisfied "this tag carries a signature";
//!   * and two of my own probes measured a `usage:` banner and a dragged-along flag rather than the cell.
//!
//! A helper existed — `uncommented()` — and was used by nine of eleven properties. That is the point: the failure
//! was not a missing helper but that **forgetting it was possible**. So a corpus is never handed to a recognizer as
//! `&str`: it arrives as a region, and a recognizer that wants executed text cannot be given the whole file.
//!
//! [`Source::whole`] is the deliberate escape, spelled out so it is greppable. The family already handles `dyn`
//! this way: not forbidden globally, but every appearance visible where it matters.

/// A whole tracked text, from which a region is taken.
pub struct Source(String);

impl Source {
    pub fn of(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Executed Rust text. Rust attributes beginning with `#` remain code.
    pub fn rust(&self) -> Executed<'_> {
        Executed {
            text: &self.0,
            comment: "//",
        }
    }

    /// Executed shell text. Shell comments beginning with `#` are excluded.
    pub fn shell(&self) -> Executed<'_> {
        Executed {
            text: &self.0,
            comment: "#",
        }
    }

    /// Everything above the first `##` heading.
    ///
    /// Where a generated document warns, and where a shell gate declares its contract. A warning below the first
    /// section heading sits where the damage has already been done.
    pub fn header(&self) -> Header<'_> {
        Header(match self.0.find("\n## ") {
            Some(index) => &self.0[..index],
            None => &self.0,
        })
    }

    /// Prose: outside every fenced block **and** every HTML comment.
    ///
    /// Where a reader is sent. A fence is where a command lives, and an HTML comment is invisible to the reader
    /// the requirement is about — that second exclusion is a measured defect, not a precaution.
    pub fn prose(&self) -> Prose<'_> {
        Prose(&self.0)
    }

    /// The unscoped text. Deliberately explicit: a property that is genuinely about the whole blob says so here,
    /// where a reader and a `grep` can both see it.
    pub fn whole(&self) -> &str {
        &self.0
    }
}

/// Executed text with the source language's line-comment marker removed.
pub struct Executed<'a> {
    text: &'a str,
    comment: &'static str,
}

impl<'a> Executed<'a> {
    pub fn lines(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        let comment = self.comment;
        self.text.lines().filter(move |line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with(comment)
        })
    }

    /// Executed lines with their one-based position in the original source.
    pub fn numbered_lines(&self) -> impl Iterator<Item = (usize, &'a str)> + use<'a> {
        let comment = self.comment;
        self.text
            .lines()
            .enumerate()
            .filter_map(move |(index, line)| {
                (!line.trim_start().starts_with(comment)).then_some((index + 1, line))
            })
    }

    /// Whether any executed line contains `needle`.
    pub fn contains(&self, needle: &str) -> bool {
        self.lines().any(|line| line.contains(needle))
    }

    /// Whether any executed line begins with `prefix`, ignoring indentation.
    pub fn starts_a_line_with(&self, prefix: &str) -> bool {
        self.lines()
            .any(|line| line.trim_start().starts_with(prefix))
    }
}

/// A document's header: everything above its first `##` heading.
pub struct Header<'a>(&'a str);

impl<'a> Header<'a> {
    pub fn text(&self) -> &'a str {
        self.0
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.0.contains(needle)
    }

    pub fn lines(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        self.0.lines()
    }
}

/// Prose: outside every fenced block and every HTML comment.
pub struct Prose<'a>(&'a str);

impl<'a> Prose<'a> {
    pub fn contains(&self, needle: &str) -> bool {
        self.lines().any(|line| line.contains(needle))
    }

    /// Prose lines. A fence toggles on any line whose trimmed start is ```` ``` ````; an HTML comment spans from
    /// `<!--` to `-->`, which may be one line or several.
    pub fn lines(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        let mut fenced = false;
        let mut commented = false;
        self.0.lines().filter(move |line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                fenced = !fenced;
                return false;
            }
            if fenced {
                return false;
            }
            // An HTML comment opening and closing on one line hides only that line.
            if trimmed.contains("<!--") && trimmed.contains("-->") {
                return false;
            }
            if trimmed.contains("<!--") {
                commented = true;
                return false;
            }
            if commented {
                if trimmed.contains("-->") {
                    commented = false;
                }
                return false;
            }
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Source;

    #[test]
    fn executed_regions_respect_the_source_language() {
        let rust = Source::of("#[cfg(test)]\n// hidden\nfn guarded() {}\n");
        assert!(rust.rust().contains("#[cfg(test)]"));
        assert!(!rust.rust().contains("hidden"));

        let shell = Source::of("# hidden\nprintf '%s\\n' '# data'\n");
        assert!(!shell.shell().contains("hidden"));
        assert!(shell.shell().contains("# data"));
    }
}
