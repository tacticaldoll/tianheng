//! Cross-dimension conformance for how a module path is spelled — 圭表 (`guibiao`) and 渾儀 (`hunyi`)
//! each accept a module path only as `crate` or `crate::` followed by `::`-separated identifiers,
//! fold `r#x` into `x`, and refuse a path naming no module. 三儀 ⊥ 三儀 means neither can call the
//! other's reading, so each keeps its own and the two agree by using the same rule, not the same
//! function — this is what holds them to it.
//!
//! One table, defined here once, is fed to every public builder of either dimension that takes a
//! module path: 渾儀's module anchor, and 圭表's governed module and the four modules its import
//! rules name. Each row must be accepted or refused alike, and an accepted row must be recorded in
//! the same form — the violation target for a governed module or anchor, the rule key for a named
//! module. Where both refuse a row for its spelling, the two refusals are the same text. The rows past
//! the 渾儀 spelling table probe characters past ASCII: 渾儀 asks syn's lexer and 圭表 asks Unicode's
//! `XID_Start` and `XID_Continue`, the rule syn's lexer applies, so a no-break space, a zero-width space
//! and an emoji are refused by both for their spelling, with one message, and `é` is an identifier to both.

use std::path::Path;

use guibiao::{Constitution as GnomonConstitution, ModuleBoundary, Outcome as GnomonOutcome};
use hunyi::{AsyncExposureBoundary, Outcome as HunyiOutcome, check_async_exposure};

#[path = "support/mod.rs"]
mod support;
use support::TempFixture;

/// Every spelling judged, with how each dimension must answer it.
const SPELLINGS: &[(&str, Expect)] = &[
    ("crate", Expect::Canonical("crate")),
    ("crate::kernel", Expect::Canonical("crate::kernel")),
    ("crate::r#kernel", Expect::Canonical("crate::kernel")),
    (
        "crate::kernel::r#inner",
        Expect::Canonical("crate::kernel::inner"),
    ),
    ("crate::kernel::", Expect::Misspelled),
    ("", Expect::Misspelled),
    ("kernel", Expect::Misspelled),
    ("::crate::kernel", Expect::Misspelled),
    ("crate::::kernel", Expect::Misspelled),
    ("self::kernel", Expect::Misspelled),
    ("super::kernel", Expect::Misspelled),
    ("crate:: kernel", Expect::Misspelled),
    ("crate::kernel ", Expect::Misspelled),
    ("crate ::kernel", Expect::Misspelled),
    ("r#crate::kernel", Expect::Misspelled),
    ("crate::kernel::*", Expect::Misspelled),
    ("crate::1kernel", Expect::Misspelled),
    ("crate::r#self", Expect::Misspelled),
    ("crate::r#", Expect::Misspelled),
    ("crate::r#r#kernel", Expect::Misspelled),
    ("crate::nope", Expect::Absent),
    ("crate::type", Expect::Absent),
    ("crate::_", Expect::Absent),
    ("crate::k\u{e9}rnel", Expect::Absent),
    ("crate::\u{a0}kernel", Expect::Misspelled),
    ("crate::kernel\u{200b}", Expect::Misspelled),
    ("crate::\u{1f600}", Expect::Misspelled),
];

/// How a row must be answered.
#[derive(Clone, Copy, Debug)]
enum Expect {
    /// Accepted by both, and recorded as this form.
    Canonical(&'static str),
    /// Refused by both for its spelling, with one message.
    Misspelled,
    /// A canonical spelling of no module: refused by both as absent.
    Absent,
}

impl Expect {
    fn hunyi(self) -> Kind {
        match self {
            Expect::Canonical(form) => Kind::Accepted(form.to_string()),
            Expect::Misspelled => Kind::Misspelled,
            Expect::Absent => Kind::Absent,
        }
    }

    fn guibiao(self) -> Kind {
        self.hunyi()
    }
}

/// An answer with its message set aside.
#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Accepted(String),
    Misspelled,
    Absent,
    Other,
}

const PACKAGE: &str = "module-path-spelling-parity";

/// A crate in which every accepted spelling reacts in every role: `crate::kernel` and its child
/// `crate::kernel::inner` each expose an async fn and import `crate::other`, which imports both.
fn fixture() -> TempFixture {
    let fixture = TempFixture::new(PACKAGE, "pub mod kernel;\npub mod other;\n");
    fixture.write(
        "src/kernel.rs",
        "pub mod inner;\npub struct K;\npub async fn a() {}\nuse crate::other::O;\n",
    );
    fixture.write(
        "src/kernel/inner.rs",
        "pub struct I;\npub async fn b() {}\nuse crate::other::O;\n",
    );
    fixture.write(
        "src/other.rs",
        "pub struct O;\nuse crate::kernel::K;\nuse crate::kernel::inner::I;\n",
    );
    fixture
}

/// One dimension's answer for one spelling in one role.
#[derive(Debug, PartialEq, Eq)]
enum Answer {
    /// Judged, recorded as this form.
    Accepted(String),
    /// Refused with this message.
    Refused(String),
}

impl Answer {
    fn kind(&self) -> Kind {
        match self {
            Answer::Accepted(form) => Kind::Accepted(form.clone()),
            Answer::Refused(message)
                if message.starts_with("a module is named by one spelling") =>
            {
                Kind::Misspelled
            }
            Answer::Refused(message) if message.contains("is not found among") => Kind::Absent,
            Answer::Refused(_) => Kind::Other,
        }
    }
}

/// The only violation target an outcome carries, or the refusal. An outcome judged clean, or
/// carrying more than one target, answers neither and fails the case naming it.
fn hunyi_answer(outcome: HunyiOutcome) -> Answer {
    match outcome {
        HunyiOutcome::ConstitutionError(message) => Answer::Refused(message),
        HunyiOutcome::Violations(report) => Answer::Accepted(the_only_target(
            report.violations.iter().map(|v| v.target()),
        )),
        other => panic!("the fixture reacts to every accepted anchor: {other:?}"),
    }
}

fn gnomon_target(outcome: GnomonOutcome) -> Answer {
    match outcome {
        GnomonOutcome::ConstitutionError(message) => Answer::Refused(message),
        GnomonOutcome::Violations(report) => Answer::Accepted(the_only_target(
            report.violations.iter().map(|v| v.target()),
        )),
        other => panic!("the fixture reacts to every accepted governed module: {other:?}"),
    }
}

fn the_only_target<'a>(targets: impl Iterator<Item = &'a str>) -> String {
    let mut targets: Vec<&str> = targets.collect();
    targets.sort_unstable();
    targets.dedup();
    assert_eq!(targets.len(), 1, "one module is one target: {targets:?}");
    targets[0].to_string()
}

/// A named module's answer: the refusal, or the form its rule key records it under. An allowlist of
/// one records a one-member set.
fn gnomon_named(boundary: ModuleBoundary, field: &str, manifest: &Path) -> Answer {
    let recorded = boundary
        .rule_key()
        .fields()
        .find(|(name, _)| *name == field)
        .map(|(_, value)| value.to_string())
        .expect("the rule key names the module");
    match check_gnomon(boundary, manifest) {
        GnomonOutcome::ConstitutionError(message) => Answer::Refused(message),
        _ => Answer::Accepted(match serde_json::from_str::<Vec<String>>(&recorded) {
            Ok(set) => {
                assert_eq!(set.len(), 1, "an allowlist of one: {set:?}");
                set[0].clone()
            }
            Err(_) => recorded,
        }),
    }
}

fn check_gnomon(boundary: ModuleBoundary, manifest: &Path) -> GnomonOutcome {
    guibiao::check(
        &GnomonConstitution::new(PACKAGE).boundary(boundary),
        manifest,
    )
}

type Role = fn(&str, &Path) -> Answer;

fn named(module: &str) -> guibiao::ModuleTargetDraft {
    ModuleBoundary::in_crate(PACKAGE).module(module)
}

fn other() -> guibiao::ModuleTargetDraft {
    named("crate::other")
}

/// Every 圭表 builder that takes a module path, each driven through `guibiao::check`.
fn gnomon_roles() -> Vec<(&'static str, Role)> {
    vec![
        ("module", |p, m| {
            gnomon_target(check_gnomon(
                named(p).must_not_import("crate::other").because("r"),
                m,
            ))
        }),
        ("must_not_import", |p, m| {
            gnomon_named(other().must_not_import(p).because("r"), "module", m)
        }),
        ("must_not_be_imported_by", |p, m| {
            gnomon_named(
                other().must_not_be_imported_by(p).because("r"),
                "importer",
                m,
            )
        }),
        ("restrict_imports_to", |p, m| {
            gnomon_named(other().restrict_imports_to([p]).because("r"), "allowed", m)
        }),
        ("must_only_be_imported_by", |p, m| {
            gnomon_named(
                other().must_only_be_imported_by([p]).because("r"),
                "allowed",
                m,
            )
        }),
    ]
}

/// Every row is answered as the table says, by 渾儀 and by each 圭表 role, and a row both refuse for
/// its spelling is refused with one message. Every row and role is judged before the assertion, so a
/// failure lists all of them.
#[test]
fn guibiao_and_hunyi_accept_and_record_a_module_path_alike() {
    let fixture = fixture();
    let manifest = fixture.manifest();
    let mut wrong = Vec::new();
    for (written, expected) in SPELLINGS {
        let hunyi = hunyi_answer(check_async_exposure(
            &[AsyncExposureBoundary::in_crate(PACKAGE)
                .module(written)
                .must_not_expose_async_fn()
                .including_submodules()
                .because("r")],
            manifest,
        ));
        if hunyi.kind() != expected.hunyi() {
            wrong.push(format!(
                "hunyi {written:?}: {hunyi:?}, expected {expected:?}"
            ));
        }
        for (role, answer) in gnomon_roles() {
            let gnomon = answer(written, manifest);
            if gnomon.kind() != expected.guibiao() {
                wrong.push(format!(
                    "guibiao {role} {written:?}: {gnomon:?}, expected {expected:?}"
                ));
            } else if gnomon.kind() == Kind::Misspelled && gnomon != hunyi {
                wrong.push(format!(
                    "guibiao {role} {written:?} is refused unlike hunyi:\n    {gnomon:?}\n    {hunyi:?}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
