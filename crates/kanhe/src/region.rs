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

/// A fence delimiter line's character, run length, and whether anything follows the run.
///
/// Markdown fences with either backticks or tildes, three or more. Reading only the backtick form made a `~~~`
/// block count as prose, so a path appearing nowhere but inside one satisfied "reachable from where a reader is
/// sent" — the requirement is about fenced code, not about one spelling of a fence.
///
/// Block structure is not modelled, and the residue divides by direction. None of it is reachable in this
/// repository's tracked Markdown, and every direction below was measured rather than reasoned.
///
/// **Over-excluding** — hides text a reader can see, refusing a conforming document: an unpaired fence indented
/// four or more columns (a paired one behaves correctly), a line opening with an inline code span of three or
/// more backticks, and a fence line inside an open HTML comment span, where the fence check runs first, the
/// comment's `-->` then falls inside the fence, and the rest of the document is dropped.
///
/// **Under-excluding** — lets fenced content count as prose, which is the direction this reader exists to avoid:
/// a fence opened on a **blockquote** or **list-marker** line. Both were left unmodelled deliberately. Handling
/// the blockquote form by stripping a `>` prefix was tried and reverted: the strip cannot know whether a fence
/// is already open, so a quoted run displayed *inside* a fence closed it, and a path shown in a Markdown sample
/// became prose — a worse instance of the same fault. Closing either needs block structure, not a line rule.
///
/// Stated here rather than in a spec deliberately: the register's undeclared-prose direction reads only
/// `openspec/specs/*`, so this is a note to a reader and not a bound claimed and unpinned.
fn fence_run(trimmed: &str) -> Option<Fence> {
    let marker = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let length = trimmed.chars().take_while(|c| *c == marker).count();
    // `marker` is one of two ASCII characters, so the char count is also the byte offset past the run.
    (length >= 3).then(|| Fence {
        marker,
        length,
        bare: trimmed[length..].trim().is_empty(),
    })
}

/// A fence delimiter line: which character opened it, how long the run is, and whether anything follows it.
struct Fence {
    marker: char,
    length: usize,
    /// Nothing but whitespace after the run. A **closing** fence carries no info string, so a run followed by
    /// text is content of the open block rather than its end. Without this the third leg of the same-line
    /// problem stays open and errs in *both* directions at once: an inner ```` ```rust ```` closed the block, so
    /// its contents counted as prose, and the following bare run re-opened a fence that then never closed, so
    /// everything after it was excluded forever.
    bare: bool,
}

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

    /// Prose: outside every fenced block **and** outside every HTML comment span.
    ///
    /// Where a reader is sent. A fence is where a command lives, and an HTML comment is invisible to the reader
    /// the requirement is about — that second exclusion is a measured defect, not a precaution. The comment
    /// **span** is what is excluded, never the line carrying it: see [`Prose::lines`].
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
        self.numbered_lines().map(|(_, line)| line)
    }

    /// Executed lines with their one-based position in the original source.
    ///
    /// A **whole-line** comment is dropped and a **tail** comment is cut, because a comment is not executed
    /// text wherever it sits. Filtering whole lines alone made placement decide the verdict: a bare marker
    /// line naming a document did not satisfy "this holder names its document" while the same name written
    /// after `let n = 1;` did — the same text answering opposite ways. One site noticed and stripped tails by hand; this is that rule with one
    /// implementation.
    ///
    /// The marker is recognised **preceded by whitespace or at line start**, never bare — measured against
    /// this repository rather than reasoned about. Cutting at the first marker corrupts 26 lines here,
    /// including `"https://…"` constants, a string carrying `"/// …"`, and this file's own `comment: "//"`.
    /// Requiring the head to keep non-space content was measured too and separates nothing today, so it is
    /// not adopted.
    ///
    /// **Residue, declared rather than approximated:** a marker preceded by whitespace *inside* a string
    /// literal is cut, because telling one from the other needs the string-literal lexing this tree has
    /// defeated repeatedly. `observer-protocol` already declares that direction. It sits beside this region's
    /// other residues — a fence inside an open HTML comment span, and a comment span opened inside a fence.
    pub fn numbered_lines(&self) -> impl Iterator<Item = (usize, &'a str)> + use<'a> {
        let comment = self.comment;
        self.text
            .lines()
            .enumerate()
            .filter_map(move |(index, line)| {
                if line.trim_start().starts_with(comment) {
                    return None;
                }
                Some((index + 1, cut_tail_comment(line, comment)))
            })
    }

    /// Executed lines laid back out at their **original positions**, blank where one was dropped.
    ///
    /// [`Self::lines`] compacts: dropping a whole-line comment makes its neighbours adjacent. That is
    /// harmless for a caller asking *does any line say X* and wrong for one applying a rule where position
    /// matters — the shell's continuation rule is such a rule, and both callers of this region in this crate
    /// apply it.
    ///
    /// Measured rather than reasoned about. Given
    ///
    /// ```text
    /// echo START \
    /// # comment
    /// --exact ghost
    /// ```
    ///
    /// bash runs `echo START` and then `--exact ghost` as its own command (`--exact: command not found`): the
    /// continuation pulls the comment onto the line, and `#` at a word boundary ends the command there. Over
    /// the compacted lines the backslash instead reaches across the comment and binds `--exact ghost` into the
    /// first invocation — a command bash never runs. A blank at the comment's position ends the continuation
    /// exactly as bash does.
    ///
    /// This exists because the idiom was written twice by hand and the two disagreed: one caller built the
    /// dense array and the other joined the compacted lines, in the same commit. One implementation is the
    /// only arrangement in which they cannot.
    pub fn positioned_lines(&self) -> Vec<&'a str> {
        let mut positioned = vec![""; self.text.lines().count()];
        for (number, line) in self.numbered_lines() {
            positioned[number - 1] = line;
        }
        positioned
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

/// The executed head of `line`: everything before a comment marker that begins a token.
///
/// "Begins a token" means at line start or after whitespace. A marker glued to the character before it is part
/// of something else — `https://`, `"//"`, `"/// …"` — and cutting there would delete executed text, which is
/// the direction the Core Contract forbids.
fn cut_tail_comment<'a>(line: &'a str, comment: &str) -> &'a str {
    let mut from = 0;
    while let Some(offset) = line[from..].find(comment) {
        let at = from + offset;
        let begins_a_token = at == 0
            || line[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
        if begins_a_token {
            return &line[..at];
        }
        from = at + comment.len();
    }
    line
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

/// Prose: outside every fenced block and outside every HTML comment span.
pub struct Prose<'a>(&'a str);

impl<'a> Prose<'a> {
    pub fn contains(&self, needle: &str) -> bool {
        self.lines().any(|line| line.contains(needle))
    }

    /// Prose lines, each carrying only the text a reader sees. A fence opens on a run of three or more backticks
    /// or tildes and closes only on a bare run of the same character, at least as long; an HTML comment spans
    /// from `<!--` to `-->`, which may be one line or several.
    ///
    /// The comment **span** is excised, never the line holding it. The requirement this serves says a path
    /// appearing *only* inside an HTML comment is not a mention — so a line carrying a visible mention *and* a
    /// comment must keep the mention. Dropping the whole line answered a different question and refused a
    /// document that satisfies the rule: a path followed on its own line by an HTML comment counted as
    /// unmentioned, a false refusal where the reader the requirement is about has been served.
    ///
    /// Yields owned text because excision produces a new string; a fully-commented line yields an empty one,
    /// which is what the whole-line drop it replaces already amounted to for every caller.
    pub fn lines(&self) -> impl Iterator<Item = String> + use<'a> {
        let mut fence: Option<(char, usize)> = None;
        let mut commented = false;
        self.0.lines().filter_map(move |line| {
            let trimmed = line.trim_start();
            if let Some(delimiter) = fence_run(trimmed) {
                match fence {
                    None => fence = Some((delimiter.marker, delimiter.length)),
                    // Closes only on its own character, at least as long. A run of the *other* form, or a
                    // shorter one, is content of the open block — which is why the state carries the character
                    // instead of a boolean: toggling on either marker would let a `~~~` shown inside a backtick
                    // block close it, turning the rest of that block into prose. Measured against that form.
                    Some((open_marker, open_length)) => {
                        if delimiter.bare
                            && delimiter.marker == open_marker
                            && delimiter.length >= open_length
                        {
                            fence = None;
                        }
                    }
                }
                return None;
            }
            if fence.is_some() {
                return None;
            }
            // Walk the line, alternating between visible text and comment span. `rest` strictly shrinks on
            // every branch, so this terminates; `commented` carries across lines for a span that does not close
            // on the one it opened.
            let mut visible = String::new();
            let mut rest = line;
            loop {
                if commented {
                    match rest.find("-->") {
                        Some(at) => {
                            commented = false;
                            rest = &rest[at + "-->".len()..];
                        }
                        None => break,
                    }
                } else {
                    match rest.find("<!--") {
                        Some(at) => {
                            visible.push_str(&rest[..at]);
                            commented = true;
                            rest = &rest[at + "<!--".len()..];
                        }
                        None => {
                            visible.push_str(rest);
                            break;
                        }
                    }
                }
            }
            Some(visible)
        })
    }
}

/// The warning a generated document carries in its header.
pub const DO_NOT_EDIT: &str = "Do not edit by hand";

/// Whether a document declares itself generated: the marker, bolded, in its header.
///
/// One definition, because two readers ask it — the projection register, which enumerates generated
/// documents, and the restatement check, which must not judge a projection for naming what it projects.
pub fn declares_itself_generated(header: &Header<'_>) -> bool {
    header.contains(&format!("**{DO_NOT_EDIT}"))
}
