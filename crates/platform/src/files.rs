//! What survives between runs, as small text files in the per-user config folder. On
//! Windows the brains stay at %APPDATA%\IdleWalker\brains.txt, so an existing cast keeps
//! its memories, and choices not yet in the file fall back to the C# version's registry
//! values.
use idlewalker_application::{BrainStore, PreferenceStore};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

// ---- Menu choices: settings.txt -------------------------------------------------------

/// `name=value` lines, read once and rewritten whole on every change.
pub struct FilePreferences {
    path: Option<PathBuf>,
    values: Mutex<Option<BTreeMap<String, i32>>>,
}

impl FilePreferences {
    /// settings.txt in the config folder.
    pub fn new() -> Self {
        FilePreferences {
            path: crate::config_dir().map(|d| d.join("settings.txt")),
            values: Mutex::new(None),
        }
    }

    fn with_values<R>(&self, f: impl FnOnce(&mut BTreeMap<String, i32>) -> R) -> R {
        let mut guard = self.values.lock().unwrap();
        let map = guard.get_or_insert_with(|| {
            let mut m = BTreeMap::new();
            if let Some(text) = self
                .path
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p).ok())
            {
                for line in text.lines() {
                    if let Some((k, v)) = line.split_once('=') {
                        if let Ok(v) = v.trim().parse() {
                            m.insert(k.trim().to_string(), v);
                        }
                    }
                }
            }
            m
        });
        f(map)
    }
}

impl PreferenceStore for FilePreferences {
    fn load(&self, name: &str) -> Option<i32> {
        if let Some(v) = self.with_values(|m| m.get(name).copied()) {
            return Some(v);
        }
        crate::legacy_setting(name)
    }

    fn save(&self, name: &str, value: i32) {
        let text = self.with_values(|m| {
            m.insert(name.to_string(), value);
            m.iter()
                .map(|(k, v)| format!("{}={}\n", k, v))
                .collect::<String>()
        });
        if let Some(path) = &self.path {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let tmp = path.with_extension("tmp");
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
        }
    }
}

// ---- What the walkers learned: brains.txt ---------------------------------------------

pub struct FileBrainStore {
    path: Option<PathBuf>,
    /// One write at a time: the background save and the exit save can overlap.
    writing: Mutex<()>,
}

impl FileBrainStore {
    /// brains.txt in the config folder.
    pub fn new() -> Self {
        Self::at_path(crate::config_dir().map(|d| d.join("brains.txt")))
    }

    /// A fixed file, for tests.
    pub fn at(path: PathBuf) -> Self {
        Self::at_path(Some(path))
    }

    fn at_path(path: Option<PathBuf>) -> Self {
        FileBrainStore {
            path,
            writing: Mutex::new(()),
        }
    }
}

impl BrainStore for FileBrainStore {
    fn read(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
    }

    /// Atomic: a crash can never leave a half-written file behind.
    fn write(&self, text: &str) {
        let _guard = self.writing.lock().unwrap();
        let Some(path) = &self.path else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = PathBuf::from(format!("{}.tmp", path.display()));
        if std::fs::write(&tmp, text).is_err() {
            return;
        }
        if path.exists() {
            let _ = std::fs::copy(path, format!("{}.bak", path.display()));
        }
        if std::fs::rename(&tmp, path).is_err() {
            let _ = std::fs::copy(&tmp, path);
            let _ = std::fs::remove_file(&tmp);
        }
    }

    fn delete(&self) {
        if let Some(p) = &self.path {
            let _ = std::fs::remove_file(p);
        }
    }

    fn size(&self) -> i64 {
        self.path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map_or(-1, |m| m.len() as i64)
    }

    fn location(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}
