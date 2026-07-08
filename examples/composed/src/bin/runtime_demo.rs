//! 漏刻's **prod face** — what static and semantic analysis structurally cannot see: the concrete
//! type behind a `dyn Adapter` crossing the seam, at runtime. Wire it into your binary once:
//! install the boundary + the observed origins, then probe at each seam.
use composed_app::adapters::{blessed, rogue};
use composed_app::port::Adapter;
use louke::{assert_boundary, install, set_sink, RuntimeBoundary};

fn main() {
    // The default reaction is an event, not a crash — a governance tool must never panic prod on
    // a false positive. Here the sink just prints; a real adopter ships it to logs/metrics.
    set_sink(|v| {
        eprintln!(
            "⛒ runtime reaction — {} · {}\n    {}",
            v.target, v.finding, v.reason
        )
    });

    install(
        [RuntimeBoundary::at("adapter-seam")
            .only_origins(["composed_app::adapters::blessed"])
            .because("only the blessed adapter may cross the port seam")],
        [blessed::origin(), rogue::origin()],
    );

    let allowed: Box<dyn Adapter> = Box::new(blessed::BlessedAdapter);
    assert_boundary!("adapter-seam", &*allowed);
    println!("blessed adapter crossed the seam cleanly");

    let rogue_obj: Box<dyn Adapter> = Box::new(rogue::RogueAdapter);
    assert_boundary!("adapter-seam", &*rogue_obj);
    println!("rogue adapter crossed — the fail-closed reaction fired above");
}
