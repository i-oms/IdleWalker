//! What the walkers learned, kept between runs: the text format, reading it back into
//! the cast, and when the work happens. The bytes go through the `BrainStore` port.
use crate::ports::BrainStore;
use idlewalker_domain::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

const HEADER: &str = "# IdleWalker brains v4"; // v4: 81 situations, sparse table
const OLD_HEADERS: [&str; 3] = [
    "# IdleWalker brains v1",
    "# IdleWalker brains v2",
    "# IdleWalker brains v3",
];

#[derive(Clone)]
pub struct Brains(Arc<dyn BrainStore>);

impl Brains {
    pub fn new(store: Arc<dyn BrainStore>) -> Self {
        Brains(store)
    }

    /// Where the brains live, to show them in the file manager.
    pub fn location(&self) -> Option<PathBuf> {
        self.0.location()
    }

    /// How big the saved brains are in bytes; -1 when nothing is saved yet.
    pub fn size(&self) -> i64 {
        self.0.size()
    }

    /// Forgets everything: clears the in-memory minds and deletes what was saved.
    pub fn forget(&self, troupe: Troupe) {
        troupe.forget_brains();
        self.0.delete();
    }

    /// Loads lasting memories; nothing saved, or something corrupt, just means a fresh cast.
    pub fn load(&self, troupe: Troupe) {
        if let Some(text) = self.0.read() {
            unpack(troupe, &text);
        }
    }

    /// Saves on this thread; for exit, when the write must have finished.
    pub fn save(&self, troupe: Troupe) {
        self.0.write(&format_minds(copy_minds(troupe)));
    }

    /// Copies the memories here and does the formatting and the writing on another
    /// thread, so the once-a-minute save never holds a frame.
    pub fn save_in_background(&self, troupe: Troupe) {
        let minds = copy_minds(troupe);
        let store = self.0.clone();
        std::thread::spawn(move || store.write(&format_minds(minds)));
    }
}

fn unpack(troupe: Troupe, text: &str) {
    let text = text.trim_start_matches('\u{feff}');
    let first = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if first.starts_with('#') && first != HEADER && !OLD_HEADERS.contains(&first) {
        return; // a foreign file
    }
    rt::attempt(|| {
        troupe.ensure_cast();
        let by_name: Dict<String, Walker> = Dict::new();
        for w in troupe.walkers().items() {
            by_name.set(w.personality.name.clone(), w);
        }
        let mut current: Option<(Walker, Dict<String, String>)> = None;
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                if let Some((w, f)) = current.take() {
                    w.unpack_mind(f, by_name.clone());
                }
                let name = line[1..line.len() - 1].to_string();
                if by_name.contains_key(&name) {
                    current = Some((by_name.get(&name), Dict::new()));
                }
                continue;
            }
            if let (Some((_, f)), Some((k, v))) = (&current, line.split_once('=')) {
                if !k.is_empty() {
                    f.set(k.to_string(), v.to_string());
                }
            }
        }
        if let Some((w, f)) = current.take() {
            w.unpack_mind(f, by_name.clone());
        }
    });
}

fn copy_minds(troupe: Troupe) -> Vec<(String, MindCopy)> {
    troupe
        .walkers()
        .items()
        .filter_map(|w| rt::attempt_val(|| (w.personality.name.clone(), w.copy_mind())))
        .collect()
}

fn format_minds(minds: Vec<(String, MindCopy)>) -> String {
    let mut out = format!("{}\n", HEADER);
    for (name, mind) in minds {
        if let Some(pack) = rt::attempt_val(|| mind.pack()) {
            out += &format!("[{}]\n{}", name, pack);
        }
        mind.free();
    }
    out
}
