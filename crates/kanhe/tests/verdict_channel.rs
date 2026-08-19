//! The failure matrix for [`crate::verdict_channel`]: what each verdict reaches, and what it fails.

use kanhe::refusal::{Kind, Refusal};
use kanhe::verdict_channel::{CLEAN, Verdict, reached, refuses, rendered};

/// A real violation, from a judgement that produces one.
///
/// **Built by a judgement rather than by a constructor**, because a constructor call here would be a refusal
/// site inside `crates/kanhe/src` carrying no identity — which `no_refusal_site_is_untriaged` refuses, and
/// rightly: the register cannot tell a fixture's construction from a branch nobody observes. Taking the
/// value from a module that already registers its sites costs nothing and exercises what the crate produces.
fn a_violation() -> Refusal {
    kanhe::merge_message_gate::judge(
        "not a conventional subject",
        "body",
        "not a conventional subject",
        &["feat(x): one".to_string()],
    )
    .expect_err("a subject that is not a Conventional Commit is a violation")
}

/// A real cannot-judge, likewise.
fn a_cannot_judge() -> Refusal {
    kanhe::selection::the_only("probe", Vec::<u8>::new())
        .expect_err("no candidate is a cannot-judge")
}

/// Every verdict a harness can reach, so the directions below enumerate rather than sample.
fn every_verdict() -> Vec<Verdict> {
    vec![
        Verdict::NotAsked("the act is not being made".to_string()),
        Verdict::Clean("ok".to_string()),
        Verdict::Refused(a_violation()),
        Verdict::Refused(a_cannot_judge()),
    ]
}

/// **Every verdict that fails the run carries a class to report.**
///
/// This is the property the direction it replaces could not hold. That one located the harness's
/// `Err(refusal) => {` arm by substring and asserted the report preceded the panic *inside it*, so every
/// other exit owed nothing — and a subject supplied as unreadable bytes left through one of them, clean.
/// Asked of the type instead, the enumeration is the whole set by construction.
///
/// **What this does not observe, and what holds it instead.** It asks two pure functions to agree —
/// `refuses` implies `reached` is `Some` — which is what makes it total over the enumeration. The
/// *ordering*, that `deliver` writes the channel before it panics, is held by construction: the write sits
/// above the `match` in a function with one exit per arm, and nothing here runs it. Its earlier title said
/// the verdict *reached the channel first*, which is the ordering, and named a fact no assertion in this
/// body reaches.
#[test]
fn every_refusing_verdict_reaches_the_channel() {
    for verdict in every_verdict() {
        assert!(
            !refuses(&verdict) || reached(&verdict).is_some(),
            "a verdict that fails the run must first say so on the channel, or a wrapper reads the failure \
             as a run that never judged: {verdict:?}"
        );
    }
}

/// And the converse a wrapper's success path depends on: a verdict that does **not** fail still says whether
/// one was reached.
///
/// *Not asked* is the only state that writes nothing, which is what makes an absent file mean **unjudged**
/// rather than merely *not refused*. Without the clean arm writing, a wrapper's success path had no evidence
/// at all and `require_one_pass` stood in for it — answering *a test passed*, a different question, satisfied
/// by a harness that returned without judging.
#[test]
fn only_an_unasked_verdict_writes_nothing() {
    assert_eq!(
        reached(&Verdict::NotAsked("nothing to judge".to_string())),
        None
    );
    assert_eq!(
        reached(&Verdict::Clean("ok".to_string())),
        Some(CLEAN.to_string())
    );
    assert_eq!(
        reached(&Verdict::Refused(a_violation())),
        Some(rendered(Kind::Violation))
    );
    assert_eq!(
        reached(&Verdict::Refused(a_cannot_judge())),
        Some(rendered(Kind::CannotJudge))
    );
}

/// Exactly one of the four fails, and it is the one that carries a refusal.
#[test]
fn a_verdict_fails_the_run_only_where_it_refuses() {
    let failing: Vec<bool> = every_verdict().iter().map(refuses).collect();
    assert_eq!(
        failing,
        vec![false, false, true, true],
        "a run fails where a judgement refused and nowhere else — `not asked` and `clean` both let the \
         wrapper proceed, and they are told apart by the channel rather than by the exit status"
    );
}

/// The clean rendering is not one of the refusal classes, so a wrapper cannot read agreement as disagreement.
#[test]
fn the_clean_rendering_is_no_refusal_class() {
    assert_ne!(CLEAN, rendered(Kind::Violation));
    assert_ne!(CLEAN, rendered(Kind::CannotJudge));
}
