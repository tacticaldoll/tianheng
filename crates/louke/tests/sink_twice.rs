//! The sink is set once: a second `set_sink` must fail loud (its own test binary).
use louke::Violation;

#[test]
fn a_second_set_sink_is_an_error() {
    louke::set_sink(|_: &Violation| {});
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let again = std::panic::catch_unwind(|| louke::set_sink(|_: &Violation| {}));
    std::panic::set_hook(prev);
    assert!(
        again.is_err(),
        "a second set_sink must fail loud (the sink is set once)"
    );
}
