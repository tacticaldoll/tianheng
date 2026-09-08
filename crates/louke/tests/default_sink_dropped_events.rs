//! Reproduces the original finding end-to-end: no custom sink, a broken stderr, one fired
//! violation through the real `assert_boundary!` macro. This is its own process (a separate
//! integration-test binary), so replacing this process's real fd 2 cannot affect any other test.
#![cfg(unix)]

trait Port: louke::Tracked {}
struct Bad;
impl Port for Bad {}

// Raw FFI declarations rather than a `libc` dev-dependency: the workspace carries no
// dev-dependencies anywhere today, and CI runs exclusively on `ubuntu-latest`.
unsafe extern "C" {
    fn pipe(fds: *mut i32) -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
    fn close(fd: i32) -> i32;
}

/// Point fd 2 (stderr) at a pipe whose read end is already closed, so any write to it fails
/// with EPIPE. Unlike plain `close(2)`, fd 2 stays continuously occupied by this broken pipe —
/// never numerically free — so a concurrent `open` elsewhere in this multi-threaded test binary
/// cannot race in and claim fd 2 before the write under test happens.
fn break_stderr() {
    let mut fds = [0i32; 2];
    let rc = unsafe { pipe(fds.as_mut_ptr()) };
    assert_eq!(rc, 0, "failed to create the pipe for the test setup");
    let [read_end, write_end] = fds;
    assert_eq!(
        unsafe { close(read_end) },
        0,
        "failed to close the pipe's read end"
    );
    assert_eq!(
        unsafe { dup2(write_end, 2) },
        2,
        "failed to install the broken pipe as fd 2"
    );
    assert_eq!(
        unsafe { close(write_end) },
        0,
        "failed to close the spare fd"
    );
}

#[test]
fn a_broken_stderr_write_is_counted_instead_of_silently_lost() {
    // Deliberately do NOT call louke::set_sink — exercise the shipped default sink.
    louke::install(
        [louke::RuntimeBoundary::at("s")
            .only_origins(["app::domain"])
            .because("only domain may cross")],
        [],
    );

    let before = louke::dropped_sink_events();

    // The finding's trigger — `myapp 2>&1 | consumer` after its reader exits (EPIPE — Rust std
    // ignores SIGPIPE), a daemon with closed inherited fds, or plainly `myapp 2>&-` (EBADF).
    break_stderr();

    let bad: &dyn Port = &Bad;
    // Triggers the default sink's write with stderr broken. Must not panic (verified by the
    // test simply returning normally — a panic here would fail the test on its own).
    louke::assert_boundary!("s", bad);

    assert_eq!(
        louke::dropped_sink_events(),
        before + 1,
        "a violation dropped by a broken default-sink write must be counted exactly once"
    );
}
