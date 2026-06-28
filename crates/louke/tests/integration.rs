//! End-to-end exercise of the prod-face macros and global write-once state. Process-global
//! install/sink are write-once, so the whole flow lives in ONE test function.
use std::sync::{Mutex, OnceLock};

use louke::{RuntimeBoundary, Violation};

// A governed trait carries the louke::Tracked supertrait so a probe can recover the
// concrete type behind &dyn Port (rust 1.85, no upcasting).
trait Port: louke::Tracked {}

mod allowed {
    pub struct Good;
    impl super::Port for Good {}
    pub fn origin() -> &'static str {
        module_path!()
    }
    pub fn entry() -> louke::OriginEntry {
        louke::register_origin!(Good)
    }
}

mod forbidden {
    pub struct Bad;
    impl super::Port for Bad {}
    pub fn entry() -> louke::OriginEntry {
        louke::register_origin!(Bad)
    }
}

fn events() -> &'static Mutex<Vec<String>> {
    static EVENTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    EVENTS.get_or_init(|| Mutex::new(Vec::new()))
}

#[test]
fn prod_face_end_to_end() {
    louke::set_sink(|v: &Violation| {
        events()
            .lock()
            .unwrap()
            .push(format!("{}|{}", v.target, v.finding));
    });
    louke::install(
        [
            RuntimeBoundary::at("evt")
                .only_origins([allowed::origin()])
                .because("only good may cross evt"),
            RuntimeBoundary::at("pan")
                .only_origins([allowed::origin()])
                .panic_on_violation()
                .because("only good may cross pan"),
        ],
        [allowed::entry(), forbidden::entry()],
    );

    let good: &dyn Port = &allowed::Good;
    let bad: &dyn Port = &forbidden::Bad;

    // allowed origin crossing evt: no event
    louke::assert_boundary!("evt", good);
    assert!(
        events().lock().unwrap().is_empty(),
        "allowed origin must not react"
    );

    // disallowed origin crossing evt: an event, no panic
    louke::assert_boundary!("evt", bad);
    {
        let e = events().lock().unwrap();
        assert_eq!(e.len(), 1, "disallowed origin must emit one event");
        assert!(e[0].starts_with("evt|"), "{:?}", e[0]);
        assert!(
            e[0].contains("forbidden"),
            "finding names the origin: {:?}",
            e[0]
        );
    }

    // disallowed origin crossing the panic seam: panics (opt-in), caught here
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // silence the panic message
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        louke::assert_boundary!("pan", bad);
    }))
    .is_err();
    std::panic::set_hook(prev);
    assert!(panicked, "a panic-opt-in boundary must panic on violation");
}
