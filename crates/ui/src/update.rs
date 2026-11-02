//! When update checks run, and how their answers reach the UI thread. What a check
//! does is the `Updates` use case's business.
use super::UserEvent;
use idlewalker_application::Updates;
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

/// Long enough after start-up not to slow it, or to race a sign-in's network connection.
const FIRST_CHECK: Duration = Duration::from_secs(60);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

/// Checks soon after start-up, then every few hours, for as long as the program runs.
pub fn spawn_periodic(proxy: EventLoopProxy<UserEvent>, updates: Updates) {
    let _ = std::thread::Builder::new()
        .name("IdleWalker update check".into())
        .spawn(move || {
            std::thread::sleep(FIRST_CHECK);
            loop {
                let state = updates.check();
                let event = UserEvent::UpdateChecked {
                    state,
                    asked: false,
                };
                if proxy.send_event(event).is_err() {
                    return;
                }
                std::thread::sleep(CHECK_EVERY);
            }
        });
}

/// A check the person asked for; the answer goes to the open dialog.
pub fn spawn_check(proxy: EventLoopProxy<UserEvent>, updates: Updates) {
    std::thread::spawn(move || {
        let state = updates.check();
        let _ = proxy.send_event(UserEvent::UpdateChecked { state, asked: true });
    });
}
