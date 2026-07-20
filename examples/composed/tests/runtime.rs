//! 漏刻 run-mode, asserted as runnable proof. `install`/`set_sink` write process-global state
//! once, so this lives in its own test binary and installs a single time; it then crosses the
//! seam with an allowed origin (no reaction) and a rogue origin (fail-closed reaction).
use std::sync::atomic::{AtomicUsize, Ordering};

use composed_app::adapters::{blessed, rogue};
use composed_app::port::Adapter;
use louke::{assert_boundary, install, set_sink, RuntimeBoundary};

static REACTIONS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn a_rogue_origin_reacts_at_runtime_and_a_blessed_one_does_not() {
    set_sink(|violation| {
        assert_eq!(violation.kind, louke::BoundaryKind::Runtime);
        assert_eq!(violation.target, "adapter-seam");
        assert_eq!(violation.finding_key().namespace(), "louke");
        assert_eq!(violation.finding_key().code(), "registered_crossing");
        REACTIONS.fetch_add(1, Ordering::SeqCst);
    });
    install(
        [RuntimeBoundary::at("adapter-seam")
            .only_origins(["composed_app::adapters::blessed"])
            .because("only the blessed adapter may cross the port seam")],
        [blessed::origin(), rogue::origin()],
    );

    let allowed: Box<dyn Adapter> = Box::new(blessed::BlessedAdapter);
    assert_boundary!("adapter-seam", &*allowed);
    assert_eq!(
        REACTIONS.load(Ordering::SeqCst),
        0,
        "the blessed origin is on the allowlist — no reaction"
    );

    let rogue_obj: Box<dyn Adapter> = Box::new(rogue::RogueAdapter);
    assert_boundary!("adapter-seam", &*rogue_obj);
    assert_eq!(
        REACTIONS.load(Ordering::SeqCst),
        1,
        "the rogue origin is not on the allowlist — fail-closed reaction"
    );
}
