//! The write-once constitution: a second `install` must fail loud (its own test binary, so
//! the process-global registry is fresh).
use louke::RuntimeBoundary;

#[test]
fn a_second_install_is_a_constitution_error() {
    louke::install(
        [RuntimeBoundary::at("s").only_origins(["o"]).because("r")],
        [],
    );
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let again = std::panic::catch_unwind(|| {
        louke::install(
            [RuntimeBoundary::at("s2").only_origins(["o"]).because("r")],
            [],
        );
    });
    std::panic::set_hook(prev);
    assert!(
        again.is_err(),
        "a second install must fail loud (the constitution is write-once)"
    );
}
