//! The launchers that hand a wrapper a closed stream or one nobody reads, for the directions holding that a stream
//! moves no class. Each takes the descriptor as `$1`, so one launcher serves both streams.

/// Runs `bash "$@"` with descriptor `$1` — `1` or `2` — closed.
pub const CLOSED: &str = r#"fd=$1; shift
case $fd in 1) exec >&- ;; 2) exec 2>&- ;; *) exit 97 ;; esac
exec bash "$@""#;

/// Runs `bash "$@"` with descriptor `$1` — `1` or `2` — a FIFO whose one reader has closed before the command
/// starts, so every write to it finds the pipe broken.
///
/// **A reader that exits on its own races the writer.** `| true` exits when it is scheduled, and a write that lands
/// in the pipe's buffer first succeeds, so a direction built on it passes for a reason unrelated to the stream —
/// most of all for a stop that writes in its first milliseconds, which is the stop a broken stderr has to be
/// measured on. Opening the FIFO read-write first lets the write end open without blocking, and closing that
/// descriptor leaves no reader before the command runs. The launcher writes nothing once the stream is broken, so
/// a signal it met would not be the command's.
pub const BROKEN: &str = r#"fd=$1; shift
d=$(mktemp -d) && mkfifo "$d/p" && exec 3<>"$d/p" || exit 97
case $fd in 1) exec >"$d/p" ;; 2) exec 2>"$d/p" ;; *) exit 97 ;; esac
exec 3>&-
rm -rf "$d"
exec bash "$@""#;
