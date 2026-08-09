//! One kinded refusal, shared by every reaction that has one, and the instrumentation that makes a refusal
//! **site** observable.
//!
//! Two reactions each defined their own `Kind`, `Refusal`, `violation` and `cannot_judge`. Two constructions
//! of one concept is the twin-drift class this repository keeps closing, and here it had a second cost: the
//! contract `rust-self-governance-gates` places on directions — *assert which outcome a shape produces* — was
//! enforced by nothing, and a review sweep measured 24 of 60 construction sites as distinguished by no
//! direction at all.
//!
//! A **site** is one construction of a refusal. It carries two independent contracts: the **kind**, which is
//! what an operator acts on before an irreversible act, and the **message**, which is what tells them where to
//! look. `refusal_bites.rs` holds each site to both by perturbing it and requiring some direction to die.
//!
//! The perturbation happens **here**, at run time, rather than by rewriting source: one build, then a process
//! run per site per perturbation, with nothing on disk changed and so no window in which an interrupted run
//! leaves the tree edited.
//!
//! **`Location::caller()` is read in each constructor's own body.** `#[track_caller]` propagates only through
//! annotated frames, so reading it inside [`instrument`] — which is not annotated, and must not be — would
//! measure this file's own interior: every site would report one location, the sweep would enumerate sixty
//! sites and intercept one, and it would report clean over all of it. The location is therefore read at the
//! top of each constructor and passed down as a value.

#![allow(dead_code)]

use std::io::Write;
use std::panic::Location;
use std::sync::{Mutex, OnceLock};

/// Names the site to perturb, as `<file>:<line>:<mode>` or `ALL:<mode>`, where mode is `kind` or `message`.
pub const MUTANT: &str = "TIANHENG_REFUSAL_MUTANT";

/// Names a file to append each construction's site to, one file per process.
pub const RECORD: &str = "TIANHENG_REFUSAL_RECORD";

/// What a replaced message says. Long and unmistakable, because a direction asserting a substring of it would
/// survive the perturbation and be reported as defending the site.
pub const MUTANT_MESSAGE: &str =
    "TIANHENG-REFUSAL-MUTANT: this message was replaced by the refusal-site sweep";

/// Marks a panic raised by the instrumentation itself.
///
/// The sweep reads a target's failure as *the site was distinguished*. A panic from this file is not that —
/// it is the instrument failing — and a sweep that could not tell them apart would report a site as defended
/// on the strength of its own malfunction, which is a false negative. Every panic here carries this marker so
/// the sweep can refuse instead of concluding.
pub const INSTRUMENT_PANIC: &str = "TIANHENG-REFUSAL-INSTRUMENT-PANIC";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    /// The source disagrees with what it is judged against.
    Violation,
    /// The source could not be read, which is not the same fact.
    CannotJudge,
}

#[derive(Debug, Clone)]
pub struct Refusal {
    pub kind: Kind,
    pub message: String,
}

#[track_caller]
pub fn violation(message: impl Into<String>) -> Refusal {
    let site = Location::caller();
    instrument(
        site,
        Refusal {
            kind: Kind::Violation,
            message: message.into(),
        },
    )
}

#[track_caller]
pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    let site = Location::caller();
    instrument(
        site,
        Refusal {
            kind: Kind::CannotJudge,
            message: message.into(),
        },
    )
}

/// Record this construction, then apply whatever perturbation names it.
///
/// Deliberately **not** `#[track_caller]`: it receives the site rather than asking for it, which is what keeps
/// the location the caller's. Annotating it would not break anything today, but it would make the design read
/// as though either placement worked.
fn instrument(site: &'static Location<'static>, refusal: Refusal) -> Refusal {
    record(site);
    match selector() {
        Some(selector) if selector.names(site) => match selector.mode {
            Mode::Kind => Refusal {
                kind: match refusal.kind {
                    Kind::Violation => Kind::CannotJudge,
                    Kind::CannotJudge => Kind::Violation,
                },
                message: refusal.message,
            },
            Mode::Message => Refusal {
                kind: refusal.kind,
                message: MUTANT_MESSAGE.to_string(),
            },
        },
        _ => refusal,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Kind,
    Message,
}

#[derive(Debug, Clone)]
struct Selector {
    /// `None` names every site — the control proving the injection is wired at all.
    site: Option<(String, u32)>,
    mode: Mode,
}

impl Selector {
    fn names(&self, site: &Location<'_>) -> bool {
        match &self.site {
            None => true,
            Some((file, line)) => site.file() == file && site.line() == *line,
        }
    }
}

/// Parse `<file>:<line>:<mode>` or `ALL:<mode>`, panicking on anything else.
///
/// A malformed selector must not be treated as "no perturbation". A sweep that poisons nothing sees no
/// direction die and reports the site undistinguished — noise rather than a false negative, but still a
/// verdict about the wrong thing. The panic carries [`INSTRUMENT_PANIC`] so the sweep refuses instead.
fn parse_selector(raw: &str) -> Selector {
    let (head, mode) = raw.rsplit_once(':').unwrap_or_else(|| {
        panic!("{INSTRUMENT_PANIC}: {MUTANT}={raw:?} carries no mode; expected `<file>:<line>:<mode>` or `ALL:<mode>`")
    });
    let mode = match mode {
        "kind" => Mode::Kind,
        "message" => Mode::Message,
        other => panic!(
            "{INSTRUMENT_PANIC}: {MUTANT} names mode {other:?}; expected `kind` or `message`"
        ),
    };
    if head == "ALL" {
        return Selector { site: None, mode };
    }
    let (file, line) = head
        .rsplit_once(':')
        .unwrap_or_else(|| panic!("{INSTRUMENT_PANIC}: {MUTANT}={raw:?} carries no line number"));
    let line = line.parse().unwrap_or_else(|_| {
        panic!("{INSTRUMENT_PANIC}: {MUTANT}={raw:?} names line {line:?}, which is not a number")
    });
    Selector {
        site: Some((file.to_string(), line)),
        mode,
    }
}

/// The selector, parsed once. Reading the environment per construction would parse it thousands of times and
/// let a mid-run change split one run's verdict between two selectors.
fn selector() -> Option<&'static Selector> {
    static PARSED: OnceLock<Option<Selector>> = OnceLock::new();
    PARSED
        .get_or_init(|| std::env::var(MUTANT).ok().map(|raw| parse_selector(&raw)))
        .as_ref()
}

/// Append this construction's site to the record file, if one was named.
///
/// **A lost line is not self-announcing.** For a site that declares itself out of reach, a lost record makes
/// it look legally unreached and the whole run reports clean. So writes are serialised within the process,
/// each target run gets a file of its own so no two processes interleave, and a failed write panics rather
/// than degrading.
fn record(site: &Location<'_>) {
    static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    let handle = FILE.get_or_init(|| {
        let path = std::env::var_os(RECORD)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|err| {
                panic!("{INSTRUMENT_PANIC}: cannot open {RECORD}={path:?} for appending: {err}")
            });
        Some(Mutex::new(file))
    });
    let Some(handle) = handle else {
        return;
    };
    let mut file = handle
        .lock()
        .unwrap_or_else(|err| panic!("{INSTRUMENT_PANIC}: the record lock is poisoned: {err}"));
    writeln!(file, "{}:{}", site.file(), site.line())
        .unwrap_or_else(|err| panic!("{INSTRUMENT_PANIC}: cannot append to the record: {err}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two sites on two lines record two locations.
    ///
    /// This is the `#[track_caller]` propagation chain's own falsifier. Were the location read inside
    /// [`instrument`] rather than in each constructor, both of these would report this file's interior and be
    /// equal — the reaction would then enumerate every site and intercept one, reporting clean over the rest.
    #[test]
    fn two_sites_on_two_lines_are_two_locations() {
        #[track_caller]
        fn site() -> &'static Location<'static> {
            Location::caller()
        }
        let first = site();
        let second = site();
        assert_ne!(
            (first.file(), first.line()),
            (second.file(), second.line()),
            "two constructions on different lines reported one location, so the caller location is being \
             read outside the #[track_caller] chain and every site would look like one site"
        );
    }

    /// What `Location::caller().file()` actually says, measured rather than assumed.
    ///
    /// The sweep matches a site against the source list the compiler reported reading, and those are relative
    /// to the workspace root. If the two spellings disagree, every selector names nothing, every poisoned run
    /// stays green, and the sweep reports all sixty sites undistinguished — a wrong verdict that looks like a
    /// finding.
    #[test]
    fn a_site_is_spelled_the_way_the_compiler_spells_its_sources() {
        let here = Location::caller();
        assert_eq!(
            here.file(),
            "crates/tianheng/tests/support/refusal.rs",
            "the caller location spells this file differently from the compiler's own source list, so a \
             selector built from that list would name no site"
        );
    }

    #[test]
    fn a_selector_names_one_site_or_every_site() {
        let all = parse_selector("ALL:kind");
        assert!(all.site.is_none() && all.mode == Mode::Kind);

        let one = parse_selector("crates/tianheng/tests/support/refusal.rs:41:message");
        assert_eq!(
            one.site,
            Some(("crates/tianheng/tests/support/refusal.rs".to_string(), 41))
        );
        assert_eq!(one.mode, Mode::Message);
    }

    #[test]
    fn a_malformed_selector_is_a_panic_the_sweep_can_recognise() {
        for raw in ["ALL", "ALL:sideways", "some/file.rs:notanumber:kind"] {
            let err = std::panic::catch_unwind(|| parse_selector(raw))
                .expect_err("a malformed selector must not parse as `no perturbation`");
            let said = err
                .downcast_ref::<String>()
                .map(String::as_str)
                .unwrap_or_default();
            assert!(
                said.contains(INSTRUMENT_PANIC),
                "the panic for {raw:?} carries no instrument marker, so the sweep would read it as a site \
                 being distinguished rather than as its own instrument failing: {said}"
            );
        }
    }
}
