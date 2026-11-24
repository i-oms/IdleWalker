//! The Telegram remote's presentation: the tray menu shown as buttons in a chat. The
//! buttons are made from the very description the tray menu is drawn from, so whatever
//! the menu offers the chat offers too. Only a few slash commands remain: /menu, /info,
//! /shutdown and /restart; everything else is a button.
//!
//! A worker waits on Telegram. What it hears goes to the UI thread, where the choices
//! live, and the answer comes back to be sent; what is slow (brightness, the update
//! check) the worker does itself.
use super::UserEvent;
use crate::prelude::*;
use crate::tray::{self, Node};
use idlewalker_application::macros::{Played, Renamed};
use idlewalker_application::telegram::{
    Button, Call, Keyboard, Link, Polled, Update, What, LONG_POLL_SECONDS,
};
use idlewalker_application::{
    BrightnessInfo, Desktop, Indicator, Journal, Macros, Power, Session, Telegram, UpdateState,
    Updates,
};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

/// What a pressed button sends back. Telegram allows 64 bytes; the longest here is a
/// colour in a group in a group, well under that.
mod data {
    /// A button for something that is switched off.
    pub const OFF: &str = "n";
    /// "p:<path>" shows a group; the path is child indexes, "6.2", empty for the top.
    pub const PAGE: &str = "p:";
    /// "a:<path>:<0|1>:<menu id>" makes a tick box ticked (1) or not, then shows the group.
    pub const APPLY: &str = "a:";
    /// "h:<menu id>" explains a choice that needs typing.
    pub const HINT: &str = "h:";
    /// Alone, the brightness page; "b:<percent>" sets every display.
    pub const BRIGHTNESS: &str = "b";
    /// A picture of every screen.
    pub const SCREENSHOT: &str = "c";
    /// Alone, the mouse macros' page; "m:<name>" plays one, and "m!" stops it. A name
    /// is at most 32 bytes and has neither ":" nor "!".
    pub const MACROS: &str = "m";
    pub const MACRO_STOP: &str = "m!";
    /// The page for renaming and deleting macros; "mr:<name>" asks for a new name,
    /// "md:<name>" asks whether to delete it, and "md!:<name>" deletes it.
    pub const MACRO_MANAGE: &str = "mm";
    pub const MACRO_RENAME: &str = "mr:";
    pub const MACRO_DELETE: &str = "md:";
    pub const MACRO_DELETE_SURE: &str = "md!:";
    pub const UPDATE: &str = "u";
    pub const ABOUT: &str = "i";
    pub const EXIT: &str = "x";
    pub const EXIT_SURE: &str = "x!";
    /// Asks first; with "!" it is done.
    pub const SHUT_DOWN: &str = "s";
    pub const SHUT_DOWN_SURE: &str = "s!";
    pub const RESTART: &str = "r";
    pub const RESTART_SURE: &str = "r!";
}

const NOT_NOW: &str = "Not available while its feature is off.";

const SLEEPY: &str = "Nothing holds off sleep now: once the computer sleeps this remote \
                      cannot reach it. Prevent sleep keeps it within reach.";

/// What a plain message (not a command) is taken as, after a button asked for it.
#[derive(Clone, Debug, PartialEq)]
enum Awaiting {
    /// A colour, for the "-custom" choice with this menu id.
    Colour(String),
    /// A new name for this macro.
    Rename(String),
}

/// Shared by the worker (renames) and the UI thread (colours). Any command or button
/// press clears it.
static AWAITING: Mutex<Option<Awaiting>> = Mutex::new(None);

fn set_awaiting(a: Option<Awaiting>) {
    *AWAITING.lock().unwrap_or_else(|e| e.into_inner()) = a;
}

fn awaiting() -> Option<Awaiting> {
    AWAITING.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

// ---- The menu as a keyboard --------------------------------------------------------------

/// The menu without the remote's own group: it cannot switch itself off from afar.
fn menu(session: &Session) -> Vec<Node> {
    tray::build(session.choices(), &**session.desktop(), None, None)
}

/// The group at a path: its title, what is in it, and whether it can be used.
fn group<'a>(nodes: &'a [Node], path: &str) -> Option<(String, &'a [Node], bool)> {
    let (mut title, mut at, mut on) = (String::new(), nodes, true);
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match at.get(part.parse::<usize>().ok()?)? {
            Node::Sub {
                label,
                enabled,
                children,
            } => {
                title = label.clone();
                at = children;
                on &= *enabled;
            }
            _ => return None,
        }
    }
    Some((title, at, on))
}

fn child_path(path: &str, i: usize) -> String {
    if path.is_empty() {
        i.to_string()
    } else {
        format!("{path}.{i}")
    }
}

/// A tick box anywhere in the menu: whether it is ticked, whether it can be used, and
/// the path of the group it is in.
fn find(nodes: &[Node], id: &str, on: bool, path: &str) -> Option<(bool, bool, String)> {
    nodes.iter().enumerate().find_map(|(i, n)| match n {
        Node::Check {
            id: this,
            enabled,
            checked,
            ..
        } if this == id => Some((*checked, on && *enabled, path.to_string())),
        Node::Sub {
            enabled, children, ..
        } => find(children, id, on && *enabled, &child_path(path, i)),
        _ => None,
    })
}

/// Two short buttons share a row; a long one has a row to itself.
fn rows(buttons: Vec<Button>) -> Keyboard {
    let short = |b: &Button| b.label.chars().count() <= 18;
    let mut rows: Keyboard = Vec::new();
    for b in buttons {
        match rows.last_mut() {
            Some(row) if row.len() == 1 && short(&row[0]) && short(&b) => row.push(b),
            _ => rows.push(vec![b]),
        }
    }
    rows
}

fn keyboard(at: &[Node], path: &str, on: bool) -> Keyboard {
    let mut buttons = Vec::new();
    for (i, n) in at.iter().enumerate() {
        let (label, usable, what) = match n {
            Node::Sep => continue,
            Node::Item { id, label, enabled } => {
                let what = match id.as_str() {
                    "brightness" => data::BRIGHTNESS,
                    "check-update" => data::UPDATE,
                    "about" => data::ABOUT,
                    "exit" => data::EXIT,
                    // The rest are windows on the computer: nothing to show in a chat.
                    _ => continue,
                };
                (
                    label.trim_end_matches("...").to_string(),
                    *enabled,
                    what.to_string(),
                )
            }
            Node::Check {
                id,
                label,
                enabled,
                checked,
            } => {
                let mark = if *checked { "\u{2705}" } else { "\u{2B1C}" };
                let what = if id.ends_with("-custom") {
                    format!("{}{id}", data::HINT)
                } else {
                    format!("{}{path}:{}:{id}", data::APPLY, !*checked as i32)
                };
                (format!("{mark} {label}"), *enabled, what)
            }
            Node::Sub { label, enabled, .. } => (
                format!("{label} \u{203A}"),
                *enabled,
                format!("{}{}", data::PAGE, child_path(path, i)),
            ),
        };
        buttons.push(if on && usable {
            Button::new(label, what)
        } else {
            Button::new(format!("\u{1F512} {label}"), data::OFF)
        });
    }
    let mut keys = rows(buttons);
    let refresh = Button::new("\u{1F504} Refresh", format!("{}{path}", data::PAGE));
    if path.is_empty() {
        // The computer itself, which the menu on it has no need to offer.
        keys.push(vec![
            Button::new("\u{1F4F8} Screenshot", data::SCREENSHOT),
            Button::new("\u{1F5B1} Macros", data::MACROS),
        ]);
        keys.push(vec![
            Button::new("\u{23FB} Shut down", data::SHUT_DOWN),
            Button::new("\u{1F501} Restart", data::RESTART),
        ]);
    }
    keys.push(match path.rsplit_once('.') {
        _ if path.is_empty() => vec![refresh],
        parent => {
            let up = parent.map_or("", |(up, _)| up);
            let back = Button::new("\u{2039} Back", format!("{}{up}", data::PAGE));
            vec![back, refresh]
        }
    });
    keys
}

fn on_off(on: bool) -> &'static str {
    if on {
        "on"
    } else {
        "off"
    }
}

/// How things stand, in a few lines.
fn status(session: &Session) -> String {
    let c = session.choices();
    let out = |mask: i32| mask.count_ones();
    [
        "IdleWalker".to_string(),
        String::new(),
        format!("Walkers: {}", on_off(c.walkers)),
        format!("Appear after: {}", tray::describe_idle(c.idle_seconds)),
        format!("Hide on mouse move: {}", on_off(c.hide_on_move)),
        format!(
            "Minions: {} ({} of {}), dress: {}",
            on_off(c.minions),
            out(c.minion_mask),
            Settings::MAX_WALKERS,
            tray::describe_dress(c.dress)
        ),
        format!(
            "Angry Birds: {} ({} of {})",
            on_off(c.birds),
            out(c.bird_mask),
            Settings::MAX_BIRDS
        ),
        format!("Hot corners: {}", on_off(c.corners)),
        format!("Highlight mouse clicks: {}", on_off(c.click_highlight)),
        format!("Find cursor: {}", on_off(c.find_cursor)),
        format!("Keep system awake: {}", on_off(c.keep_awake)),
        format!("Prevent sleep: {}", on_off(c.prevent_sleep)),
        format!(
            "{}: {}",
            session.desktop().startup_label(),
            on_off(c.startup)
        ),
    ]
    .join("\n")
}

/// A group as a message: what it says, and the buttons under it. A path that leads
/// nowhere (the menu has changed since the button was made) shows the top.
fn page(session: &Session, path: &str) -> (String, Keyboard) {
    let nodes = menu(session);
    match group(&nodes, path) {
        Some((title, at, on)) if !title.is_empty() => {
            let text = if on {
                title
            } else {
                format!("{title}\n\n{NOT_NOW}")
            };
            (text, keyboard(at, path, on))
        }
        _ => (status(session), keyboard(&nodes, "", true)),
    }
}

fn show(session: &Session, chat: i64, path: &str) -> Vec<Call> {
    let (text, keys) = page(session, path);
    vec![Call::send(chat, &text, Some(&keys))]
}

fn say(chat: i64, text: &str) -> Vec<Call> {
    vec![Call::send(chat, text, None)]
}

// ---- What a message does -----------------------------------------------------------------

/// "/Menu@my_bot now" is the command "menu" with "now".
fn command(text: &str) -> Option<(String, &str)> {
    let text = text.trim().strip_prefix('/')?;
    let (name, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    let name = name.split('@').next().unwrap_or(name).to_ascii_lowercase();
    Some((name, rest.trim()))
}

/// "#FF8800" or "ff8800".
fn hex(text: &str) -> Option<i32> {
    let digits = text.strip_prefix('#').unwrap_or(text);
    (digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| i32::from_str_radix(digits, 16).ok())
        .flatten()
}

/// What a "-custom" colour choice sets: the indicator (none for the dress), and its name.
fn custom_colour(id: &str) -> (Option<Indicator>, &'static str) {
    match id.strip_suffix("-custom") {
        Some("dress") => (None, "Dress"),
        Some(i) if i == Indicator::LeftClick.id() => (Some(Indicator::LeftClick), "Left click"),
        Some(i) if i == Indicator::RightClick.id() => (Some(Indicator::RightClick), "Right click"),
        _ => (Some(Indicator::FindRing), "Find cursor ring"),
    }
}

/// What pressing a custom colour asks for.
fn hint(id: &str) -> String {
    let (_, label) = custom_colour(id);
    format!("{label}: send the colour as a message, like #FF8800.")
}

/// A colour typed after a custom colour button was pressed.
fn typed_colour(session: &mut Session, chat: i64, id: String, text: &str) -> Vec<Call> {
    let Some(rgb) = hex(text.trim()) else {
        set_awaiting(Some(Awaiting::Colour(id)));
        return say(
            chat,
            "That is not a colour. Send one like #FF8800, or /menu to leave it.",
        );
    };
    let (which, label) = custom_colour(&id);
    match which {
        Some(which) => session.choose_indicator_colour(which, rgb),
        None => session.choose_dress(rgb),
    }
    let path = find(&menu(session), &id, true, "").map_or(String::new(), |f| f.2);
    let (page_text, keys) = page(session, &path);
    let text = format!("{label}: #{rgb:06X}\n\n{page_text}");
    vec![Call::send(chat, &text, Some(&keys))]
}

fn on_text(session: &mut Session, chat: i64, text: &str, version: &str) -> Vec<Call> {
    match command(text) {
        Some((name, _)) => match name.as_str() {
            "start" | "menu" => show(session, chat, ""),
            "info" => say(chat, &format!("{}\n\nVersion {version}", status(session))),
            _ => say(chat, "No such command. Everything is under /menu."),
        },
        None => match awaiting() {
            Some(Awaiting::Colour(id)) => {
                set_awaiting(None);
                typed_colour(session, chat, id, text)
            }
            _ => say(chat, "Tap /menu for the buttons."),
        },
    }
}

fn on_press(
    session: &mut Session,
    chat: i64,
    callback: &str,
    message: i64,
    what: &str,
) -> Vec<Call> {
    let (mut note, mut alert, mut path) = (String::new(), false, None);
    if what == data::OFF {
        note = NOT_NOW.into();
    } else if let Some(id) = what.strip_prefix(data::HINT) {
        note = hint(id);
        alert = true;
        set_awaiting(Some(Awaiting::Colour(id.to_string())));
    } else if let Some(p) = what.strip_prefix(data::PAGE) {
        path = Some(p);
    } else if let Some(rest) = what.strip_prefix(data::APPLY) {
        let mut parts = rest.splitn(3, ':');
        if let (Some(p), Some(want), Some(id)) = (parts.next(), parts.next(), parts.next()) {
            path = Some(p);
            match find(&menu(session), id, true, "") {
                None => note = "That choice is no longer there.".into(),
                Some((_, false, _)) => note = NOT_NOW.into(),
                // A button from an old message may ask for what is already so.
                Some((ticked, true, _)) => {
                    if (ticked as i32).to_string() != want {
                        session.apply(id);
                    }
                    let c = session.choices();
                    let sleepy = matches!(id, "keep-awake" | "prevent-sleep")
                        && !c.keep_awake
                        && !c.prevent_sleep;
                    if sleepy {
                        // Fair warning, while it can still be undone.
                        note = SLEEPY.into();
                        alert = true;
                    }
                }
            }
        }
    }
    let mut calls = vec![Call::answer(callback, &note, alert)];
    if let Some(p) = path {
        let (text, keys) = page(session, p);
        calls.push(Call::edit(chat, message, &text, &keys));
    }
    calls
}

/// What the paired chat's message or button press does, and what to say back. On the UI
/// thread, where the choices live.
pub fn respond(session: &mut Session, update: &Update, version: &str) -> Vec<Call> {
    match &update.what {
        What::Text(text) => on_text(session, update.chat, text, version),
        What::Press {
            callback,
            message,
            data,
        } => on_press(session, update.chat, callback, *message, data),
    }
}

// ---- The worker --------------------------------------------------------------------------

/// What the worker needs.
pub struct Remote {
    pub telegram: Telegram,
    pub macros: Macros,
    pub desktop: Arc<dyn Desktop>,
    pub updates: Updates,
    pub journal: Arc<dyn Journal>,
}

fn nap(seconds: u64) {
    std::thread::sleep(Duration::from_secs(seconds));
}

fn brightness_page(displays: &[BrightnessInfo]) -> (String, Keyboard) {
    let mut text = String::from("Brightness\n");
    let mut keys: Keyboard = Vec::new();
    let several = displays.len() > 1;
    for (i, d) in displays.iter().enumerate() {
        let main = if d.primary && several { " (main)" } else { "" };
        let level = if d.supported {
            format!("{}%", d.percent)
        } else {
            "cannot be set from here".into()
        };
        text.push_str(&format!("\n{}. {}{main}: {level}", i + 1, d.name));
    }
    if displays.is_empty() {
        text.push_str("\nNo display was found.");
    }
    // Each preset applies to every supported display.
    if displays.iter().any(|d| d.supported) {
        let button = |percent: i32| {
            Button::new(
                format!("{percent}%"),
                format!("{}:{percent}", data::BRIGHTNESS),
            )
        };
        keys.push([0, 10, 20, 30].map(button).to_vec());
        keys.push([40, 50, 60, 70].map(button).to_vec());
        keys.push([80, 90, 100].map(button).to_vec());
    }
    keys.push(vec![
        Button::new("\u{2039} Menu", data::PAGE),
        Button::new("\u{1F504} Refresh", data::BRIGHTNESS),
    ]);
    (text, keys)
}

const NO_MACROS_HERE: &str =
    "Mouse macros do not work under Wayland: it lets no program watch or work the mouse.";

/// The mouse macros, each a button that plays it.
fn macros_page(names: &[String], playing: bool) -> (String, Keyboard) {
    let mut text = String::from("Mouse macros\n\n");
    text.push_str(if names.is_empty() {
        "None yet. Record one on the computer: the tray menu, Mouse macros, Record."
    } else {
        "Tap one to play it on the computer; a picture of the screen follows. The same \
         places are clicked as when it was recorded, whatever is there now."
    });
    if playing {
        text.push_str("\n\nOne is being played now.");
    }
    let play = |name: &String| {
        Button::new(
            format!("\u{25B6} {name}"),
            format!("{}:{name}", data::MACROS),
        )
    };
    let mut keys = rows(names.iter().map(play).collect());
    if playing {
        keys.push(vec![Button::new("\u{23F9} Stop", data::MACRO_STOP)]);
    }
    if !names.is_empty() {
        keys.push(vec![Button::new(
            "\u{2699} Rename or delete",
            data::MACRO_MANAGE,
        )]);
    }
    keys.push(vec![
        Button::new("\u{2039} Menu", data::PAGE),
        Button::new("\u{1F504} Refresh", data::MACROS),
    ]);
    (text, keys)
}

/// Each macro with a button to rename it and one to delete it.
fn manage_page(names: &[String]) -> (String, Keyboard) {
    let text = if names.is_empty() {
        "Rename or delete a macro\n\nThere are none."
    } else {
        "Rename or delete a macro\n\nTap a name to give it a new one."
    };
    let mut keys: Keyboard = names
        .iter()
        .map(|name| {
            vec![
                Button::new(
                    format!("\u{270F} {name}"),
                    format!("{}{name}", data::MACRO_RENAME),
                ),
                Button::new("\u{1F5D1} Delete", format!("{}{name}", data::MACRO_DELETE)),
            ]
        })
        .collect();
    keys.push(vec![
        Button::new("\u{2039} Macros", data::MACROS),
        Button::new("\u{1F504} Refresh", data::MACRO_MANAGE),
    ]);
    (text.into(), keys)
}

fn delete_page(name: &str) -> (String, Keyboard) {
    let keys = vec![vec![
        Button::new("Yes, delete", format!("{}{name}", data::MACRO_DELETE_SURE)),
        Button::new("\u{2039} No", data::MACRO_MANAGE),
    ]];
    (format!("Delete the macro {name}?"), keys)
}

fn exit_page() -> (String, Keyboard) {
    let keys = vec![vec![
        Button::new("Yes, quit", data::EXIT_SURE),
        Button::new("\u{2039} No", data::PAGE),
    ]];
    let text = "Quit IdleWalker on the computer?\n\n\
                This remote stops with it: it can only be started again there.";
    (text.into(), keys)
}

fn power_page(what: Power) -> (String, Keyboard) {
    let (text, yes, sure) = match what {
        Power::ShutDown => (
            "Shut the computer down?\n\n\
             It cannot be switched on again from here, and unsaved work may be lost.",
            "Yes, shut down",
            data::SHUT_DOWN_SURE,
        ),
        Power::Restart => (
            "Restart the computer?\n\n\
             Unsaved work may be lost. This remote answers again once you are signed in \
             and IdleWalker has started.",
            "Yes, restart",
            data::RESTART_SURE,
        ),
    };
    let keys = vec![vec![
        Button::new(yes, sure),
        Button::new("\u{2039} No", data::PAGE),
    ]];
    (text.into(), keys)
}

/// What is left to do once the answer has been sent.
enum After {
    Nothing,
    /// IdleWalker quits.
    Quit,
    Power(Power),
    /// A picture of every screen is taken and sent.
    Screenshots,
    /// A mouse macro is played, and a picture of what came of it sent.
    Play(String),
}

impl Remote {
    fn about(&self) -> String {
        format!(
            "IdleWalker {}\n{}",
            self.updates.current_version(),
            self.updates.releases_url()
        )
    }

    /// Slow, and needs the network.
    fn update_report(&self) -> String {
        match self.updates.check() {
            UpdateState::Available(v) => format!(
                "IdleWalker {v} is out; this is {}.\n{}",
                self.updates.current_version(),
                self.updates.releases_url()
            ),
            UpdateState::UpToDate => {
                format!(
                    "IdleWalker {} is the newest.",
                    self.updates.current_version()
                )
            }
            _ => "The update check did not get through.".into(),
        }
    }

    /// Reads every display, after setting some: all that can be set (`None`) or one.
    /// Slow: DDC/CI takes its time.
    fn brightness(&self, set: Option<(Option<usize>, i32)>) -> (String, Keyboard) {
        let mut displays = self.desktop.brightness_enumerate();
        if let Some((which, percent)) = set {
            let percent = percent.clamp(0, 100);
            for (i, d) in displays.iter_mut().enumerate() {
                if d.supported && which.is_none_or(|w| w == i) {
                    self.desktop.brightness_write(&d.key, percent);
                    d.percent = percent;
                }
            }
        }
        brightness_page(&displays)
    }

    fn macros_page(&self) -> (String, Keyboard) {
        macros_page(&self.macros.list(), self.macros.playing())
    }

    fn stop_macro(&self) -> &'static str {
        if self.macros.stop_playing() {
            "Stopping the macro."
        } else {
            "No macro is being played."
        }
    }

    /// A new name typed after a macro's rename button was pressed.
    fn typed_rename(&self, chat: i64, old: String, new: &str) -> Vec<Call> {
        let new = new.trim();
        let retry = |why: String| {
            set_awaiting(Some(Awaiting::Rename(old.clone())));
            format!("{why} Send another name, or /menu to leave it.")
        };
        let text = match self.macros.rename(&old, new) {
            Renamed::Done => format!("{old} is now {new}."),
            Renamed::Missing => format!("There is no macro called {old} any more."),
            Renamed::BadName => {
                retry("A name is letters, digits, - and _, 32 at most, with no spaces.".into())
            }
            Renamed::Taken => retry(format!("Another macro is already called {new}.")),
            Renamed::Failed => "The file could not be renamed.".into(),
        };
        let (_, keys) = manage_page(&self.macros.list());
        vec![Call::send(chat, &text, Some(&keys))]
    }

    /// Plays a macro on a thread of its own, so that the chat is still heard meanwhile
    /// (a /stop, above all), then says how it went and shows the screen.
    fn play(&self, token: &str, chat: i64, name: String) {
        let (telegram, macros) = (self.telegram.clone(), self.macros.clone());
        let (desktop, token) = (self.desktop.clone(), token.to_string());
        let _ = std::thread::Builder::new()
            .name("IdleWalker mouse macro".into())
            .spawn(move || {
                let played = macros.play(&name);
                let text = match played {
                    Played::Done => format!("{name} was played."),
                    Played::Stopped => format!("{name} was stopped part-way."),
                    Played::Missing => format!("{name} is gone, or its file is not a macro."),
                    Played::Busy => "Not now: a macro is being played or recorded.".into(),
                    Played::Locked => format!(
                        "{name} was not played: the screen is locked. Unlock it, then try again."
                    ),
                };
                telegram.call(&token, &Call::send(chat, &text, None), 20);
                if matches!(played, Played::Done | Played::Stopped) {
                    // A moment for the last click to show on the screen.
                    std::thread::sleep(Duration::from_millis(800));
                    telegram.send_screenshots(&*desktop, &token, chat);
                }
            });
    }

    /// What the worker answers itself: the slow things, and leaving. `None` for what is
    /// the UI thread's to answer. With it, what to do once it has been said.
    fn own(&self, update: &Update) -> Option<(Vec<Call>, After)> {
        let chat = update.chat;
        let calls = match &update.what {
            What::Text(text) => {
                let Some((name, _)) = command(text) else {
                    return match awaiting() {
                        Some(Awaiting::Rename(old)) => {
                            set_awaiting(None);
                            Some((self.typed_rename(chat, old, text), After::Nothing))
                        }
                        _ => None,
                    };
                };
                match name.as_str() {
                    "shutdown" | "restart" => {
                        let what = if name == "restart" {
                            Power::Restart
                        } else {
                            Power::ShutDown
                        };
                        let (text, keys) = power_page(what);
                        vec![Call::send(chat, &text, Some(&keys))]
                    }
                    _ => return None,
                }
            }
            What::Press {
                callback,
                message,
                data: what,
            } => {
                let done = Call::answer(callback, "", false);
                if what == data::UPDATE {
                    vec![done, Call::send(chat, &self.update_report(), None)]
                } else if what == data::ABOUT {
                    vec![done, Call::send(chat, &self.about(), None)]
                } else if what == data::SCREENSHOT {
                    let taking = Call::answer(callback, "Taking the picture...", false);
                    return Some((vec![taking], After::Screenshots));
                } else if what == data::MACROS {
                    let (text, keys) = self.macros_page();
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if what == data::MACRO_STOP {
                    vec![Call::answer(callback, self.stop_macro(), false)]
                } else if what == data::MACRO_MANAGE {
                    let (text, keys) = manage_page(&self.macros.list());
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if let Some(name) = what.strip_prefix(data::MACRO_RENAME) {
                    if self.macros.find(name).is_none() {
                        vec![Call::answer(callback, "That macro is gone.", true)]
                    } else {
                        set_awaiting(Some(Awaiting::Rename(name.to_string())));
                        let ask = format!("Send the new name for {name} as a message.");
                        vec![Call::answer(callback, &ask, true)]
                    }
                } else if let Some(name) = what.strip_prefix(data::MACRO_DELETE_SURE) {
                    let note = if self.macros.delete(name) {
                        format!("{name} is deleted.")
                    } else {
                        "That macro is gone.".to_string()
                    };
                    let (text, keys) = manage_page(&self.macros.list());
                    vec![
                        Call::answer(callback, &note, false),
                        Call::edit(chat, *message, &text, &keys),
                    ]
                } else if let Some(name) = what.strip_prefix(data::MACRO_DELETE) {
                    let (text, keys) = delete_page(name);
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if let Some(name) = what.strip_prefix("m:") {
                    if !self.macros.supported() {
                        vec![Call::answer(callback, NO_MACROS_HERE, true)]
                    } else if self.macros.find(name).is_none() {
                        vec![Call::answer(callback, "That macro is gone.", true)]
                    } else {
                        let playing = Call::answer(callback, &format!("Playing {name}..."), false);
                        return Some((vec![playing], After::Play(name.to_string())));
                    }
                } else if what == data::EXIT {
                    let (text, keys) = exit_page();
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if what == data::EXIT_SURE {
                    let gone = Call::edit(chat, *message, "IdleWalker has quit.", &Vec::new());
                    return Some((vec![done, gone], After::Quit));
                } else if what == data::SHUT_DOWN || what == data::RESTART {
                    let (text, keys) = power_page(if what == data::RESTART {
                        Power::Restart
                    } else {
                        Power::ShutDown
                    });
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if what == data::SHUT_DOWN_SURE || what == data::RESTART_SURE {
                    let (power, text) = if what == data::RESTART_SURE {
                        (Power::Restart, "Restarting the computer...")
                    } else {
                        (Power::ShutDown, "Shutting the computer down...")
                    };
                    let going = Call::edit(chat, *message, text, &Vec::new());
                    return Some((vec![done, going], After::Power(power)));
                } else {
                    let rest = what.strip_prefix(data::BRIGHTNESS)?;
                    // "b:<percent>" is every display; "b:<display>:<percent>" is what
                    // the buttons of an older message still say.
                    let set = rest
                        .strip_prefix(':')
                        .and_then(|r| match r.split_once(':') {
                            Some((which, percent)) => {
                                Some((Some(which.parse().ok()?), percent.parse().ok()?))
                            }
                            None => Some((None, r.parse().ok()?)),
                        });
                    if !rest.is_empty() && set.is_none() {
                        return None;
                    }
                    let (text, keys) = self.brightness(set);
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                }
            }
        };
        Some((calls, After::Nothing))
    }

    /// One update. False once the program is leaving.
    fn handle(&self, proxy: &EventLoopProxy<UserEvent>, token: &str, mut update: Update) -> bool {
        let send = |calls: Vec<Call>| {
            for call in &calls {
                self.telegram.call(token, call, 20);
            }
        };
        if self.telegram.pairs(&update) {
            self.journal.note("Telegram remote: paired with a chat");
            send(say(
                update.chat,
                "Paired: this chat, and only this one, now works IdleWalker on that computer.",
            ));
            // Whatever the first message said, it only introduces the chat.
            update.what = What::Text("/menu".into());
        }
        // Strangers, and groups before pairing, get no answer: the bot gives nothing away.
        if self.telegram.config().chat != Some(update.chat) {
            return true;
        }
        // A command or a press moves on from whatever a button had asked to be typed;
        // the press itself may ask again.
        let moves_on = match &update.what {
            What::Text(text) => command(text).is_some(),
            What::Press { .. } => true,
        };
        if moves_on {
            set_awaiting(None);
        }
        if let Some((calls, after)) = self.own(&update) {
            let chat = update.chat;
            send(calls);
            match after {
                After::Nothing => {}
                After::Quit => {
                    self.journal.note("Telegram remote: asked to quit");
                    let _ = proxy.send_event(UserEvent::Menu("exit".into()));
                    return false;
                }
                After::Power(what) => {
                    self.journal
                        .note(&format!("Telegram remote: asked to {what:?}"));
                    if !self.desktop.power(what) {
                        send(say(chat, "The system would not take the request."));
                    }
                }
                After::Screenshots => {
                    self.journal.note("Telegram remote: asked for a screenshot");
                    let (taken, arrived) =
                        self.telegram.send_screenshots(&*self.desktop, token, chat);
                    if taken == 0 {
                        send(say(
                            chat,
                            "The system would not give a picture of the screen.",
                        ));
                    } else if arrived < taken {
                        let lost = taken - arrived;
                        send(say(chat, &format!("{lost} of {taken} could not be sent.")));
                    }
                }
                After::Play(name) => {
                    self.journal
                        .note(&format!("Telegram remote: asked to play {name}"));
                    self.play(token, chat, name);
                }
            }
            return true;
        }
        let (reply, answer) = mpsc::channel();
        if proxy.send_event(UserEvent::Remote(update, reply)).is_err() {
            return false;
        }
        send(
            answer
                .recv_timeout(Duration::from_secs(10))
                .unwrap_or_default(),
        );
        true
    }

    fn run(self, proxy: EventLoopProxy<UserEvent>) {
        // Where the next poll starts; `None` until what was sent earlier has been skipped.
        let mut offset: Option<i64> = None;
        // The token whose commands Telegram has been told.
        let mut announced: Option<String> = None;
        let mut refused = false;
        loop {
            if !self.telegram.enabled() {
                offset = None;
                nap(2);
                continue;
            }
            let Some(token) = self.telegram.config().token else {
                nap(2);
                continue;
            };
            let listening = offset.is_some();
            let polled = match offset {
                Some(from) => self.telegram.poll(&token, from, LONG_POLL_SECONDS),
                None => self.telegram.poll(&token, -1, 0),
            };
            let batch = match polled {
                Polled::Batch(batch) => batch,
                Polled::Unreachable => {
                    nap(10);
                    continue;
                }
                Polled::Refused(why) => {
                    if !refused {
                        refused = true;
                        self.journal
                            .note(&format!("Telegram remote: the bot was refused: {why}"));
                    }
                    nap(30);
                    continue;
                }
            };
            refused = false;
            if announced.as_deref() != Some(token.as_str()) {
                self.telegram.announce_commands(&token);
                announced = Some(token.clone());
            }
            offset = Some(batch.last.map_or(offset.unwrap_or(0), |last| last + 1));
            for update in batch.updates {
                // What was sent while nobody listened is not obeyed, however long ago.
                // Before pairing it still counts as the first hello: Start is often
                // pressed before the token is pasted.
                if !listening && self.telegram.link() != Link::Unpaired {
                    continue;
                }
                if !self.handle(&proxy, &token, update) {
                    return;
                }
            }
        }
    }
}

/// Starts the worker; it idles while the remote is switched off.
pub fn spawn(proxy: EventLoopProxy<UserEvent>, remote: Remote) {
    let _ = std::thread::Builder::new()
        .name("IdleWalker Telegram remote".into())
        .spawn(move || remote.run(proxy));
}

#[cfg(test)]
mod tests {
    use super::*;
    use idlewalker_application::telegram::COMMANDS;

    fn check(id: &str, label: &str, enabled: bool, checked: bool) -> Node {
        Node::Check {
            id: id.into(),
            label: label.into(),
            enabled,
            checked,
        }
    }

    fn sample() -> Vec<Node> {
        vec![
            check("walkers", "Walkers", true, false),
            Node::Sep,
            Node::Sub {
                label: "Minions".into(),
                enabled: false,
                children: vec![
                    check("minions", "Enabled", true, true),
                    Node::Sub {
                        label: "Dress: Red".into(),
                        enabled: true,
                        children: vec![check("dress-custom", "Custom...", true, false)],
                    },
                ],
            },
            Node::Item {
                id: "posh".into(),
                label: "Terminal prompt...".into(),
                enabled: true,
            },
            Node::Item {
                id: "brightness".into(),
                label: "Brightness...".into(),
                enabled: true,
            },
        ]
    }

    #[test]
    fn commands_are_read_whatever_their_dress() {
        assert_eq!(command("/Menu@my_bot  now "), Some(("menu".into(), "now")));
        assert_eq!(command("/menu"), Some(("menu".into(), "")));
        assert_eq!(command("hello"), None);
        assert_eq!(hex("#FF8800"), Some(0xFF8800));
        assert_eq!(hex("ff8800"), Some(0xFF8800));
        assert_eq!(hex("#FF88"), None);
        assert_eq!(hex("+12345"), None);
    }

    #[test]
    fn a_tick_box_is_found_with_its_group_and_whether_it_can_be_used() {
        let nodes = sample();
        assert_eq!(
            find(&nodes, "walkers", true, ""),
            Some((false, true, String::new()))
        );
        // In a group that is off, so not to be used.
        assert_eq!(
            find(&nodes, "minions", true, ""),
            Some((true, false, "2".into()))
        );
        assert_eq!(
            find(&nodes, "dress-custom", true, ""),
            Some((false, false, "2.1".into()))
        );
        assert_eq!(find(&nodes, "nothing", true, ""), None);
    }

    #[test]
    fn the_top_is_buttons_for_what_a_chat_can_do() {
        let nodes = sample();
        let (title, at, on) = group(&nodes, "").unwrap();
        assert!(title.is_empty() && on);
        let keys = keyboard(at, "", on);
        let flat: Vec<(&str, &str)> = keys
            .iter()
            .flatten()
            .map(|b| (b.label.as_str(), b.data.as_str()))
            .collect();
        assert_eq!(
            flat,
            vec![
                ("\u{2B1C} Walkers", "a::1:walkers"),
                ("\u{1F512} Minions \u{203A}", "n"),
                ("Brightness", "b"),
                ("\u{1F4F8} Screenshot", "c"),
                ("\u{1F5B1} Macros", "m"),
                ("\u{23FB} Shut down", "s"),
                ("\u{1F501} Restart", "r"),
                ("\u{1F504} Refresh", "p:"),
            ]
        );
        // Two short ones share a row.
        assert_eq!(keys[0].len(), 2);
    }

    #[test]
    fn a_group_in_a_group_leads_back_to_its_parent() {
        let nodes = sample();
        let (title, at, on) = group(&nodes, "2.1").unwrap();
        assert_eq!(title, "Dress: Red");
        assert!(!on);
        let keys = keyboard(at, "2.1", true);
        assert_eq!(keys[0][0].data, "h:dress-custom");
        assert_eq!(keys[1][0].data, "p:2");
        assert_eq!(keys[1][1].data, "p:2.1");
        let (_, at, _) = group(&nodes, "2").unwrap();
        assert_eq!(keyboard(at, "2", true)[0][0].data, "a:2:0:minions");
        assert_eq!(keyboard(at, "2", true).last().unwrap()[0].data, "p:");
        // A path that leads nowhere, or to something that is not a group.
        assert!(group(&nodes, "9").is_none());
        assert!(group(&nodes, "0").is_none());
        assert!(group(&nodes, "x").is_none());
    }

    #[test]
    fn every_button_fits_what_telegram_allows() {
        let nodes = sample();
        for path in ["", "2", "2.1"] {
            let (_, at, on) = group(&nodes, path).unwrap();
            for b in keyboard(at, path, on).iter().flatten() {
                assert!(b.data.len() <= 64, "{}", b.data);
            }
        }
        assert!(hint("dress-custom").starts_with("Dress"));
        let names = vec!["a".repeat(32)];
        let (_, manage) = manage_page(&names);
        let (_, delete) = delete_page(&names[0]);
        for b in manage.iter().chain(delete.iter()).flatten() {
            assert!(b.data.len() <= 64, "{}", b.data);
        }
    }

    #[test]
    fn only_the_four_commands_are_listed() {
        let names: Vec<&str> = COMMANDS.iter().map(|(c, _)| *c).collect();
        assert_eq!(names, ["menu", "info", "shutdown", "restart"]);
    }
}
