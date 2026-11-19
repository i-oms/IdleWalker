//! The Telegram remote's presentation: the tray menu shown as buttons in a chat, and
//! slash commands for the same choices. The buttons are made from the very description
//! the tray menu is drawn from, so whatever the menu offers the chat offers too.
//!
//! A worker waits on Telegram. What it hears goes to the UI thread, where the choices
//! live, and the answer comes back to be sent; what is slow (brightness, the update
//! check) the worker does itself.
use super::UserEvent;
use crate::prelude::*;
use crate::tray::{self, Node};
use idlewalker_application::telegram::{
    Button, Call, Keyboard, Link, Polled, Update, What, COMMANDS, LONG_POLL_SECONDS,
};
use idlewalker_application::{
    BrightnessInfo, Desktop, Indicator, Journal, Session, Telegram, UpdateState, Updates,
};
use std::sync::mpsc;
use std::sync::Arc;
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
    /// Alone, the brightness page; "b:<display>:<percent>" sets one.
    pub const BRIGHTNESS: &str = "b";
    pub const UPDATE: &str = "u";
    pub const ABOUT: &str = "i";
    pub const EXIT: &str = "x";
    pub const EXIT_SURE: &str = "x!";
}

const NOT_NOW: &str = "Not available while its feature is off.";

/// Commands that switch one thing: the command, the menu id, and what to call it.
const SWITCHES: [(&str, &str, &str); 9] = [
    ("awake", "keep-awake", "Keep system awake"),
    ("walkers", "walkers", "Walkers"),
    ("minions", "minions", "Minions"),
    ("birds", "birds", "Angry Birds"),
    ("hide", "hide-on-move", "Hide on mouse move"),
    ("corners", "corners", "Hot corners"),
    ("highlight", "click-highlight", "Highlight mouse clicks"),
    ("findcursor", "find-cursor", "Find cursor"),
    ("startup", "startup", "Start at sign-in"),
];

// ---- The menu as a keyboard --------------------------------------------------------------

/// The menu without the remote's own group: it cannot switch itself off from afar.
fn menu(session: &Session) -> Vec<Node> {
    tray::build(session.choices(), &**session.desktop(), None)
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

/// "/Awake@my_bot on" is the command "awake" with "on".
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

/// "5", "5s", "2m", "2 minutes": seconds.
fn seconds(text: &str) -> Option<i32> {
    let digits = text.bytes().take_while(|b| b.is_ascii_digit()).count();
    let n: i32 = text[..digits].parse().ok()?;
    match text[digits..].trim().bytes().next() {
        None | Some(b's') => Some(n),
        Some(b'm') => n.checked_mul(60),
        _ => None,
    }
}

fn help() -> String {
    let mut out = String::from("IdleWalker's remote. Tap /menu for buttons, or type:\n");
    for (name, what) in COMMANDS {
        out.push_str(&format!("\n/{name} - {what}"));
    }
    out
}

/// Sets a tick box; with nothing said, shows the group it is in.
fn switch(session: &mut Session, chat: i64, id: &str, label: &str, arg: &str) -> Vec<Call> {
    let Some((ticked, _, path)) = find(&menu(session), id, true, "") else {
        return say(chat, "That choice is not there.");
    };
    let want = match arg.to_ascii_lowercase().as_str() {
        "" => return show(session, chat, &path),
        "on" | "yes" | "1" => true,
        "off" | "no" | "0" => false,
        _ => return say(chat, "Say on or off."),
    };
    if ticked != want {
        session.apply(id);
    }
    // Asked again, not assumed: the system may have refused.
    let now = find(&menu(session), id, true, "").is_some_and(|(t, _, _)| t);
    let note = if now == want {
        ""
    } else {
        " (the system would not change it)"
    };
    say(chat, &format!("{label}: {}{note}", on_off(now)))
}

/// One of the offered values, or the group to pick one from.
fn choose(
    session: &mut Session,
    chat: i64,
    kind: &str,
    value: Option<i32>,
    offered: Vec<i32>,
) -> Vec<Call> {
    match value.filter(|v| offered.contains(v)) {
        Some(v) => {
            session.apply(&format!("{kind}:{v}"));
            let c = session.choices();
            say(
                chat,
                &match kind {
                    "idle" => format!("Appear after: {}", tray::describe_idle(c.idle_seconds)),
                    _ => format!("Hot corner delay: {} ms", c.delay_ms),
                },
            )
        }
        None => {
            let first = format!("{kind}:{}", offered[0]);
            let path = find(&menu(session), &first, true, "").map_or(String::new(), |f| f.2);
            show(session, chat, &path)
        }
    }
}

fn dress(session: &mut Session, chat: i64, arg: &str) -> Vec<Call> {
    let arg = arg.to_lowercase();
    let named = || {
        (0..SETTINGS_DRESS_NAMES.len())
            .find(|&i| SETTINGS_DRESS_NAMES[i].to_lowercase().contains(&arg))
            .map(|i| SETTINGS_DRESS_COLORS[i])
    };
    let chosen = match arg.as_str() {
        "" => None,
        "random" => Some(Settings::RANDOM_DRESS),
        _ => hex(&arg).or_else(named),
    };
    match chosen {
        Some(c) => {
            session.choose_dress(c);
            let now = tray::describe_dress(session.choices().dress);
            say(chat, &format!("Dress: {now}"))
        }
        None => {
            let first = format!("dress:{}", SETTINGS_DRESS_COLORS[0]);
            let path = find(&menu(session), &first, true, "").map_or(String::new(), |f| f.2);
            show(session, chat, &path)
        }
    }
}

fn colour(session: &mut Session, chat: i64, arg: &str) -> Vec<Call> {
    let arg = arg.to_lowercase();
    let (which, value) = arg
        .split_once(char::is_whitespace)
        .unwrap_or((arg.as_str(), ""));
    let which = match which {
        "left" => Some((Indicator::LeftClick, "Left click")),
        "right" => Some((Indicator::RightClick, "Right click")),
        "ring" | "find" => Some((Indicator::FindRing, "Find cursor ring")),
        _ => None,
    };
    let value = value.trim();
    let rgb = hex(value).or_else(|| {
        tray::INDICATOR_COLOURS
            .iter()
            .find(|(name, _)| name.to_lowercase() == value)
            .map(|(_, rgb)| *rgb)
    });
    match (which, rgb) {
        (Some((which, label)), Some(rgb)) => {
            session.choose_indicator_colour(which, rgb);
            say(chat, &format!("{label}: #{rgb:06X}"))
        }
        _ => say(
            chat,
            "Say which and what: left, right or ring, then a colour.\n\
             For example: /colour left #FF8800 or /colour ring cyan",
        ),
    }
}

fn on_text(session: &mut Session, chat: i64, text: &str) -> Vec<Call> {
    let Some((name, arg)) = command(text) else {
        return say(
            chat,
            "Tap /menu for the settings, or /help for the commands.",
        );
    };
    match name.as_str() {
        "start" | "menu" => show(session, chat, ""),
        "status" => say(chat, &status(session)),
        "help" => say(chat, &help()),
        "idle" => {
            let offered = SETTINGS_IDLE_CHOICES.items().collect();
            choose(session, chat, "idle", seconds(arg), offered)
        }
        "delay" => {
            let ms = arg.trim_end_matches("ms").trim().parse().ok();
            let offered = SETTINGS_HOT_CORNER_DELAY_CHOICES.items().collect();
            choose(session, chat, "delay", ms, offered)
        }
        "dress" => dress(session, chat, arg),
        "colour" | "color" => colour(session, chat, arg),
        _ => match SWITCHES.iter().find(|(command, _, _)| *command == name) {
            Some((_, id, label)) => switch(session, chat, id, label, arg),
            None => say(chat, "No such command. /help lists them."),
        },
    }
}

/// What typing a colour looks like, for the choices that open a picker on the computer.
fn hint(id: &str) -> String {
    let which = match id.strip_suffix("-custom") {
        Some("dress") => return "Send it as text: /dress #FF8800".into(),
        Some(i) if i == Indicator::LeftClick.id() => "left",
        Some(i) if i == Indicator::RightClick.id() => "right",
        _ => "ring",
    };
    format!("Send it as text: /colour {which} #FF8800")
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
pub fn respond(session: &mut Session, update: &Update) -> Vec<Call> {
    match &update.what {
        What::Text(text) => on_text(session, update.chat, text),
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
        if d.supported {
            let button = |percent: i32| {
                let label = if several {
                    format!("{} \u{00B7} {percent}", i + 1)
                } else {
                    format!("{percent}%")
                };
                Button::new(label, format!("{}:{i}:{percent}", data::BRIGHTNESS))
            };
            keys.push([0, 25, 50, 75, 100].map(button).to_vec());
        }
    }
    if displays.is_empty() {
        text.push_str("\nNo display was found.");
    }
    keys.push(vec![
        Button::new("\u{2039} Menu", data::PAGE),
        Button::new("\u{1F504} Refresh", data::BRIGHTNESS),
    ]);
    (text, keys)
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

    /// What the worker answers itself: the slow things, and leaving. `None` for what is
    /// the UI thread's to answer. The flag says the program is to quit afterwards.
    fn own(&self, update: &Update) -> Option<(Vec<Call>, bool)> {
        let chat = update.chat;
        let calls = match &update.what {
            What::Text(text) => {
                let (name, arg) = command(text)?;
                match name.as_str() {
                    "brightness" => {
                        let percent = arg.trim_end_matches('%').trim().parse::<i32>().ok();
                        let (text, keys) = self.brightness(percent.map(|p| (None, p)));
                        vec![Call::send(chat, &text, Some(&keys))]
                    }
                    "update" => say(chat, &self.update_report()),
                    "about" => say(chat, &self.about()),
                    "exit" => {
                        let (text, keys) = exit_page();
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
                } else if what == data::EXIT {
                    let (text, keys) = exit_page();
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                } else if what == data::EXIT_SURE {
                    let gone = Call::edit(chat, *message, "IdleWalker has quit.", &Vec::new());
                    return Some((vec![done, gone], true));
                } else {
                    let rest = what.strip_prefix(data::BRIGHTNESS)?;
                    let set = rest.strip_prefix(':').and_then(|r| {
                        let (which, percent) = r.split_once(':')?;
                        Some((Some(which.parse().ok()?), percent.parse().ok()?))
                    });
                    if !rest.is_empty() && set.is_none() {
                        return None;
                    }
                    let (text, keys) = self.brightness(set);
                    vec![done, Call::edit(chat, *message, &text, &keys)]
                }
            }
        };
        Some((calls, false))
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
        if let Some((calls, leaving)) = self.own(&update) {
            send(calls);
            if leaving {
                self.journal.note("Telegram remote: asked to quit");
                let _ = proxy.send_event(UserEvent::Menu("exit".into()));
            }
            return !leaving;
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
        assert_eq!(command("/Awake@my_bot  on "), Some(("awake".into(), "on")));
        assert_eq!(command("/menu"), Some(("menu".into(), "")));
        assert_eq!(command("hello"), None);
        assert_eq!(hex("#FF8800"), Some(0xFF8800));
        assert_eq!(hex("ff8800"), Some(0xFF8800));
        assert_eq!(hex("#FF88"), None);
        assert_eq!(hex("+12345"), None);
        assert_eq!(seconds("5"), Some(5));
        assert_eq!(seconds("5s"), Some(5));
        assert_eq!(seconds("2 minutes"), Some(120));
        assert_eq!(seconds("2m"), Some(120));
        assert_eq!(seconds("soon"), None);
        assert_eq!(seconds("5 hours"), None);
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
        assert!(hint("left-click-colour-custom").contains("/colour left"));
        assert!(hint("find-colour-custom").contains("/colour ring"));
        assert!(hint("dress-custom").contains("/dress"));
    }

    #[test]
    fn every_switch_is_a_listed_command() {
        for (name, _, _) in SWITCHES {
            assert!(COMMANDS.iter().any(|(c, _)| *c == name), "{name}");
        }
    }
}
