//! The debug trace: off unless the program was started with `--debug` (or
//! `IDLEWALKER_DEBUG=1`). Every layer can write to it with `debug_note!`; where the lines
//! go is the composition root's business, so this crate still names no file.
use std::sync::OnceLock;

type Sink = Box<dyn Fn(&str) + Send + Sync>;

static SINK: OnceLock<Sink> = OnceLock::new();

/// Turns the trace on, writing each line to `sink`. Only the first call counts.
pub fn install(sink: Sink) {
    let _ = SINK.set(sink);
}

/// Whether the trace is on: checked before a line is even formatted.
pub fn enabled() -> bool {
    SINK.get().is_some()
}

/// Writes one line, if the trace is on.
pub fn note(line: &str) {
    if let Some(sink) = SINK.get() {
        sink(line);
    }
}

/// `debug_note!("tray: {}", what)`: formats and writes a line only while the trace is on.
#[macro_export]
macro_rules! debug_note {
    ($($arg:tt)*) => {
        if $crate::debug::enabled() {
            $crate::debug::note(&format!($($arg)*));
        }
    };
}
