//! The composition root: the one place that knows every layer. It picks the adapters
//! for this machine, plugs them into the ports, and hands the result to the UI.
// A windowed app on Windows: no console opens behind it.
#![cfg_attr(windows, windows_subsystem = "windows")]

use idlewalker_application::{Brains, Preferences, Session, SessionPorts, Updates};
use idlewalker_domain::rt;
use idlewalker_platform as platform;
use std::sync::Arc;

fn main() {
    if std::env::args().any(|a| a == "--version") {
        println!("idlewalker {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    // IDLEWALKER_TEST: keep the panic hook, leave the sign-in entry alone, no update checks.
    let test = std::env::var_os("IDLEWALKER_TEST").is_some();

    // A caught panic stands in for a caught C# exception, so a release build prints
    // nothing; with IDLEWALKER_TEST the default hook stays, to show what went wrong.
    if !cfg!(debug_assertions) && !test {
        std::panic::set_hook(Box::new(|_| {}));
    }
    if !platform::single_instance() {
        return;
    }
    // Before any window exists, so monitor coordinates are real pixels on every display.
    platform::init_process();
    rt::install_local_clock(Box::new(platform::SystemClock));

    let desktop = Arc::new(platform::NativeDesktop);
    let journal = Arc::new(platform::TempFileJournal);
    let preferences = Preferences::new(Arc::new(platform::FilePreferences::new()));
    let brains = Brains::new(Arc::new(platform::FileBrainStore::new()));
    // IDLEWALKER_TEST_VERSION pretends to be another version, to try the update dialog.
    let version = std::env::var("IDLEWALKER_TEST_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").into());
    let updates = Updates::new(
        Arc::new(platform::CurlReleaseFeed),
        preferences.clone(),
        version,
    );

    let session_journal = journal.clone();
    idlewalker_ui::run(idlewalker_ui::Wiring {
        session: Box::new(move || {
            let ports = SessionPorts {
                desktop,
                displays: Box::new(platform::MonitorWorld::new()),
                journal: session_journal,
                preferences,
                brains,
            };
            Session::new(ports, !test)
        }),
        sprites: platform::sprite_factory(),
        updates,
        journal,
        // Pixels are never urgent: the painters yield to whatever the user is doing.
        on_painter_start: platform::lower_thread_priority,
        check_for_updates: !test,
        // Lets a test drive the menu: IDLEWALKER_TEST_MENU=about,brains
        test_menu: std::env::var("IDLEWALKER_TEST_MENU")
            .map(|ids| ids.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
    });
}
