//! The Telegram remote: a bot of the person's own, through which the menu can be worked
//! from a phone. This is the bot's side of it: where the token and the paired chat are
//! kept, how a chat is paired, and what is said to Telegram's Bot API. What a message
//! does to the menu is the presentation layer's business.
//!
//! Only one chat is ever obeyed: the first to write to the bot once its token is set.
//! The program only ever calls out (long polling), so no port is opened on the machine.
use crate::jsonc;
use crate::ports::{WebPost, Workstation};
use crate::preferences::Preferences;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const API: &str = "https://api.telegram.org";

/// How long Telegram holds a poll open when there is nothing to say.
pub const LONG_POLL_SECONDS: u32 = 25;

/// The slash commands, as Telegram lists them when "/" is typed.
pub const COMMANDS: [(&str, &str); 23] = [
    ("menu", "Every setting, as buttons"),
    ("status", "How things stand"),
    ("awake", "Keep system awake: on or off"),
    ("nosleep", "Prevent sleep: on or off"),
    ("walkers", "The walkers: on or off"),
    ("minions", "The Minions: on or off"),
    ("birds", "The Angry Birds: on or off"),
    ("hide", "Hide on mouse move: on or off"),
    ("idle", "Rest before they appear, like 5 or 2m"),
    ("dress", "Overalls: a name, random or #RRGGBB"),
    ("corners", "Hot corners: on or off"),
    ("delay", "Hot corner delay in ms"),
    ("highlight", "Highlight mouse clicks: on or off"),
    ("findcursor", "Find cursor: on or off"),
    ("colour", "left, right or ring, then #RRGGBB"),
    ("brightness", "Screen brightness, like 40"),
    ("startup", "Start at sign-in: on or off"),
    ("update", "Check for a newer version"),
    ("about", "Which version is running"),
    ("shutdown", "Shut the computer down"),
    ("restart", "Restart the computer"),
    ("exit", "Quit IdleWalker"),
    ("help", "The commands"),
];

const TEMPLATE: &str = "\
# IdleWalker's Telegram remote.
#
# 1. In Telegram, talk to @BotFather, send /newbot and answer its questions.
# 2. Paste the token it gives you after \"token =\" below, and save this file.
# 3. Open your new bot in Telegram and press Start (or send it any message).
#
# The first chat to write to the bot is paired with this computer, and from then
# on it is the only one obeyed. Its number is kept after \"chat =\"; empty that to
# pair again. Whoever has the token can read what is sent to the bot: keep it to
# yourself.
token =
chat =
";

// ---- What is said to Telegram ------------------------------------------------------------

/// A JSON string literal.
fn json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A button under a message: pressing it sends `data` back.
#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    pub label: String,
    /// At most 64 bytes, Telegram's limit.
    pub data: String,
}

impl Button {
    pub fn new(label: impl Into<String>, data: impl Into<String>) -> Button {
        Button {
            label: label.into(),
            data: data.into(),
        }
    }
}

/// Rows of buttons.
pub type Keyboard = Vec<Vec<Button>>;

fn keyboard_json(keys: &Keyboard) -> String {
    let rows: Vec<String> = keys
        .iter()
        .map(|row| {
            let buttons: Vec<String> = row
                .iter()
                .map(|b| {
                    format!(
                        "{{\"text\":{},\"callback_data\":{}}}",
                        json(&b.label),
                        json(&b.data)
                    )
                })
                .collect();
            format!("[{}]", buttons.join(","))
        })
        .collect();
    format!("{{\"inline_keyboard\":[{}]}}", rows.join(","))
}

/// One request to the Bot API.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub method: &'static str,
    /// The JSON that goes with it.
    pub body: String,
}

impl Call {
    /// A new message, with buttons under it or without.
    pub fn send(chat: i64, text: &str, keys: Option<&Keyboard>) -> Call {
        let markup = keys.map_or(String::new(), |k| {
            format!(",\"reply_markup\":{}", keyboard_json(k))
        });
        Call {
            method: "sendMessage",
            body: format!("{{\"chat_id\":{chat},\"text\":{}{markup}}}", json(text)),
        }
    }

    /// A message rewritten where it stands: how a pressed button shows its new state.
    pub fn edit(chat: i64, message: i64, text: &str, keys: &Keyboard) -> Call {
        Call {
            method: "editMessageText",
            body: format!(
                "{{\"chat_id\":{chat},\"message_id\":{message},\"text\":{},\"reply_markup\":{}}}",
                json(text),
                keyboard_json(keys)
            ),
        }
    }

    /// Ends a button's "working" state; a note shows briefly, an alert until dismissed.
    pub fn answer(callback: &str, note: &str, alert: bool) -> Call {
        Call {
            method: "answerCallbackQuery",
            body: format!(
                "{{\"callback_query_id\":{},\"text\":{},\"show_alert\":{alert}}}",
                json(callback),
                json(note)
            ),
        }
    }
}

// ---- What Telegram says ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum What {
    /// A message typed in the chat.
    Text(String),
    /// A button pressed under one of the bot's messages.
    Press {
        callback: String,
        message: i64,
        data: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub chat: i64,
    /// A one-to-one chat, not a group or a channel.
    pub private: bool,
    pub what: What,
}

/// One answer to a poll.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Batch {
    /// The newest update's number, whatever its kind: the next poll starts after it.
    pub last: Option<i64>,
    pub updates: Vec<Update>,
}

pub enum Polled {
    Batch(Batch),
    /// Telegram answered, and said no: its reason. A wrong token, mostly.
    Refused(String),
    Unreachable,
}

fn parse_poll(text: &str) -> Polled {
    if jsonc::get_raw(text, &["ok"]).as_deref() != Some("true") {
        return match jsonc::get_string(text, &["description"]) {
            Some(why) => Polled::Refused(why),
            None => Polled::Unreachable,
        };
    }
    let mut batch = Batch::default();
    for open in jsonc::objects_in(text, &["result"]) {
        let raw = |path: &[&str]| jsonc::get_raw_at(text, open, path);
        let string = |path: &[&str]| jsonc::get_string_at(text, open, path);
        let number = |path: &[&str]| raw(path).and_then(|n| n.parse::<i64>().ok());
        if let Some(id) = number(&["update_id"]) {
            batch.last = Some(batch.last.map_or(id, |l| l.max(id)));
        }
        const TYPED: &[&str] = &["message", "chat"];
        const PRESSED: &[&str] = &["callback_query", "message", "chat"];
        let (chat_at, what) = if let Some(text) = string(&["message", "text"]) {
            (TYPED, What::Text(text))
        } else if let (Some(callback), Some(data), Some(message)) = (
            string(&["callback_query", "id"]),
            string(&["callback_query", "data"]),
            number(&["callback_query", "message", "message_id"]),
        ) {
            let press = What::Press {
                callback,
                message,
                data,
            };
            (PRESSED, press)
        } else {
            continue;
        };
        let Some(chat) = number(&[chat_at, &["id"]].concat()) else {
            continue;
        };
        let private = string(&[chat_at, &["type"]].concat()).as_deref() == Some("private");
        batch.updates.push(Update {
            chat,
            private,
            what,
        });
    }
    Polled::Batch(batch)
}

// ---- The token and the paired chat -------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub token: Option<String>,
    /// The one chat that is obeyed.
    pub chat: Option<i64>,
}

/// How far setting up has come.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    NoToken,
    Unpaired,
    Paired,
}

/// A token as BotFather writes them: digits, a colon, letters. It ends up in an address,
/// so nothing else is let through.
fn valid_token(t: &str) -> bool {
    let Some((id, secret)) = t.split_once(':') else {
        return false;
    };
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    t.len() <= 100
        && !id.is_empty()
        && id.bytes().all(|b| b.is_ascii_digit())
        && !secret.is_empty()
        && secret.bytes().all(word)
}

fn setting(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('#') {
        return None;
    }
    let (k, v) = line.split_once('=')?;
    Some((k.trim(), v.trim()))
}

fn parse_config(text: &str) -> Config {
    let mut config = Config::default();
    for (key, value) in text.lines().filter_map(setting) {
        match key {
            "token" => config.token = valid_token(value).then(|| value.to_string()),
            "chat" => config.chat = value.parse().ok(),
            _ => {}
        }
    }
    config
}

/// The file's text with the chat set, everything else as it was.
fn with_chat(text: &str, chat: i64) -> String {
    let mut found = false;
    let mut out = String::new();
    for line in text.lines() {
        if !found && setting(line).is_some_and(|(k, _)| k == "chat") {
            found = true;
            out.push_str(&format!("chat = {chat}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !found {
        out.push_str(&format!("chat = {chat}\n"));
    }
    out
}

// ---- The use case ------------------------------------------------------------------------

#[derive(Clone)]
pub struct Telegram {
    ws: Arc<dyn Workstation>,
    web: Arc<dyn WebPost>,
    preferences: Preferences,
    enabled: Arc<AtomicBool>,
    /// Held while a chat is being paired.
    pairing: Arc<Mutex<()>>,
}

impl Telegram {
    pub fn new(ws: Arc<dyn Workstation>, web: Arc<dyn WebPost>, preferences: Preferences) -> Self {
        let enabled = preferences.load_telegram_remote();
        Telegram {
            ws,
            web,
            preferences,
            enabled: Arc::new(AtomicBool::new(enabled)),
            pairing: Arc::new(Mutex::new(())),
        }
    }

    /// Switches the remote on, if it is not.
    pub fn enable(&self) {
        if !self.enabled() {
            self.toggle();
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// The menu's switch; while off, Telegram is never called.
    pub fn toggle(&self) {
        let on = !self.enabled();
        self.enabled.store(on, Ordering::Relaxed);
        self.preferences.save_telegram_remote(on);
    }

    fn path(&self) -> PathBuf {
        self.ws.app_dir().join("telegram.txt")
    }

    pub fn config(&self) -> Config {
        self.ws
            .read_text(&self.path())
            .map_or(Config::default(), |t| parse_config(&t))
    }

    pub fn link(&self) -> Link {
        match self.config() {
            Config { token: None, .. } => Link::NoToken,
            Config { chat: None, .. } => Link::Unpaired,
            _ => Link::Paired,
        }
    }

    /// The file the token is pasted into, made with its instructions when missing.
    pub fn setup_file(&self) -> PathBuf {
        let path = self.path();
        if !self.ws.exists(&path) {
            let _ = self.ws.write_text(&path, TEMPLATE);
        }
        path
    }

    /// One request; the answer's text, or `None` when Telegram could not be reached.
    /// Slow: off the UI thread.
    pub fn call(&self, token: &str, call: &Call, max_seconds: u32) -> Option<String> {
        let url = format!("{API}/bot{token}/{}", call.method);
        self.web.post_json(&url, &call.body, max_seconds)
    }

    /// Waits up to `wait` seconds for what was sent to the bot after update `offset`.
    /// An offset of -1 asks for the newest update alone and forgets the ones before it.
    pub fn poll(&self, token: &str, offset: i64, wait: u32) -> Polled {
        let call = Call {
            method: "getUpdates",
            body: format!(
                "{{\"offset\":{offset},\"timeout\":{wait},\
                 \"allowed_updates\":[\"message\",\"callback_query\"]}}"
            ),
        };
        match self.call(token, &call, wait + 15) {
            Some(text) => parse_poll(&text),
            None => Polled::Unreachable,
        }
    }

    /// Tells Telegram the commands, so that typing "/" lists them.
    pub fn announce_commands(&self, token: &str) {
        let list: Vec<String> = COMMANDS
            .iter()
            .map(|(name, what)| {
                format!(
                    "{{\"command\":{},\"description\":{}}}",
                    json(name),
                    json(what)
                )
            })
            .collect();
        let call = Call {
            method: "setMyCommands",
            body: format!("{{\"commands\":[{}]}}", list.join(",")),
        };
        self.call(token, &call, 20);
    }

    /// Pairs with whoever writes to the bot first: while no chat is paired, a message
    /// typed in a one-to-one chat makes that chat the one obeyed from now on. True when
    /// this update did that. Once paired nobody else can, until the chat is emptied
    /// from the file.
    pub fn pairs(&self, update: &Update) -> bool {
        let typed = matches!(update.what, What::Text(_));
        if !typed || !update.private || self.link() != Link::Unpaired {
            return false;
        }
        // One at a time: two first messages cannot both win.
        let _one = self.pairing.lock().unwrap_or_else(|e| e.into_inner());
        if self.link() != Link::Unpaired {
            return false;
        }
        let path = self.path();
        let text = self.ws.read_text(&path).unwrap_or_default();
        self.ws
            .write_text(&path, &with_chat(&text, update.chat))
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_digits_a_colon_and_letters() {
        assert!(valid_token("123456:ABC-def_123"));
        assert!(!valid_token("123456"));
        assert!(!valid_token(":abc"));
        assert!(!valid_token("12a:abc"));
        assert!(!valid_token("123:abc/../x"));
        assert!(!valid_token("123:abc def"));
        assert!(!valid_token(""));
    }

    #[test]
    fn the_file_is_read_past_its_instructions() {
        assert_eq!(parse_config(TEMPLATE), Config::default());
        let text = TEMPLATE.replace("token =", "token = 42:abc");
        let config = parse_config(&text);
        assert_eq!(config.token.as_deref(), Some("42:abc"));
        assert_eq!(config.chat, None);
        // A token that is not one is no token.
        assert_eq!(parse_config("token = \"42:abc\"").token, None);
    }

    #[test]
    fn pairing_fills_in_the_chat_and_keeps_the_rest() {
        let text = TEMPLATE.replace("token =", "token = 42:abc");
        let paired = with_chat(&text, -7);
        // The line itself, not the instructions that mention it.
        assert_eq!(paired, text.replace("\nchat =\n", "\nchat = -7\n"));
        assert_eq!(parse_config(&paired).chat, Some(-7));
        let again = with_chat(&paired, 9);
        assert_eq!(parse_config(&again).chat, Some(9));
        assert_eq!(with_chat("token = 1:a", 5), "token = 1:a\nchat = 5\n");
    }

    #[test]
    fn messages_and_presses_are_read_from_a_poll() {
        let text = r#"{"ok":true,"result":[
            {"update_id":10,"message":{"message_id":1,"from":{"id":5,"is_bot":false},
              "chat":{"id":5,"first_name":"A \"B\"","type":"private"},"date":1,"text":"/awake on",
              "entities":[{"offset":0,"length":6,"type":"bot_command"}]}},
            {"update_id":11,"edited_message":{"message_id":1,"chat":{"id":5,"type":"private"}}},
            {"update_id":12,"callback_query":{"id":"77","from":{"id":5},
              "message":{"message_id":3,"chat":{"id":-100,"type":"group"},"text":"x"},
              "data":"a:6:1:keep-awake"}}
        ]}"#;
        let Polled::Batch(batch) = parse_poll(text) else {
            panic!("not a batch");
        };
        assert_eq!(batch.last, Some(12));
        assert_eq!(
            batch.updates,
            vec![
                Update {
                    chat: 5,
                    private: true,
                    what: What::Text("/awake on".into()),
                },
                Update {
                    chat: -100,
                    private: false,
                    what: What::Press {
                        callback: "77".into(),
                        message: 3,
                        data: "a:6:1:keep-awake".into(),
                    },
                },
            ]
        );
    }

    #[test]
    fn an_empty_poll_and_a_refusal_are_told_apart() {
        assert!(matches!(
            parse_poll(r#"{"ok":true,"result":[]}"#),
            Polled::Batch(b) if b == Batch::default()
        ));
        assert!(matches!(
            parse_poll(r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#),
            Polled::Refused(why) if why == "Unauthorized"
        ));
        assert!(matches!(parse_poll("<html>"), Polled::Unreachable));
    }

    #[test]
    fn what_is_sent_is_json() {
        let keys = vec![vec![Button::new("\u{2705} \"A\"", "a::0:x")]];
        assert_eq!(
            Call::send(5, "one\ntwo", Some(&keys)).body,
            "{\"chat_id\":5,\"text\":\"one\\ntwo\",\"reply_markup\":{\"inline_keyboard\":\
             [[{\"text\":\"\u{2705} \\\"A\\\"\",\"callback_data\":\"a::0:x\"}]]}}"
        );
        assert_eq!(
            Call::answer("77", "", false).body,
            "{\"callback_query_id\":\"77\",\"text\":\"\",\"show_alert\":false}"
        );
    }
}
