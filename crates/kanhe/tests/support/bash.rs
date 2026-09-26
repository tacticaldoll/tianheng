//! The one `bash` the repository checks run, and what it inherits: nothing but what is named here.
//!
//! **The question is what a run needs, not which variables could move it.** bash reads its environment at
//! startup — `BASH_ENV` and `ENV` name a file it sources first, `SHELLOPTS` and `BASHOPTS` set options, `CDPATH`
//! and `GLOBIGNORE` change what a command does — and a list of those is bash's grammar again, answered one member
//! per finding. So the environment is cleared and [`INHERITED`] hands back the few values a run needs from the
//! host; a caller sets anything else it means, by name. [`bash_with_no_environment`] is the control, and the one
//! other `bash` this file constructs. Held by `the_bash_builder_hands_on_only_what_it_names`,
//! against what bash exports of its own accord.

use std::process::Command;

/// The host values every run keeps: where programs are found, where a program keeps its state, and where it
/// writes a temporary file.
pub const INHERITED: [&str; 3] = ["PATH", "HOME", "TMPDIR"];

/// `bash`, with the environment cleared and [`INHERITED`] handed back from the host.
pub fn bash() -> Command {
    let mut command = Command::new("bash");
    command.env_clear();
    for name in INHERITED {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

/// The same `bash` [`bash`] runs, with nothing in its environment at all — the control that says what bash
/// exports of its own accord, so what the builder hands on can be measured against it.
///
/// Named by the path that bash reports as its own, kept as the bytes it is, so the control is that binary rather
/// than whichever one a default search path would find.
pub fn bash_with_no_environment() -> Command {
    let reported = bash()
        .args(["-c", r#"printf '%s' "$BASH""#])
        .output()
        .expect("bash runs and names its own path");
    let path = <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(reported.stdout);
    let mut command = Command::new(path);
    command.env_clear();
    command
}
