//! The terminal prompt: Oh My Posh, set up from a window instead of a dozen terminal
//! commands. Finds or installs it and keeps it current; fetches its themes; puts a theme
//! into every shell on the machine (PowerShell 7, Windows PowerShell, bash, zsh, fish,
//! cmd through Clink, nu), so it shows in every terminal that runs them, VS Code's
//! included; installs the Nerd Font its glyphs need; and tells every terminal it knows,
//! profile by profile, to use that font. What it writes into a shell profile sits between
//! two marker lines, so it can be replaced or taken out again without touching anything
//! else. What it changes in a terminal's settings is written down first, with the value
//! that was there, so "put back" and "uninstall" leave the machine as it was found.
//! Everything it does to the machine goes through the `Workstation` port, and all of it
//! is slow: run it off the UI thread.
use crate::jsonc;
use crate::ports::{Os, RunResult, Workstation};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const RELEASES: &str = "https://github.com/JanDeDobbeleer/oh-my-posh/releases";
pub const THEME_GALLERY: &str = "https://ohmyposh.dev/docs/themes";
const THEMES_ZIP: &str = "https://cdn.ohmyposh.dev/releases/latest/themes.zip";
const INSTALL_SH: &str = "https://ohmyposh.dev/install.sh";
const INSTALL_PS1: &str = "https://ohmyposh.dev/install.ps1";
pub const FONT_NAME: &str = "MesloLGM Nerd Font";
/// The same font with every icon squeezed to one cell: what the terminals that cannot
/// cope with wide glyphs (Terminal.app, iTerm2, the Windows console) are given.
pub const FONT_MONO: &str = "MesloLGM Nerd Font Mono";
/// How macOS names that font internally: its PostScript name. Nerd Fonts shorten
/// "Nerd Font Mono" to "NFM" there (and "Nerd Font" to "NF") since version 3.
const FONT_MONO_POSTSCRIPT: &str = "MesloLGMNFM-Regular";
/// The names Terminal is offered for that font, in turn, until it reports one back:
/// the PostScript name, the one Nerd Fonts used before version 3, the family name and
/// the full name, then the same for the font with the wide icons. The names macOS
/// itself resolves on the machine come before these.
const APPLE_TERMINAL_FONT_NAMES: [&str; 8] = [
    FONT_MONO_POSTSCRIPT,
    "MesloLGMNerdFontMono-Regular",
    FONT_MONO,
    "MesloLGM Nerd Font Mono Regular",
    "MesloLGMNF-Regular",
    "MesloLGMNerdFont-Regular",
    FONT_NAME,
    "MesloLGM Nerd Font Regular",
];
const FONT_PACKAGE: &str = "meslo";
const WINGET_ID: &str = "JanDeDobbeleer.OhMyPosh";
const CLINK_WINGET_ID: &str = "chrisant996.Clink";
const BEGIN: &str = ">>> IdleWalker: Oh My Posh >>>";
const END: &str = "<<< IdleWalker: Oh My Posh <<<";
/// The theme line's word for Oh My Posh's built-in default.
pub const DEFAULT_THEME: &str = "(default)";
/// `oh-my-posh init nu` writes its script into Nushell's vendor autoload folder, which
/// Nushell has had since this version.
const NU_MIN_VERSION: (u32, u32, u32) = (0, 104, 0);
/// The file it writes there.
const NU_AUTOLOAD_FILE: &str = "oh-my-posh.nu";
/// Where the terminal settings this program changed are kept, with their old values.
const UNDO_FILE: &str = "terminal-undo.txt";
/// Where the last task's every program run, with what it printed, is written for
/// anyone trying to see why a terminal did not take the font.
const DEBUG_FILE: &str = "terminal-debug.txt";
/// How much of what a program printed the debug file keeps, per stream.
const DEBUG_KEEP: usize = 6000;
/// VS Code's terminal font, and the contrast rule that recolours a prompt's segments
/// unless it is turned off (1).
const VSCODE_FONT: &str = "terminal.integrated.fontFamily";
const VSCODE_CONTRAST: &str = "terminal.integrated.minimumContrastRatio";
/// Windows Terminal's font for every profile, the key older versions used for it, and
/// the colour rule that breaks Powerline separators unless it is "never".
const WT_FACE: [&str; 4] = ["profiles", "defaults", "font", "face"];
const WT_LEGACY_FACE: [&str; 3] = ["profiles", "defaults", "fontFace"];
const WT_COLOURS: [&str; 3] = ["profiles", "defaults", "adjustIndistinguishableColors"];
const WT_LIST: [&str; 2] = ["profiles", "list"];

// ---- What the window shows ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shell {
    Pwsh,
    WindowsPowerShell,
    Bash,
    Zsh,
    Fish,
    Cmd,
    Nu,
}

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Shell::Pwsh => "PowerShell 7",
            Shell::WindowsPowerShell => "Windows PowerShell",
            Shell::Bash => "Bash",
            Shell::Zsh => "Zsh",
            Shell::Fish => "Fish",
            Shell::Cmd => "Command Prompt",
            Shell::Nu => "Nushell",
        }
    }

    /// How its profile starts a comment line.
    fn comment(self) -> &'static str {
        if self == Shell::Cmd {
            "--"
        } else {
            "#"
        }
    }

    /// What `oh-my-posh init` calls it.
    fn init_name(self) -> &'static str {
        match self {
            Shell::Pwsh | Shell::WindowsPowerShell => "pwsh",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Cmd => "cmd",
            Shell::Nu => "nu",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShellStatus {
    pub shell: Shell,
    /// The file the prompt line goes in.
    pub profile: PathBuf,
    /// The prompt line is in it.
    pub configured: bool,
    /// The theme named in it, `DEFAULT_THEME` for none.
    pub theme: Option<String>,
    /// Why it cannot be set up yet.
    pub blocker: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub file: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalKind {
    VsCode,
    WindowsTerminal,
    /// The classic console window cmd and PowerShell open in outside Windows Terminal.
    ConsoleHost,
    GnomeTerminal,
    Tilix,
    Konsole,
    XfceTerminal,
    AppleTerminal,
    ITerm2,
    Kitty,
    Alacritty,
    Tabby,
    MateTerminal,
    /// A terminal with one font setting in gsettings (Ptyxis, GNOME Console, Guake,
    /// elementary Terminal): what to set is in `GS_RECIPES`.
    Gsettings,
    /// A terminal with one settings file of a known shape (INI, `Key=Value`, TOML or
    /// JSON): what to set is in `RECIPES`, by the terminal's name.
    File,
    /// Hyper's `.hyper.js`.
    Hyper,
    /// ConEmu, and Cmder which bundles it: `ConEmu.xml`, or the registry.
    ConEmu,
    /// PuTTY: its sessions in the registry on Windows, in `~/.putty/sessions` elsewhere.
    Putty,
    Terminator,
    /// xterm and urxvt, through `~/.Xresources`.
    XResources,
    /// A terminal that carries the Nerd Font icons itself (WezTerm, Ghostty): nothing to set.
    BuiltIn,
}

#[derive(Clone, Debug)]
pub struct TerminalStatus {
    pub name: String,
    pub kind: TerminalKind,
    /// The settings file, the registry key, or the profile the terminal calls its default.
    pub target: String,
    /// Every profile looked at uses the Nerd Font, and nothing else stands in the way.
    pub font_set: bool,
    /// What was looked at: "3 profiles", or what still needs doing.
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
pub struct Status {
    pub binary: Option<PathBuf>,
    pub version: Option<String>,
    /// The newest release, once a check has run.
    pub latest: Option<String>,
    pub themes_dir: Option<PathBuf>,
    pub themes: Vec<Theme>,
    pub shells: Vec<ShellStatus>,
    pub font_installed: bool,
    pub terminals: Vec<TerminalStatus>,
    /// Terminal settings this program changed are on record and can be put back.
    pub undo_available: bool,
    /// Windows only: Clink, which gives cmd its prompt.
    pub clink: Option<PathBuf>,
    /// How an install or update would be done.
    pub installer: &'static str,
    /// What this OS's terminals need to know about the font and colours.
    pub terminal_note: &'static str,
}

impl Status {
    pub fn update_available(&self) -> bool {
        match (&self.latest, &self.version) {
            (Some(l), Some(v)) => is_newer(l, v),
            _ => false,
        }
    }
}

/// One thing the window can ask for.
#[derive(Clone, Debug)]
pub enum Task {
    Refresh,
    CheckLatest,
    Install,
    Update,
    DownloadThemes,
    Apply {
        /// `None` is the built-in default theme.
        theme: Option<String>,
        shells: Vec<Shell>,
    },
    Remove,
    /// The prompt out of every shell, the terminals as they were, then Oh My Posh off
    /// the machine.
    Uninstall,
    InstallFont,
    SetTerminalFonts,
    /// Every terminal setting back to what it was before `SetTerminalFonts`.
    RestoreTerminalFonts,
    InstallClink,
}

/// What a task did, and the state after it.
#[derive(Clone, Debug)]
pub struct Done {
    pub ok: bool,
    pub log: Vec<String>,
    pub status: Status,
}

/// One terminal setting as it was before this program changed it, so it can be put
/// back. The target names the program and the place: `vscode:<file>`, `wt:<file>`,
/// `console:<registry key>`, `gsettings:<schema:path>`, `konsole:<file>`,
/// `konsole-file:<file>` (one this program made), `xfce:<file>`, `kitty:<file>`,
/// `alacritty:<file>`, `terminal.app:<profile>`, `iterm2:<profile guid>`, and
/// `font-file:<path>` for a font file this program installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Undo {
    pub target: String,
    pub key: String,
    /// The value before, as that program writes it; `None` when there was none.
    pub before: Option<String>,
}

// ---- Pure helpers ------------------------------------------------------------------------

pub fn parse_version(text: &str) -> Option<(u32, u32, u32)> {
    let text = text.trim().trim_start_matches('v');
    let core = text.split('-').next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u32>().ok());
    let v = (parts.next()??, parts.next()??, parts.next()??);
    Some(v)
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

fn theme_name(file: &Path) -> Option<String> {
    let f = file.file_name()?.to_str()?;
    [".omp.json", ".omp.yaml", ".omp.yml", ".omp.toml"]
        .iter()
        .find_map(|ext| f.strip_suffix(ext).map(str::to_string))
}

/// Whether a font name is a Nerd Font's: "MesloLGM Nerd Font", "MesloLGM NF",
/// "MesloLGMNerdFontMono-Regular", "MesloLGMNFM-Regular" and the like.
fn is_nerd_font(name: &str) -> bool {
    let s = name.to_lowercase().replace(' ', "");
    if s.contains("nerdfont") {
        return true;
    }
    // The short family name, with or without a style after a hyphen or a space.
    let family = s.split(['-', ':']).next().unwrap_or("");
    let family = family.trim_end_matches("regular").trim_end_matches("bold");
    ["nf", "nfm", "nfp"]
        .iter()
        .any(|tail| family.ends_with(tail) && family.len() > tail.len())
}

/// Where the managed block sits: from the start of its first line to after its last.
fn block_range(text: &str, comment: &str) -> Option<(usize, usize)> {
    let begin = format!("{comment} {BEGIN}");
    let end = format!("{comment} {END}");
    let b = text.find(&begin)?;
    let e = text[b..].find(&end)? + b + end.len();
    let e = text[e..].find('\n').map_or(text.len(), |n| e + n + 1);
    Some((b, e))
}

/// The text with the managed block holding `body`: replaced where there is one, added at
/// the end otherwise.
pub fn with_block(text: &str, comment: &str, body: &str) -> String {
    let block = format!("{comment} {BEGIN}\n{body}\n{comment} {END}\n");
    match block_range(text, comment) {
        Some((b, e)) => format!("{}{}{}", &text[..b], block, &text[e..]),
        None => {
            let mut out = text.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&block);
            out
        }
    }
}

/// The text without the managed block; `None` when there was none.
pub fn without_block(text: &str, comment: &str) -> Option<String> {
    let (b, e) = block_range(text, comment)?;
    // The blank line that was put in front of it goes too.
    let b = if text[..b].ends_with("\n\n") {
        b - 1
    } else {
        b
    };
    Some(format!("{}{}", &text[..b], &text[e..]))
}

/// What is between the markers.
pub fn block_body<'a>(text: &'a str, comment: &str) -> Option<&'a str> {
    let (b, e) = block_range(text, comment)?;
    let inner = &text[b..e];
    let start = inner.find('\n')? + 1;
    let end = inner.rfind(&format!("{comment} {END}"))?;
    Some(inner[start..end].strip_suffix('\n').unwrap_or(""))
}

/// The theme named on the block's first line.
fn theme_in(body: &str, comment: &str) -> Option<String> {
    let first = body.lines().next()?;
    let rest = first.strip_prefix(comment)?.trim();
    rest.strip_prefix("theme:").map(|t| t.trim().to_string())
}

fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn dq(s: &str) -> String {
    format!("\"{s}\"")
}

fn lua_quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// A path as the shell wants to see it: the Unix-born shells on Windows take forward
/// slashes, and so does Lua, where a backslash would need escaping.
fn path_text(p: &Path, os: Os, shell: Shell) -> String {
    let s = p.display().to_string();
    match (os, shell) {
        (Os::Windows, Shell::Pwsh | Shell::WindowsPowerShell) => s,
        (Os::Windows, _) => s.replace('\\', "/"),
        _ => s,
    }
}

/// The line that starts the prompt in a shell's profile, as the Oh My Posh documentation
/// gives it for each shell, with the program and the theme as full paths so it works
/// whatever the shell's PATH is.
pub fn init_line(shell: Shell, os: Os, binary: &Path, theme: Option<&Path>) -> String {
    let b = path_text(binary, os, shell);
    let t = theme.map(|t| path_text(t, os, shell));
    let config = |quote: fn(&str) -> String| {
        t.as_deref()
            .map(|t| format!(" --config {}", quote(t)))
            .unwrap_or_default()
    };
    match shell {
        Shell::Pwsh | Shell::WindowsPowerShell => {
            format!(
                "& {} init pwsh{} | Invoke-Expression",
                ps_quote(&b),
                config(ps_quote)
            )
        }
        Shell::Bash | Shell::Zsh => format!(
            "eval \"$({} init {}{})\"",
            sh_quote(&b),
            shell.init_name(),
            config(sh_quote)
        ),
        Shell::Fish => format!("{} init fish{} | source", sh_quote(&b), config(sh_quote)),
        // Prints nothing: it writes its script into Nushell's autoload folder, which
        // Nushell reads right after this file.
        Shell::Nu => format!("^{} init nu{}", dq(&b), config(dq)),
        Shell::Cmd => {
            // io.popen hands the line to cmd, which drops the outer quotes of a line
            // that starts with one: a quoted program path is wrapped in another pair.
            let needs_quotes = b.contains(' ');
            let program = if needs_quotes { dq(&b) } else { b.clone() };
            let mut line = format!("{program} init cmd{}", config(dq));
            if needs_quotes {
                line = dq(&line);
            }
            format!("load(io.popen('{}'):read(\"*a\"))()", lua_quote(&line))
        }
    }
}

/// The last few lines a program printed, for the log.
/// What a program printed, cut to what the debug file keeps.
fn debug_clip(text: &str) -> String {
    let text = text.trim_end();
    if text.len() <= DEBUG_KEEP {
        return text.to_string();
    }
    let mut end = DEBUG_KEEP;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[... {} more bytes]", &text[..end], text.len() - end)
}

fn tail(r: &RunResult) -> Vec<String> {
    let text = if r.stderr.trim().is_empty() {
        &r.stdout
    } else {
        &r.stderr
    };
    let lines: Vec<String> = text
        .split(['\n', '\r'])
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.chars().all(|c| "-\\|/ █▒░".contains(c)))
        .map(|l| format!("    {l}"))
        .collect();
    lines[lines.len().saturating_sub(3)..].to_vec()
}

// ---- The record of what was changed ----------------------------------------------------

fn escape_field(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape_field(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => {}
        }
    }
    out
}

/// The record as a file: one line per setting, tab-separated, the old value after `=`
/// or `-` for none.
pub fn undo_text(list: &[Undo]) -> String {
    let mut out = String::from(
        "# Terminal settings as they were before IdleWalker set the Nerd Font. Do not edit.\n",
    );
    for u in list {
        let before = match &u.before {
            Some(v) => format!("={}", escape_field(v)),
            None => "-".to_string(),
        };
        out.push_str(&format!(
            "{}\t{}\t{before}\n",
            escape_field(&u.target),
            escape_field(&u.key)
        ));
    }
    out
}

pub fn undo_parse(text: &str) -> Vec<Undo> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let mut f = l.splitn(3, '\t');
            let target = unescape_field(f.next()?);
            let key = unescape_field(f.next()?);
            let before = match f.next()? {
                "-" => None,
                v => Some(unescape_field(v.strip_prefix('=')?)),
            };
            Some(Undo {
                target,
                key,
                before,
            })
        })
        .collect()
}

/// The record while settings are being changed: the first value seen for a setting is
/// the one kept, so setting the font twice still remembers what was there at first.
struct Changes {
    list: Vec<Undo>,
}

impl Changes {
    fn remember(&mut self, target: &str, key: &str, before: Option<String>) {
        if !self.list.iter().any(|u| u.target == target && u.key == key) {
            self.list.push(Undo {
                target: target.into(),
                key: key.into(),
                before,
            });
        }
    }
}

// ---- The use case ----------------------------------------------------------------------

#[derive(Clone)]
pub struct Posh {
    ws: Arc<dyn Workstation>,
    /// Every program run since the last task began, for the debug file.
    trace: Arc<Mutex<Vec<String>>>,
}

impl Posh {
    pub fn new(ws: Arc<dyn Workstation>) -> Self {
        Posh {
            ws,
            trace: Arc::default(),
        }
    }

    /// A line for the debug file.
    fn note(&self, line: impl Into<String>) {
        self.trace.lock().unwrap().push(line.into());
    }

    fn debug_path(&self) -> PathBuf {
        self.ws.app_dir().join(DEBUG_FILE)
    }

    /// Writes the trace of a task to the debug file and empties it.
    fn debug_save(&self, task: &str) {
        let lines = std::mem::take(&mut *self.trace.lock().unwrap());
        let mut text = format!(
            "IdleWalker terminal prompt: {task}\nOS: {:?}\nHome: {}\n\n",
            self.os(),
            self.home().display()
        );
        text.push_str(&lines.join("\n"));
        text.push('\n');
        let _ = self.ws.write_text(&self.debug_path(), &text);
    }

    fn os(&self) -> Os {
        self.ws.os()
    }

    fn home(&self) -> PathBuf {
        self.ws.home_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    fn env_dir(&self, name: &str) -> Option<PathBuf> {
        self.ws
            .env(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }

    fn local_app_data(&self) -> PathBuf {
        self.env_dir("LOCALAPPDATA")
            .unwrap_or_else(|| self.home().join("AppData").join("Local"))
    }

    fn app_data(&self) -> PathBuf {
        self.env_dir("APPDATA")
            .unwrap_or_else(|| self.home().join("AppData").join("Roaming"))
    }

    fn config_home(&self) -> PathBuf {
        self.env_dir("XDG_CONFIG_HOME")
            .unwrap_or_else(|| self.home().join(".config"))
    }

    fn data_home(&self) -> PathBuf {
        self.env_dir("XDG_DATA_HOME")
            .unwrap_or_else(|| self.home().join(".local").join("share"))
    }

    fn program_files(&self) -> PathBuf {
        self.env_dir("ProgramFiles")
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
    }

    fn program_files_x86(&self) -> PathBuf {
        self.env_dir("ProgramFiles(x86)")
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
    }

    fn windir(&self) -> PathBuf {
        self.env_dir("SystemRoot")
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
    }

    fn first_existing(&self, candidates: Vec<PathBuf>) -> Option<PathBuf> {
        candidates.into_iter().find(|p| self.ws.exists(p))
    }

    /// A program by its known homes first, then the PATH.
    fn program(&self, name: &str, known: Vec<PathBuf>) -> Option<PathBuf> {
        self.first_existing(known)
            .or_else(|| self.ws.find_program(name))
    }

    /// Whether a macOS program of that exact name is running.
    fn mac_running(&self, name: &str) -> bool {
        self.ws
            .find_program("pgrep")
            .map(|p| self.run(&p, &["-x", name]).success)
            .unwrap_or(false)
    }

    /// Whether the window has set Terminal's font before: the record holds its
    /// profiles, so the Automation question has been answered already.
    fn apple_terminal_applied(&self) -> bool {
        self.undo_load()
            .iter()
            .any(|u| u.target.starts_with("terminal.app:"))
    }

    /// A macOS application, in either Applications folder.
    fn mac_app(&self, name: &str) -> bool {
        self.ws.exists(&PathBuf::from("/Applications").join(name))
            || self
                .ws
                .exists(&self.home().join("Applications").join(name))
    }

    fn run(&self, program: &Path, args: &[&str]) -> RunResult {
        let p = program.display().to_string();
        let r = self.ws.run(&p, args);
        let shown: Vec<String> = args
            .iter()
            .map(|a| {
                if a.contains(' ') || a.contains('\n') {
                    format!("'{}'", a.replace('\n', "\n    "))
                } else {
                    a.to_string()
                }
            })
            .collect();
        let mut line = format!(
            "$ {p} {}\n  exit: {}",
            shown.join(" "),
            if r.success { "ok" } else { "failed" }
        );
        for (name, text) in [("stdout", &r.stdout), ("stderr", &r.stderr)] {
            if !text.trim().is_empty() {
                line.push_str(&format!("\n  {name}:\n    {}", debug_clip(text).replace('\n', "\n    ")));
            }
        }
        self.note(line);
        r
    }

    /// The last line a program printed, if it succeeded.
    fn ask(&self, program: &Path, args: &[&str]) -> Option<String> {
        let r = self.run(program, args);
        if !r.success {
            return None;
        }
        r.stdout
            .lines()
            .map(str::trim)
            .rfind(|l| !l.is_empty())
            .map(str::to_string)
    }

    /// Everything a program printed, if it succeeded.
    fn ask_all(&self, program: &Path, args: &[&str]) -> Option<String> {
        let r = self.run(program, args);
        r.success.then_some(r.stdout)
    }

    /// A PowerShell to run a script with: PowerShell 7 if there is one, else Windows'.
    fn powershell(&self) -> Option<PathBuf> {
        self.shell_programs().into_iter().find_map(|(s, p)| {
            (s == Shell::WindowsPowerShell || s == Shell::Pwsh)
                .then_some(p)
                .flatten()
        })
    }

    // ---- Oh My Posh itself ----

    pub fn find_binary(&self) -> Option<PathBuf> {
        let home = self.home();
        let known = match self.os() {
            Os::Windows => vec![
                self.local_app_data()
                    .join("Microsoft")
                    .join("WindowsApps")
                    .join("oh-my-posh.exe"),
                self.local_app_data()
                    .join("Programs")
                    .join("oh-my-posh")
                    .join("bin")
                    .join("oh-my-posh.exe"),
                home.join("scoop").join("shims").join("oh-my-posh.exe"),
                PathBuf::from(r"C:\ProgramData\chocolatey\bin\oh-my-posh.exe"),
            ],
            Os::MacOs => vec![
                PathBuf::from("/opt/homebrew/bin/oh-my-posh"),
                PathBuf::from("/usr/local/bin/oh-my-posh"),
                PathBuf::from("/opt/local/bin/oh-my-posh"),
                home.join(".local").join("bin").join("oh-my-posh"),
                home.join("bin").join("oh-my-posh"),
            ],
            Os::Linux => vec![
                home.join(".local").join("bin").join("oh-my-posh"),
                home.join("bin").join("oh-my-posh"),
                PathBuf::from("/usr/local/bin/oh-my-posh"),
                PathBuf::from("/home/linuxbrew/.linuxbrew/bin/oh-my-posh"),
                PathBuf::from("/usr/bin/oh-my-posh"),
            ],
        };
        self.ws
            .find_program("oh-my-posh")
            .or_else(|| self.first_existing(known))
    }

    fn version_of(&self, binary: &Path) -> Option<String> {
        let v = self.ask(binary, &["version"])?;
        parse_version(&v).map(|_| v.trim_start_matches('v').to_string())
    }

    /// The newest release's version. Slow, and needs the network.
    pub fn latest(&self) -> Option<String> {
        let address = self.ws.final_address(&format!("{RELEASES}/latest"))?;
        let tag = address
            .trim()
            .strip_prefix(RELEASES)?
            .strip_prefix("/tag/")?;
        parse_version(tag).map(|_| tag.trim_start_matches('v').to_string())
    }

    /// Where Oh My Posh keeps its cache and the init scripts it writes; the themes go
    /// beside them, as its own install script puts them.
    fn cache_dir(&self, binary: &Path) -> PathBuf {
        self.ask(binary, &["cache", "path"])
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| self.home().join(".cache").join("oh-my-posh"))
    }

    fn themes_in(&self, dir: &Path) -> Vec<Theme> {
        let mut themes: Vec<Theme> = self
            .ws
            .list_files(dir, 1)
            .into_iter()
            .filter_map(|file| theme_name(&file).map(|name| Theme { name, file }))
            .collect();
        themes.sort_by_key(|t| t.name.to_lowercase());
        themes.dedup_by(|a, b| a.name == b.name);
        themes
    }

    fn find_themes_dir(&self, binary: Option<&Path>) -> Option<PathBuf> {
        let home = self.home();
        let mut c = Vec::new();
        if let Some(p) = self.env_dir("POSH_THEMES_PATH") {
            c.push(p);
        }
        if let Some(b) = binary {
            if let Some(root) = b.parent().and_then(Path::parent) {
                c.push(root.join("themes"));
            }
            c.push(self.cache_dir(b).join("themes"));
        }
        match self.os() {
            Os::Windows => {
                c.push(
                    self.local_app_data()
                        .join("Programs")
                        .join("oh-my-posh")
                        .join("themes"),
                );
                c.push(
                    home.join("scoop")
                        .join("apps")
                        .join("oh-my-posh")
                        .join("current")
                        .join("themes"),
                );
            }
            Os::MacOs => {
                c.push(PathBuf::from("/opt/homebrew/opt/oh-my-posh/themes"));
                c.push(PathBuf::from("/usr/local/opt/oh-my-posh/themes"));
                c.push(PathBuf::from("/opt/local/share/oh-my-posh/themes"));
            }
            Os::Linux => {
                c.push(PathBuf::from(
                    "/home/linuxbrew/.linuxbrew/opt/oh-my-posh/themes",
                ));
            }
        }
        c.push(home.join(".cache").join("oh-my-posh").join("themes"));
        c.push(home.join(".poshthemes"));
        c.into_iter().find(|d| !self.themes_in(d).is_empty())
    }

    fn installer(&self) -> &'static str {
        match self.os() {
            Os::Windows if self.ws.find_program("winget").is_some() => "winget",
            Os::Windows => "the Oh My Posh install script (PowerShell)",
            Os::MacOs if self.ws.find_program("brew").is_some() => "Homebrew",
            _ => "the Oh My Posh install script",
        }
    }

    // ---- The shells ----

    /// Every shell this OS can have, with where it was found.
    fn shell_programs(&self) -> Vec<(Shell, Option<PathBuf>)> {
        let home = self.home();
        let lad = self.local_app_data();
        match self.os() {
            Os::Windows => vec![
                (
                    Shell::Pwsh,
                    self.program(
                        "pwsh",
                        vec![
                            self.program_files()
                                .join("PowerShell")
                                .join("7")
                                .join("pwsh.exe"),
                            lad.join("Microsoft").join("WindowsApps").join("pwsh.exe"),
                        ],
                    ),
                ),
                (
                    Shell::WindowsPowerShell,
                    self.program(
                        "powershell",
                        vec![self
                            .windir()
                            .join("System32")
                            .join("WindowsPowerShell")
                            .join("v1.0")
                            .join("powershell.exe")],
                    ),
                ),
                (
                    Shell::Bash,
                    // Git Bash; the bash.exe in System32 is WSL's launcher.
                    self.first_existing(vec![
                        self.program_files()
                            .join("Git")
                            .join("bin")
                            .join("bash.exe"),
                        self.program_files_x86()
                            .join("Git")
                            .join("bin")
                            .join("bash.exe"),
                        lad.join("Programs")
                            .join("Git")
                            .join("bin")
                            .join("bash.exe"),
                    ])
                    .or_else(|| {
                        self.ws.find_program("bash").filter(|p| {
                            !p.display().to_string().to_lowercase().contains("system32")
                        })
                    }),
                ),
                (
                    Shell::Cmd,
                    Some(self.windir().join("System32").join("cmd.exe")),
                ),
                (
                    Shell::Nu,
                    self.program(
                        "nu",
                        vec![
                            lad.join("Programs").join("nu").join("bin").join("nu.exe"),
                            home.join(".cargo").join("bin").join("nu.exe"),
                            home.join("scoop").join("shims").join("nu.exe"),
                        ],
                    ),
                ),
            ],
            Os::MacOs => vec![
                (Shell::Zsh, self.program("zsh", vec!["/bin/zsh".into()])),
                (
                    Shell::Bash,
                    self.program(
                        "bash",
                        vec!["/opt/homebrew/bin/bash".into(), "/bin/bash".into()],
                    ),
                ),
                (
                    Shell::Fish,
                    self.program(
                        "fish",
                        vec![
                            "/opt/homebrew/bin/fish".into(),
                            "/usr/local/bin/fish".into(),
                        ],
                    ),
                ),
                (
                    Shell::Pwsh,
                    self.program(
                        "pwsh",
                        vec![
                            "/opt/homebrew/bin/pwsh".into(),
                            "/usr/local/bin/pwsh".into(),
                        ],
                    ),
                ),
                (
                    Shell::Nu,
                    self.program(
                        "nu",
                        vec![
                            "/opt/homebrew/bin/nu".into(),
                            home.join(".cargo").join("bin").join("nu"),
                        ],
                    ),
                ),
            ],
            Os::Linux => vec![
                (
                    Shell::Bash,
                    self.program("bash", vec!["/bin/bash".into(), "/usr/bin/bash".into()]),
                ),
                (
                    Shell::Zsh,
                    self.program("zsh", vec!["/usr/bin/zsh".into(), "/bin/zsh".into()]),
                ),
                (
                    Shell::Fish,
                    self.program("fish", vec!["/usr/bin/fish".into()]),
                ),
                (
                    Shell::Pwsh,
                    self.program(
                        "pwsh",
                        vec![
                            "/usr/bin/pwsh".into(),
                            "/opt/microsoft/powershell/7/pwsh".into(),
                            "/snap/bin/pwsh".into(),
                        ],
                    ),
                ),
                (
                    Shell::Nu,
                    self.program(
                        "nu",
                        vec![
                            home.join(".cargo").join("bin").join("nu"),
                            "/usr/bin/nu".into(),
                        ],
                    ),
                ),
            ],
        }
    }

    /// The file the prompt line goes in. The PowerShells and nu are asked, because their
    /// answer follows a redirected Documents folder or a custom config location. For the
    /// PowerShells it is the profile every host reads (the console, VS Code, the ISE).
    fn profile_of(&self, shell: Shell, program: &Path) -> PathBuf {
        let home = self.home();
        let asked = |args: &[&str]| {
            self.ask(program, args)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
        };
        match shell {
            Shell::Pwsh => asked(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$PROFILE.CurrentUserAllHosts",
            ])
            .unwrap_or_else(|| {
                if self.os() == Os::Windows {
                    home.join("Documents")
                        .join("PowerShell")
                        .join("profile.ps1")
                } else {
                    self.config_home().join("powershell").join("profile.ps1")
                }
            }),
            Shell::WindowsPowerShell => asked(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$PROFILE.CurrentUserAllHosts",
            ])
            .unwrap_or_else(|| {
                home.join("Documents")
                    .join("WindowsPowerShell")
                    .join("profile.ps1")
            }),
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => self.env_dir("ZDOTDIR").unwrap_or(home).join(".zshrc"),
            Shell::Fish => self.config_home().join("fish").join("config.fish"),
            // Clink's profile folder, one of the places it loads every .lua from.
            Shell::Cmd => self.local_app_data().join("clink").join("oh-my-posh.lua"),
            Shell::Nu => asked(&["-c", "$nu.config-path"]).unwrap_or_else(|| {
                self.nu_config_dir(&home).join("config.nu")
            }),
        }
    }

    fn nu_config_dir(&self, home: &Path) -> PathBuf {
        match self.os() {
            Os::Windows => self.app_data().join("nushell"),
            Os::MacOs => home
                .join("Library")
                .join("Application Support")
                .join("nushell"),
            Os::Linux => self.config_home().join("nushell"),
        }
    }

    /// Where `oh-my-posh init nu` writes its script: Nushell's vendor autoload folder.
    fn nu_autoload_dir(&self, nu: &Path) -> PathBuf {
        self.ask(nu, &["-c", "$nu.data-dir | path join vendor autoload"])
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| {
                let data = match self.os() {
                    Os::Windows => self.app_data().join("nushell"),
                    Os::MacOs => self.nu_config_dir(&self.home()),
                    Os::Linux => self.data_home().join("nushell"),
                };
                data.join("vendor").join("autoload")
            })
    }

    /// Why Nushell cannot take the prompt: too old for the autoload folder.
    fn nu_blocker(&self, nu: &Path) -> Option<String> {
        let v = self.ask(nu, &["--version"])?;
        let parsed = parse_version(&v)?;
        (parsed < NU_MIN_VERSION).then(|| {
            format!(
                "needs Nushell {}.{}.{} or newer, this is {v}",
                NU_MIN_VERSION.0, NU_MIN_VERSION.1, NU_MIN_VERSION.2
            )
        })
    }

    fn find_clink(&self) -> Option<PathBuf> {
        self.program(
            "clink",
            vec![
                self.program_files_x86().join("clink").join("clink_x64.exe"),
                self.program_files().join("clink").join("clink_x64.exe"),
                self.local_app_data()
                    .join("Programs")
                    .join("clink")
                    .join("clink_x64.exe"),
                self.local_app_data().join("clink").join("clink_x64.exe"),
            ],
        )
    }

    fn shell_status(&self, shell: Shell, program: &Path, clink: Option<&Path>) -> ShellStatus {
        let profile = self.profile_of(shell, program);
        let text = self.ws.read_text(&profile).unwrap_or_default();
        let body = block_body(&text, shell.comment());
        let blocker = match shell {
            Shell::Cmd if clink.is_none() => Some("needs Clink: install it below".to_string()),
            Shell::Nu => self.nu_blocker(program),
            _ => None,
        };
        ShellStatus {
            shell,
            profile,
            configured: body.is_some(),
            theme: body.and_then(|b| theme_in(b, shell.comment())),
            blocker,
        }
    }

    fn shells(&self, clink: Option<&Path>) -> Vec<ShellStatus> {
        self.shell_programs()
            .into_iter()
            .filter_map(|(shell, program)| program.map(|p| self.shell_status(shell, &p, clink)))
            .collect()
    }

    // ---- The font ----

    /// Where fonts go for one person, which is where `oh-my-posh font install` puts them
    /// when it is not run as an administrator.
    fn user_font_dir(&self) -> PathBuf {
        match self.os() {
            Os::Windows => self
                .local_app_data()
                .join("Microsoft")
                .join("Windows")
                .join("Fonts"),
            Os::MacOs => self.home().join("Library").join("Fonts"),
            Os::Linux => self.data_home().join("fonts"),
        }
    }

    fn font_installed(&self) -> bool {
        let home = self.home();
        let mut dirs = vec![self.user_font_dir()];
        dirs.extend(match self.os() {
            Os::Windows => vec![self.windir().join("Fonts")],
            Os::MacOs => vec![PathBuf::from("/Library/Fonts")],
            Os::Linux => vec![
                home.join(".fonts"),
                PathBuf::from("/usr/share/fonts"),
                PathBuf::from("/usr/local/share/fonts"),
            ],
        });
        dirs.iter().any(|d| {
            self.ws.list_files(d, 4).iter().any(|f| {
                let name = f
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase().replace(' ', ""))
                    .unwrap_or_default();
                is_nerd_font(&name)
            })
        })
    }

    /// The font families Windows actually reports, filtered to ones mentioning Meslo
    /// or a Nerd Font. Windows Terminal rejects any face that is not one of these,
    /// so the face written into settings must match what Windows registered.
    /// Nerd Fonts 3+ register the family as "MesloLGM NF", not "MesloLGM Nerd Font".
    /// Style suffixes (" Regular", " Bold", ...) are stripped so callers get a clean
    /// family name that both Windows Terminal and VS Code accept.
    fn windows_font_families(&self) -> Vec<String> {
        let Some(ps) = self.powershell() else {
            return Vec::new();
        };
        let script = "try { Add-Type -AssemblyName System.Drawing -ErrorAction Stop; \
                      [System.Drawing.FontFamily]::Families | Where-Object { \
                      $_.Name -like '*Meslo*' -or $_.Name -like '*Nerd Font*' } | \
                      ForEach-Object { $_.Name } } catch { }";
        let r = self.run(
            &ps,
            &["-NoProfile", "-NonInteractive", "-Command", script],
        );
        if !r.success {
            self.note(format!("font-family probe: powershell did not succeed"));
            return Vec::new();
        }
        let mut families: Vec<String> = r
            .stdout
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| {
                let s = l.to_string();
                // Strip a trailing style suffix so callers get a clean family name.
                for suffix in [" Regular", " Bold", " Italic", " Bold Italic"] {
                    if let Some(stripped) = s.strip_suffix(suffix) {
                        return stripped.to_string();
                    }
                }
                s
            })
            .collect();
        families.dedup();
        families
    }

    /// The family name Windows reports for the MesloLGM Nerd Font, tried in order,
    /// falling back to `FONT_NAME` (or `FONT_MONO` when `mono`) when nothing matches.
    fn windows_font_name(&self, mono: bool) -> String {
        let fallback = if mono { FONT_MONO } else { FONT_NAME };
        let families = self.windows_font_families();
        if families.is_empty() {
            return fallback.to_string();
        }
        let wanted: &[&str] = if mono {
            &[
                "MesloLGM NF Mono",
                "MesloLGM Nerd Font Mono",
                "MesloLGM NFM",
                "MesloLGM NF Mono Regular",
            ]
        } else {
            &[
                "MesloLGM NF",
                "MesloLGM Nerd Font",
                "MesloLGM Nerd Font Regular",
            ]
        };
        for w in wanted {
            if families.iter().any(|f| f.eq_ignore_ascii_case(w)) {
                return w.to_string();
            }
        }
        families
            .into_iter()
            .find(|f| {
                let s = f.to_lowercase().replace(' ', "");
                s.contains("meslo") && s.contains("nerdfont")
            })
            .unwrap_or_else(|| fallback.to_string())
    }

    // ---- The terminals: what each one has ----

    fn terminals(&self) -> Vec<TerminalStatus> {
        let mut out = Vec::new();
        self.vscode_terminals(&mut out);
        match self.os() {
            Os::Windows => {
                self.windows_terminals(&mut out);
                out.push(self.console_terminal());
                self.windows_more_terminals(&mut out);
            }
            Os::MacOs => self.mac_terminals(&mut out),
            Os::Linux => self.linux_terminals(&mut out),
        }
        self.shared_terminals(&mut out);
        out
    }

    fn vscode_terminals(&self, out: &mut Vec<TerminalStatus>) {
        let home = self.home();
        let editors = [
            ("Code", "VS Code"),
            ("Code - Insiders", "VS Code Insiders"),
            ("Code - OSS", "Code - OSS"),
            ("VSCodium", "VSCodium"),
            ("Cursor", "Cursor"),
            ("Windsurf", "Windsurf"),
        ];
        for (folder, name) in editors {
            let user = match self.os() {
                Os::Windows => self.app_data().join(folder).join("User"),
                Os::MacOs => home
                    .join("Library")
                    .join("Application Support")
                    .join(folder)
                    .join("User"),
                Os::Linux => self.config_home().join(folder).join("User"),
            };
            if !self.ws.exists(&user) {
                continue;
            }
            let settings = user.join("settings.json");
            let text = self.ws.read_text(&settings).unwrap_or_default();
            let font = jsonc::get_string(&text, &[VSCODE_FONT]).is_some_and(|f| is_nerd_font(&f));
            let contrast =
                jsonc::get_raw(&text, &[VSCODE_CONTRAST]).is_some_and(|v| v.trim() == "1");
            out.push(TerminalStatus {
                name: name.into(),
                kind: TerminalKind::VsCode,
                target: settings.display().to_string(),
                font_set: font && contrast,
                detail: if font && !contrast {
                    "colours still adjusted".into()
                } else {
                    String::new()
                },
            });
        }
    }

    fn windows_terminals(&self, out: &mut Vec<TerminalStatus>) {
        let packages = self.local_app_data().join("Packages");
        let store = |package: &str| {
            packages
                .join(format!("{package}_8wekyb3d8bbwe"))
                .join("LocalState")
                .join("settings.json")
        };
        let candidates = [
            ("Windows Terminal", store("Microsoft.WindowsTerminal")),
            (
                "Windows Terminal Preview",
                store("Microsoft.WindowsTerminalPreview"),
            ),
            (
                "Windows Terminal Canary",
                store("Microsoft.WindowsTerminalCanary"),
            ),
            (
                "Windows Terminal (unpackaged)",
                self.local_app_data()
                    .join("Microsoft")
                    .join("Windows Terminal")
                    .join("settings.json"),
            ),
        ];
        for (name, settings) in candidates {
            if !self.ws.exists(&settings) {
                continue;
            }
            let text = self.ws.read_text(&settings).unwrap_or_default();
            let (font_set, profiles) = wt_status(&text);
            out.push(TerminalStatus {
                name: name.into(),
                kind: TerminalKind::WindowsTerminal,
                target: settings.display().to_string(),
                font_set,
                detail: profiles_text(profiles),
            });
        }
    }

    /// Every console window starts from HKCU\Console; a program or window title with
    /// its own subkey there takes that subkey's font instead.
    fn console_terminal(&self) -> TerminalStatus {
        let keys = self.console_keys();
        let with_face: Vec<&RegKey> = keys
            .iter()
            .filter(|k| reg_value(k, "FaceName").is_some())
            .collect();
        let root_set = keys
            .first()
            .and_then(|k| reg_value(k, "FaceName"))
            .is_some_and(|(_, face)| is_nerd_font(face));
        let all = root_set
            && with_face
                .iter()
                .all(|k| reg_value(k, "FaceName").is_some_and(|(_, f)| is_nerd_font(f)));
        TerminalStatus {
            name: "Console windows (cmd, PowerShell)".into(),
            kind: TerminalKind::ConsoleHost,
            target: r"HKCU\Console".into(),
            font_set: all,
            detail: profiles_text(with_face.len().max(1)),
        }
    }

    /// HKCU\Console and every subkey under it, the root first.
    fn console_keys(&self) -> Vec<RegKey> {
        let Some(reg) = self.ws.find_program("reg") else {
            return Vec::new();
        };
        let text = self
            .ask_all(&reg, &["query", r"HKCU\Console", "/s"])
            .unwrap_or_default();
        reg_parse(&text)
    }

    fn mac_terminals(&self, out: &mut Vec<TerminalStatus>) {
        let home = self.home();
        let defaults = self.ws.find_program("defaults");
        let default_profile = defaults
            .as_deref()
            .and_then(|d| {
                self.ask(
                    d,
                    &["read", "com.apple.Terminal", "Default Window Settings"],
                )
            })
            .unwrap_or_else(|| "Basic".into());
        let plist = defaults
            .as_deref()
            .and_then(|d| self.ask_all(d, &["export", "com.apple.Terminal", "-"]))
            .unwrap_or_default();
        let mut names = plist_profile_names(&plist);
        if names.is_empty() {
            names.push(default_profile.clone());
        }
        let mut font_set = names
            .iter()
            .all(|n| apple_terminal_font_is_nerd(&plist, n));
        // Terminal keeps a change made through AppleScript in its own copy of the
        // settings while it runs, and writes the file later. So once the font has
        // been applied, and Terminal is running, Terminal itself is asked. Not
        // before: asking would start Terminal and raise the Automation question.
        if !font_set && self.apple_terminal_applied() && self.mac_running("Terminal") {
            if let Some(osascript) = self.ws.find_program("osascript") {
                let live = self.apple_terminal_fonts(&osascript);
                if !live.is_empty() {
                    names = live.iter().map(|(n, _)| n.clone()).collect();
                    font_set = live.iter().all(|(_, f)| is_nerd_font(f));
                }
            }
        }
        out.push(TerminalStatus {
            name: "Terminal".into(),
            kind: TerminalKind::AppleTerminal,
            target: default_profile,
            font_set,
            detail: profiles_text(names.len()),
        });
        let iterm = home
            .join("Library")
            .join("Preferences")
            .join("com.googlecode.iterm2.plist");
        if self.ws.exists(&iterm) || self.mac_app("iTerm.app") {
            let plist = defaults
                .as_deref()
                .and_then(|d| self.ask_all(d, &["export", "com.googlecode.iterm2", "-"]))
                .unwrap_or_default();
            let fonts = iterm_fonts(&plist);
            out.push(TerminalStatus {
                name: "iTerm2".into(),
                kind: TerminalKind::ITerm2,
                target: iterm.display().to_string(),
                font_set: !fonts.is_empty() && fonts.iter().all(|f| is_nerd_font(f)),
                detail: profiles_text(fonts.len()),
            });
        }
    }

    fn linux_terminals(&self, out: &mut Vec<TerminalStatus>) {
        if let Some(gsettings) = self.ws.find_program("gsettings") {
            for (name, kind, list) in [
                ("GNOME Terminal", TerminalKind::GnomeTerminal, GNOME),
                ("Tilix", TerminalKind::Tilix, TILIX),
                ("MATE Terminal", TerminalKind::MateTerminal, MATE),
            ] {
                let profiles = self.gsettings_profiles(&gsettings, list);
                if profiles.is_empty() {
                    continue;
                }
                let all = profiles.iter().all(|p| {
                    let system = self.ask(&gsettings, &["get", p, "use-system-font"]);
                    let font = self.ask(&gsettings, &["get", p, "font"]);
                    system.as_deref() == Some("false") && font.is_some_and(|f| is_nerd_font(&f))
                });
                out.push(TerminalStatus {
                    name: name.into(),
                    kind,
                    target: list.0.into(),
                    font_set: all,
                    detail: profiles_text(profiles.len()),
                });
            }
        }
        let konsole_dir = self.konsole_dir();
        if self.ws.exists(&self.konsolerc())
            || self.ws.exists(&konsole_dir)
            || self.ws.find_program("konsole").is_some()
        {
            let profiles = self.konsole_profiles();
            let all = !profiles.is_empty()
                && profiles.iter().all(|p| {
                    let text = self.ws.read_text(p).unwrap_or_default();
                    ini_get(&text, "Appearance", "Font").is_some_and(|f| is_nerd_font(&f))
                });
            out.push(TerminalStatus {
                name: "Konsole".into(),
                kind: TerminalKind::Konsole,
                target: konsole_dir.display().to_string(),
                font_set: all,
                detail: profiles_text(profiles.len().max(1)),
            });
        }
        let xfce = self.xfce_rc();
        if self.ws.exists(&xfce) || self.ws.find_program("xfce4-terminal").is_some() {
            let text = self.ws.read_text(&xfce).unwrap_or_default();
            let font = ini_get(&text, "Configuration", "FontName")
                .is_some_and(|f| is_nerd_font(&f));
            let own = ini_get(&text, "Configuration", "FontUseSystem")
                .is_none_or(|v| !v.eq_ignore_ascii_case("true"));
            out.push(TerminalStatus {
                name: "Xfce Terminal".into(),
                kind: TerminalKind::XfceTerminal,
                target: xfce.display().to_string(),
                font_set: font && own,
                detail: String::new(),
            });
        }
        self.linux_more_terminals(out);
    }

    /// The terminals that run on more than one OS and keep plain settings files.
    fn shared_terminals(&self, out: &mut Vec<TerminalStatus>) {
        let os = self.os();
        if os != Os::Windows {
            let conf = self.config_home().join("kitty").join("kitty.conf");
            let present = self.ws.exists(&conf)
                || self.ws.find_program("kitty").is_some()
                || (os == Os::MacOs && self.mac_app("kitty.app"));
            if present {
                let text = self.ws.read_text(&conf).unwrap_or_default();
                out.push(TerminalStatus {
                    name: "kitty".into(),
                    kind: TerminalKind::Kitty,
                    target: conf.display().to_string(),
                    font_set: conf_get(&text, "font_family").is_some_and(|f| is_nerd_font(&f)),
                    detail: String::new(),
                });
            }
        }
        let alacritty = self.alacritty_config();
        let present = self.ws.exists(&alacritty)
            || self.ws.find_program("alacritty").is_some()
            || (os == Os::MacOs && self.mac_app("Alacritty.app"));
        if present {
            let text = self.ws.read_text(&alacritty).unwrap_or_default();
            out.push(TerminalStatus {
                name: "Alacritty".into(),
                kind: TerminalKind::Alacritty,
                target: alacritty.display().to_string(),
                font_set: alacritty_font(&text).is_some_and(|f| is_nerd_font(&f)),
                detail: String::new(),
            });
        }
        let tabby = self.tabby_config();
        let present = self.ws.exists(&tabby)
            || match os {
                Os::Windows => self
                    .first_existing(vec![
                        self.local_app_data().join("Programs").join("Tabby").join("Tabby.exe"),
                        self.program_files().join("Tabby").join("Tabby.exe"),
                    ])
                    .is_some(),
                Os::MacOs => self.mac_app("Tabby.app"),
                Os::Linux => self.ws.find_program("tabby").is_some(),
            };
        if present {
            let text = self.ws.read_text(&tabby).unwrap_or_default();
            out.push(TerminalStatus {
                name: "Tabby".into(),
                kind: TerminalKind::Tabby,
                target: tabby.display().to_string(),
                font_set: yaml_get(&text, "terminal", "font")
                    .is_some_and(|f| is_nerd_font(&yaml_unquote(&f))),
                detail: String::new(),
            });
        }
        let wezterm = match os {
            Os::Windows => {
                self.ws
                    .exists(&self.program_files().join("WezTerm").join("wezterm-gui.exe"))
                    || self.ws.find_program("wezterm").is_some()
            }
            Os::MacOs => self.mac_app("WezTerm.app") || self.ws.find_program("wezterm").is_some(),
            Os::Linux => self.ws.find_program("wezterm").is_some(),
        };
        if wezterm {
            out.push(built_in("WezTerm"));
        }
        let ghostty = match os {
            Os::Windows => false,
            Os::MacOs => self.mac_app("Ghostty.app") || self.ws.find_program("ghostty").is_some(),
            Os::Linux => {
                self.ws.find_program("ghostty").is_some()
                    || self.ws.exists(&self.config_home().join("ghostty"))
            }
        };
        if ghostty {
            out.push(built_in("Ghostty"));
        }
        self.shared_more_terminals(out);
    }

    // ---- More terminals ----

    /// Terminals only found on Windows: ConEmu and Cmder, PuTTY, and mintty as Git
    /// Bash, MSYS2, Cygwin and wsltty ship it.
    fn windows_more_terminals(&self, out: &mut Vec<TerminalStatus>) {
        // ConEmu reads the first settings file it finds, else the registry.
        let mut xmls: Vec<(&str, PathBuf)> = Vec::new();
        let conemu_dir = self
            .ws
            .find_program("ConEmu64")
            .or_else(|| self.ws.find_program("ConEmu"))
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.program_files().join("ConEmu"));
        let own = vec![
            conemu_dir.join("ConEmu.xml"),
            self.app_data().join("ConEmu.xml"),
            self.app_data().join(".ConEmu.xml"),
        ];
        if let Some(file) = self.first_existing(own) {
            xmls.push(("ConEmu", file));
        }
        let cmder_root = self.env_dir("CMDER_ROOT").or_else(|| {
            self.ws
                .find_program("Cmder")
                .and_then(|p| p.parent().map(Path::to_path_buf))
        });
        if let Some(root) = cmder_root {
            let user = root.join("config").join("user-ConEmu.xml");
            let vendor = root
                .join("vendor")
                .join("conemu-maximus5")
                .join("ConEmu.xml");
            if self.ws.exists(&user) {
                xmls.push(("Cmder", user));
            } else if self.ws.exists(&vendor) {
                xmls.push(("Cmder", vendor));
            }
        }
        for (name, file) in xmls {
            let text = self.ws.read_text(&file).unwrap_or_default();
            let fonts = conemu_fonts(&text);
            out.push(TerminalStatus {
                name: name.into(),
                kind: TerminalKind::ConEmu,
                target: file.display().to_string(),
                font_set: !fonts.is_empty()
                    && fonts
                        .iter()
                        .all(|(_, f)| f.as_deref().is_some_and(is_nerd_font)),
                detail: profiles_text(fonts.len().max(1)),
            });
        }
        if let Some(reg) = self.ws.find_program("reg") {
            let configs = self.reg_children(&reg, CONEMU_KEY);
            if !configs.is_empty() {
                out.push(TerminalStatus {
                    name: "ConEmu (registry)".into(),
                    kind: TerminalKind::ConEmu,
                    target: CONEMU_KEY.into(),
                    font_set: configs
                        .iter()
                        .all(|k| reg_value(k, "FontName").is_some_and(|(_, f)| is_nerd_font(f))),
                    detail: profiles_text(configs.len()),
                });
            }
            let sessions = self.reg_children(&reg, PUTTY_SESSIONS);
            let installed = self.ws.find_program("putty").is_some()
                || self
                    .ws
                    .exists(&self.program_files().join("PuTTY").join("putty.exe"))
                || self
                    .ws
                    .exists(&self.program_files_x86().join("PuTTY").join("putty.exe"));
            if installed || !sessions.is_empty() {
                out.push(TerminalStatus {
                    name: "PuTTY".into(),
                    kind: TerminalKind::Putty,
                    target: PUTTY_SESSIONS.into(),
                    font_set: !sessions.is_empty()
                        && sessions
                            .iter()
                            .all(|k| reg_value(k, "Font").is_some_and(|(_, f)| is_nerd_font(f))),
                    detail: profiles_text(sessions.len().max(1)),
                });
            }
        }
        // mintty, once per environment that ships it: each reads its own home's .minttyrc,
        // which overrides every other file it looks at.
        let has_mintty = |dir: &PathBuf| self.ws.exists(&dir.join("usr").join("bin").join("mintty.exe"));
        let git_dir = self
            .ws
            .find_program("git")
            .and_then(|p| p.parent()?.parent().map(Path::to_path_buf))
            .filter(has_mintty)
            .or_else(|| {
                [
                    self.program_files().join("Git"),
                    self.local_app_data().join("Programs").join("Git"),
                ]
                .into_iter()
                .find(has_mintty)
            });
        if git_dir.is_some() {
            let home = self.env_dir("HOME").unwrap_or_else(|| self.home());
            self.push_file(out, "Git Bash (mintty)", home.join(".minttyrc"), true);
        }
        let user = self
            .ws
            .env("USERNAME")
            .filter(|u| !u.is_empty())
            .or_else(|| self.home().file_name().map(|n| n.to_string_lossy().into_owned()));
        if let Some(user) = user {
            let roots: [(&str, &[&str], &[&str]); 2] = [
                ("MSYS2 (mintty)", &[r"C:\msys64", r"C:\msys32"], &["usr", "bin", "mintty.exe"]),
                ("Cygwin (mintty)", &[r"C:\cygwin64", r"C:\cygwin"], &["bin", "mintty.exe"]),
            ];
            for (name, candidates, bin) in roots {
                let found = candidates.iter().map(PathBuf::from).find(|root| {
                    let exe = bin.iter().fold(root.clone(), |p, part| p.join(part));
                    self.ws.exists(&exe)
                });
                if let Some(root) = found {
                    self.push_file(out, name, root.join("home").join(&user).join(".minttyrc"), true);
                }
            }
        }
        let wsltty = self
            .local_app_data()
            .join("wsltty")
            .join("bin")
            .join("mintty.exe");
        if self.ws.exists(&wsltty) {
            self.push_file(out, "wsltty", self.app_data().join("wsltty").join("config"), true);
        }
    }

    /// The keys right under a registry key, with their values.
    fn reg_children(&self, reg: &Path, base: &str) -> Vec<RegKey> {
        let text = self.ask_all(reg, &["query", base, "/s"]).unwrap_or_default();
        reg_parse(&text)
            .into_iter()
            .filter(|k| {
                k.path.len() > base.len() + 1
                    && k.path[..base.len()].eq_ignore_ascii_case(base)
                    && !k.path[base.len() + 1..].contains('\\')
            })
            .collect()
    }

    /// Terminals only found on Linux desktops.
    fn linux_more_terminals(&self, out: &mut Vec<TerminalStatus>) {
        if let Some(gsettings) = self.ws.find_program("gsettings") {
            for r in GS_RECIPES {
                let (schema, key, _) = r.settings[0];
                let present = self.ws.find_program(r.program).is_some()
                    || self.ask(&gsettings, &["get", schema, key]).is_some();
                if !present {
                    continue;
                }
                let all = r.settings.iter().all(|(schema, key, form)| {
                    self.ask(&gsettings, &["get", schema, key])
                        .is_some_and(|v| form.is_set(&v))
                });
                out.push(TerminalStatus {
                    name: r.name.into(),
                    kind: TerminalKind::Gsettings,
                    target: schema.into(),
                    font_set: all,
                    detail: String::new(),
                });
            }
        }
        let terminator = self.config_home().join("terminator").join("config");
        if self.ws.exists(&terminator) || self.ws.find_program("terminator").is_some() {
            let text = self.ws.read_text(&terminator).unwrap_or_default();
            let profiles = terminator_profiles(&text);
            let all = !profiles.is_empty()
                && profiles.iter().all(|p| {
                    terminator_get(&text, p, "font").is_some_and(|f| is_nerd_font(&f))
                        && terminator_get(&text, p, "use_system_font")
                            .is_some_and(|v| v.eq_ignore_ascii_case("false"))
                });
            out.push(TerminalStatus {
                name: "Terminator".into(),
                kind: TerminalKind::Terminator,
                target: terminator.display().to_string(),
                font_set: all,
                detail: profiles_text(profiles.len().max(1)),
            });
        }
        let config = self.config_home();
        let found = |p: &str| self.ws.find_program(p).is_some();
        self.push_file(
            out,
            "LXTerminal",
            config.join("lxterminal").join("lxterminal.conf"),
            found("lxterminal"),
        );
        self.push_file(
            out,
            "QTerminal",
            config.join("qterminal.org").join("qterminal.ini"),
            found("qterminal"),
        );
        self.push_file(out, "Sakura", config.join("sakura").join("sakura.conf"), found("sakura"));
        self.push_file(
            out,
            "Deepin Terminal",
            config.join("deepin").join("deepin-terminal").join("config.conf"),
            found("deepin-terminal"),
        );
        self.push_file(out, "foot", config.join("foot").join("foot.ini"), found("foot"));
        // xterm and urxvt read X resources from the file already there.
        let xres = self.xresources();
        for (name, key) in [("xterm", "faceName"), ("urxvt", "font")] {
            if !found(name) {
                continue;
            }
            let text = self.ws.read_text(&xres).unwrap_or_default();
            out.push(TerminalStatus {
                name: name.into(),
                kind: TerminalKind::XResources,
                target: xres.display().to_string(),
                font_set: xres_get(&text, xres_app(name), key).is_some_and(|v| is_nerd_font(&v)),
                detail: String::new(),
            });
        }
        // PuTTY keeps a file per session.
        let sessions = self.putty_dir();
        if self.ws.exists(&sessions) || found("putty") {
            let files = self.putty_session_files();
            let all = !files.is_empty()
                && files.iter().all(|f| {
                    kv_get(&self.ws.read_text(f).unwrap_or_default(), "Font")
                        .is_some_and(|v| is_nerd_font(&v))
                });
            out.push(TerminalStatus {
                name: "PuTTY".into(),
                kind: TerminalKind::Putty,
                target: sessions.display().to_string(),
                font_set: all,
                detail: profiles_text(files.len().max(1)),
            });
        }
    }

    /// `~/.Xresources`, or `~/.Xdefaults` when only that one is there.
    fn xresources(&self) -> PathBuf {
        let res = self.home().join(".Xresources");
        let defaults = self.home().join(".Xdefaults");
        if !self.ws.exists(&res) && self.ws.exists(&defaults) {
            defaults
        } else {
            res
        }
    }

    fn putty_dir(&self) -> PathBuf {
        self.home().join(".putty").join("sessions")
    }

    fn putty_session_files(&self) -> Vec<PathBuf> {
        let mut files = self.ws.list_files(&self.putty_dir(), 1);
        files.sort();
        files
    }

    /// Terminals that run on more than one OS, beyond those in `shared_terminals`.
    fn shared_more_terminals(&self, out: &mut Vec<TerminalStatus>) {
        let os = self.os();
        let (js, json) = self.hyper_configs();
        let hyper = self.ws.find_program("hyper").is_some()
            || match os {
                Os::Windows => self
                    .ws
                    .exists(&self.local_app_data().join("hyper").join("Hyper.exe")),
                Os::MacOs => self.mac_app("Hyper.app"),
                Os::Linux => false,
            };
        if self.ws.exists(&json) {
            self.push_file(out, "Hyper (hyper.json)", json, true);
        } else if self.ws.exists(&js) || hyper {
            let text = self.ws.read_text(&js).unwrap_or_default();
            out.push(TerminalStatus {
                name: "Hyper".into(),
                kind: TerminalKind::Hyper,
                target: js.display().to_string(),
                font_set: hyper_font(&text).is_some_and(|v| hyper_is_nerd(&v)),
                detail: String::new(),
            });
        }
        let rio = self
            .env_dir("RIO_CONFIG_HOME")
            .map(|d| d.join("config.toml"))
            .unwrap_or_else(|| match os {
                Os::Windows => self.local_app_data().join("rio").join("config.toml"),
                _ => self.config_home().join("rio").join("config.toml"),
            });
        self.push_file(
            out,
            "Rio",
            rio,
            self.ws.find_program("rio").is_some() || (os == Os::MacOs && self.mac_app("Rio.app")),
        );
        let warp = match os {
            Os::Windows => self
                .local_app_data()
                .join("warp")
                .join("Warp")
                .join("config")
                .join("settings.toml"),
            Os::MacOs => self.home().join(".warp").join("settings.toml"),
            Os::Linux => self
                .config_home()
                .join("warp-terminal")
                .join("settings.toml"),
        };
        let warp_installed = match os {
            Os::Windows => self.ws.exists(
                &self
                    .local_app_data()
                    .join("Programs")
                    .join("Warp")
                    .join("Warp.exe"),
            ),
            Os::MacOs => self.mac_app("Warp.app"),
            Os::Linux => self.ws.find_program("warp-terminal").is_some(),
        };
        self.push_file(out, "Warp", warp, warp_installed);
        let zed = match os {
            Os::Windows => self.app_data().join("Zed").join("settings.json"),
            _ => self.config_home().join("zed").join("settings.json"),
        };
        self.push_file(
            out,
            "Zed",
            zed,
            self.ws.find_program("zed").is_some() || (os == Os::MacOs && self.mac_app("Zed.app")),
        );
        let wave = self
            .home()
            .join(".config")
            .join("waveterm")
            .join("settings.json");
        let wave_installed = self.ws.find_program("wave").is_some()
            || self.ws.find_program("waveterm").is_some()
            || (os == Os::MacOs && self.mac_app("Wave.app"));
        self.push_file(out, "Wave", wave, wave_installed);
    }

    /// Hyper's settings files: `.hyper.js` up to version 3 and `hyper.json` after it.
    fn hyper_configs(&self) -> (PathBuf, PathBuf) {
        if let Some(xdg) = self.env_dir("XDG_CONFIG_HOME") {
            return (
                xdg.join("hyper").join(".hyper.js"),
                xdg.join("Hyper").join("hyper.json"),
            );
        }
        match self.os() {
            Os::Windows => {
                let d = self.app_data().join("Hyper");
                (d.join(".hyper.js"), d.join("hyper.json"))
            }
            _ => (
                self.home().join(".hyper.js"),
                self.home().join(".config").join("Hyper").join("hyper.json"),
            ),
        }
    }

    /// Lists a terminal with a settings file of a known shape when the file or the
    /// program is there.
    fn push_file(&self, out: &mut Vec<TerminalStatus>, name: &str, path: PathBuf, installed: bool) {
        if !installed && !self.ws.exists(&path) {
            return;
        }
        let Some(r) = recipe(name) else {
            return;
        };
        let text = self.ws.read_text(&path).unwrap_or_default();
        let text = if r.name == "foot" { foot_main(&text) } else { text };
        out.push(TerminalStatus {
            name: name.into(),
            kind: TerminalKind::File,
            target: path.display().to_string(),
            font_set: recipe_font_set(r, &text),
            detail: String::new(),
        });
    }

    /// The gsettings profile paths (`schema:/path/`) of a terminal with a profile list.
    fn gsettings_profiles(&self, gsettings: &Path, list: GsettingsList) -> Vec<String> {
        let Some(printed) = self.ask(gsettings, &["get", list.0, list.1]) else {
            return Vec::new();
        };
        let mut ids = gvariant_strings(&printed);
        if ids.is_empty() {
            if let Some(d) = self.ask(gsettings, &["get", list.0, list.2]) {
                ids = gvariant_strings(&d);
            }
        }
        ids.iter()
            .map(|id| format!("{}:{}{id}/", list.3, list.4))
            .collect()
    }

    fn konsolerc(&self) -> PathBuf {
        self.config_home().join("konsolerc")
    }

    fn konsole_dir(&self) -> PathBuf {
        self.data_home().join("konsole")
    }

    /// Konsole's profile files.
    fn konsole_profiles(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self
            .ws
            .list_files(&self.konsole_dir(), 1)
            .into_iter()
            .filter(|f| f.extension().is_some_and(|e| e == "profile"))
            .collect();
        files.sort();
        files
    }

    fn xfce_rc(&self) -> PathBuf {
        self.config_home()
            .join("xfce4")
            .join("terminal")
            .join("terminalrc")
    }

    /// Alacritty's settings file: the one that exists, else where it would be made.
    fn alacritty_config(&self) -> PathBuf {
        let candidates = match self.os() {
            Os::Windows => vec![self.app_data().join("alacritty").join("alacritty.toml")],
            _ => vec![
                self.config_home().join("alacritty").join("alacritty.toml"),
                self.config_home().join("alacritty.toml"),
                self.home().join(".alacritty.toml"),
            ],
        };
        let first = candidates[0].clone();
        self.first_existing(candidates).unwrap_or(first)
    }

    /// Tabby's settings file: `config.yaml` in the folder `TABBY_CONFIG_DIRECTORY` names,
    /// else in its user-data folder (a portable copy keeps it in `data` next to the
    /// program, which is not looked for).
    fn tabby_config(&self) -> PathBuf {
        if let Some(dir) = self.env_dir("TABBY_CONFIG_DIRECTORY") {
            return dir.join("config.yaml");
        }
        let candidates = match self.os() {
            Os::Windows => vec![self.app_data().join("tabby").join("config.yaml")],
            Os::MacOs => vec![self
                .home()
                .join("Library")
                .join("Application Support")
                .join("tabby")
                .join("config.yaml")],
            Os::Linux => vec![
                self.config_home().join("tabby").join("config.yaml"),
                self.config_home().join("Tabby").join("config.yaml"),
            ],
        };
        let first = candidates[0].clone();
        self.first_existing(candidates).unwrap_or(first)
    }

    fn terminal_note(&self) -> &'static str {
        match self.os() {
            Os::Windows => {
                "WezTerm has the icons built in. JetBrains terminals cannot show the icons. MobaXterm and others: pick \"MesloLGM Nerd Font Mono\" in their own settings."
            }
            Os::MacOs => {
                "Terminal shows only 256 colours, so themes look best in iTerm2, kitty or VS Code. WezTerm and Ghostty have the icons built in. Others: pick \"MesloLGM Nerd Font Mono\" in their settings."
            }
            Os::Linux => {
                "WezTerm and Ghostty have the icons built in; Yakuake uses Konsole's profiles. st is set when it is built; Terminology, Tilda and others: pick \"MesloLGM Nerd Font\" in their settings."
            }
        }
    }

    // ---- The record ----

    fn undo_path(&self) -> PathBuf {
        self.ws.app_dir().join(UNDO_FILE)
    }

    fn undo_load(&self) -> Vec<Undo> {
        self.ws
            .read_text(&self.undo_path())
            .map(|t| undo_parse(&t))
            .unwrap_or_default()
    }

    fn undo_save(&self, list: &[Undo]) {
        let path = self.undo_path();
        if list.is_empty() {
            self.ws.remove_file(&path);
        } else {
            let _ = self.ws.write_text(&path, &undo_text(list));
        }
    }

    // ---- The state ----

    /// Everything the window shows. Runs a few programs: off the UI thread.
    pub fn status(&self) -> Status {
        let binary = self.find_binary();
        let version = binary.as_deref().and_then(|b| self.version_of(b));
        let themes_dir = self.find_themes_dir(binary.as_deref());
        let themes = themes_dir
            .as_deref()
            .map(|d| self.themes_in(d))
            .unwrap_or_default();
        let clink = (self.os() == Os::Windows)
            .then(|| self.find_clink())
            .flatten();
        Status {
            shells: self.shells(clink.as_deref()),
            font_installed: self.font_installed(),
            terminals: self.terminals(),
            undo_available: self.ws.exists(&self.undo_path()),
            installer: self.installer(),
            terminal_note: self.terminal_note(),
            binary,
            version,
            latest: None,
            themes_dir,
            themes,
            clink,
        }
    }

    /// Does what was asked and reports back with the state after it.
    pub fn perform(&self, task: &Task) -> Done {
        let mut log = Vec::new();
        let mut latest = None;
        self.trace.lock().unwrap().clear();
        let ok = match task {
            Task::Refresh => true,
            Task::CheckLatest => {
                latest = self.latest();
                if latest.is_none() {
                    log.push("Could not reach GitHub to find the newest version.".into());
                }
                latest.is_some()
            }
            Task::Install => {
                let ok = self.install(&mut log);
                latest = self.latest();
                ok
            }
            Task::Update => {
                let ok = self.update(&mut log);
                latest = self.latest();
                ok
            }
            Task::DownloadThemes => self.download_themes(&mut log),
            Task::Apply { theme, shells } => self.apply(theme.as_deref(), shells, &mut log),
            Task::Remove => self.remove(&mut log),
            Task::Uninstall => self.uninstall(&mut log),
            Task::InstallFont => self.install_font(&mut log),
            Task::SetTerminalFonts => self.set_terminal_fonts(&mut log),
            Task::RestoreTerminalFonts => self.restore_terminal_fonts(false, &mut log),
            Task::InstallClink => self.install_clink(&mut log),
        };
        if matches!(
            task,
            Task::SetTerminalFonts | Task::RestoreTerminalFonts | Task::InstallFont
        ) {
            self.note("---- the state afterwards ----");
        }
        let mut status = self.status();
        status.latest = latest;
        if matches!(
            task,
            Task::SetTerminalFonts | Task::RestoreTerminalFonts | Task::InstallFont
        ) {
            self.debug_save(&format!("{task:?}"));
            log.push(format!(
                "Every program run and what it printed is in {}.",
                self.debug_path().display()
            ));
        }
        Done { ok, log, status }
    }

    // ---- Installing and updating ----

    fn winget(&self, verb: &str, id: &str, log: &mut Vec<String>) -> bool {
        let Some(winget) = self.ws.find_program("winget") else {
            return false;
        };
        log.push(format!("Running winget {verb} {id}..."));
        let mut args = vec![
            verb,
            "--id",
            id,
            "--source",
            "winget",
            "--exact",
            "--accept-source-agreements",
            "--disable-interactivity",
        ];
        // Only install and upgrade know this flag: uninstall refuses the whole command over it.
        if verb == "uninstall" {
            args.push("--silent");
        } else {
            args.push("--accept-package-agreements");
        }
        let r = self.run(&winget, &args);
        log.extend(tail(&r));
        if !r.success {
            log.push(format!("winget {verb} did not succeed."));
        }
        r.success
    }

    /// The MSIX build from winget, the Store or the install script, taken off through
    /// PowerShell when winget itself will not: it is the same package either way.
    fn remove_appx(&self, log: &mut Vec<String>) -> bool {
        let Some(ps) = self.powershell() else {
            return false;
        };
        log.push("Removing the Oh My Posh app package...".into());
        let script = "$p = Get-AppxPackage -Name 'ohmyposh.cli'; if (-not $p) { exit 1 }; \
                      $p | Remove-AppxPackage; exit 0";
        let r = self.run(&ps, &["-NoProfile", "-NonInteractive", "-Command", script]);
        log.extend(tail(&r));
        r.success
    }

    fn install_script(&self, log: &mut Vec<String>) -> bool {
        match self.os() {
            Os::Windows => {
                let Some(ps) = self.powershell() else {
                    log.push("No PowerShell found to run the install script.".into());
                    return false;
                };
                log.push("Running the Oh My Posh install script...".into());
                let script = format!(
                    "Set-ExecutionPolicy Bypass -Scope Process -Force; \
                     Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('{INSTALL_PS1}'))"
                );
                let r = self.run(
                    &ps,
                    &[
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        &script,
                    ],
                );
                log.extend(tail(&r));
                r.success
            }
            _ => {
                let bin = self.home().join(".local").join("bin");
                let _ = self.ws.make_dir(&bin);
                log.push(format!(
                    "Running the Oh My Posh install script into {}...",
                    bin.display()
                ));
                let line = format!(
                    "curl -fsSL {INSTALL_SH} | bash -s -- -d {}",
                    sh_quote(&bin.display().to_string())
                );
                let r = self.ws.run("bash", &["-c", &line]);
                log.extend(tail(&r));
                r.success
            }
        }
    }

    fn install(&self, log: &mut Vec<String>) -> bool {
        if let Some(b) = self.find_binary() {
            log.push(format!(
                "Oh My Posh is already installed at {}.",
                b.display()
            ));
            return true;
        }
        let mut ran = false;
        match self.os() {
            Os::Windows => ran = self.winget("install", WINGET_ID, log),
            Os::MacOs => {
                if let Some(brew) = self.ws.find_program("brew") {
                    log.push("Running brew install jandedobbeleer/oh-my-posh/oh-my-posh...".into());
                    let r = self.run(&brew, &["install", "jandedobbeleer/oh-my-posh/oh-my-posh"]);
                    log.extend(tail(&r));
                    ran = r.success;
                }
            }
            Os::Linux => {}
        }
        if !ran || self.find_binary().is_none() {
            self.install_script(log);
        }
        match self.find_binary() {
            Some(b) => {
                log.push(format!("Oh My Posh installed at {}.", b.display()));
                if self.find_themes_dir(Some(&b)).is_none() {
                    self.download_themes(log);
                }
                true
            }
            None => {
                log.push("Oh My Posh was not installed. Check the lines above.".into());
                false
            }
        }
    }

    fn update(&self, log: &mut Vec<String>) -> bool {
        let Some(binary) = self.find_binary() else {
            return self.install(log);
        };
        let before = self.version_of(&binary).unwrap_or_default();
        let path = binary.display().to_string().to_lowercase();
        let mut ran = false;
        match self.os() {
            Os::Windows if path.contains("windowsapps") || path.contains("programs") => {
                ran = self.winget("upgrade", WINGET_ID, log);
            }
            Os::MacOs if path.contains("/homebrew/") || path.contains("/usr/local/") => {
                if let Some(brew) = self.ws.find_program("brew") {
                    log.push("Running brew upgrade oh-my-posh...".into());
                    let r = self.run(&brew, &["upgrade", "oh-my-posh"]);
                    log.extend(tail(&r));
                    ran = r.success;
                }
            }
            _ => {}
        }
        if !ran {
            log.push("Running oh-my-posh upgrade...".into());
            let r = self.run(&binary, &["upgrade"]);
            log.extend(tail(&r));
            if !r.success && self.os() != Os::Windows {
                self.install_script(log);
            }
        }
        let after = self.version_of(&binary).unwrap_or_default();
        if after != before {
            log.push(format!("Oh My Posh updated: {before} to {after}."));
        } else {
            log.push(format!("Oh My Posh is at version {after}."));
        }
        true
    }

    /// Takes the prompt out of every shell, puts the terminals back as they were, then
    /// takes Oh My Posh off the machine the way it came: the package manager that
    /// installed it, or the files the install script put there. Its cache, with the
    /// init scripts and themes, goes too. The font goes only when this program installed it.
    fn uninstall(&self, log: &mut Vec<String>) -> bool {
        self.remove(log);
        self.restore_terminal_fonts(true, log);
        let Some(binary) = self.find_binary() else {
            log.push("Oh My Posh is not installed.".into());
            return true;
        };
        let cache = self.cache_dir(&binary);
        let path = binary.display().to_string().to_lowercase();
        let home = self.home();
        let mut ran = false;
        match self.os() {
            Os::Windows if path.contains("windowsapps") => {
                ran = self.winget("uninstall", WINGET_ID, log);
                if !ran || self.ws.exists(&binary) {
                    ran = self.remove_appx(log);
                }
            }
            Os::Windows if path.contains("scoop") => {
                if let Some(scoop) = self.ws.find_program("scoop") {
                    log.push("Running scoop uninstall oh-my-posh...".into());
                    let r = self.run(&scoop, &["uninstall", "oh-my-posh"]);
                    log.extend(tail(&r));
                    ran = r.success;
                }
            }
            Os::Windows if path.contains("chocolatey") => {
                if let Some(choco) = self.ws.find_program("choco") {
                    log.push("Running choco uninstall oh-my-posh...".into());
                    let r = self.run(&choco, &["uninstall", "oh-my-posh", "-y"]);
                    log.extend(tail(&r));
                    ran = r.success;
                }
            }
            Os::Windows => {
                // The older install script's layout: Programs\oh-my-posh\{bin,themes}.
                // winget's older, non-MSIX builds lived there too and are listed by it.
                ran = self.winget("uninstall", WINGET_ID, log);
                if let Some(root) = binary.parent().and_then(Path::parent) {
                    if root.ends_with("oh-my-posh") && self.ws.exists(root) {
                        match self.ws.remove_dir(root) {
                            Ok(()) => {
                                log.push(format!("Removed {}.", root.display()));
                                ran = true;
                            }
                            Err(e) => {
                                log.push(format!("Could not remove {}: {e}.", root.display()))
                            }
                        }
                    }
                }
            }
            Os::MacOs if path.contains("/homebrew/") || path.contains("/usr/local/cellar") => {
                if let Some(brew) = self.ws.find_program("brew") {
                    log.push("Running brew uninstall oh-my-posh...".into());
                    let r = self.run(&brew, &["uninstall", "oh-my-posh"]);
                    log.extend(tail(&r));
                    ran = r.success;
                }
            }
            Os::MacOs if path.starts_with("/opt/local/") => {
                log.push(
                    "Oh My Posh came from MacPorts: run `sudo port uninstall oh-my-posh`.".into(),
                );
                ran = true;
            }
            Os::MacOs | Os::Linux => {}
        }
        if !ran && self.ws.exists(&binary) {
            // A plain file from the install script, or a Homebrew link; a folder that only
            // root may write to is reported rather than fought.
            let own = binary.starts_with(&home);
            self.ws.remove_file(&binary);
            if self.ws.exists(&binary) {
                log.push(if own {
                    format!("Could not delete {}.", binary.display())
                } else {
                    format!(
                        "{} needs administrator rights to delete: run `sudo rm {}`.",
                        binary.display(),
                        binary.display()
                    )
                });
            } else {
                log.push(format!("Deleted {}.", binary.display()));
            }
        }
        if self.ws.exists(&cache) {
            match self.ws.remove_dir(&cache) {
                Ok(()) => log.push(format!(
                    "Removed the cache, init scripts and themes at {}.",
                    cache.display()
                )),
                Err(e) => log.push(format!("Could not remove {}: {e}.", cache.display())),
            }
        }
        let themes = home.join(".poshthemes");
        if self.ws.exists(&themes) && self.ws.remove_dir(&themes).is_ok() {
            log.push(format!("Removed {}.", themes.display()));
        }
        match self.find_binary() {
            None => {
                log.push("Oh My Posh is uninstalled.".into());
                true
            }
            Some(b) => {
                log.push(format!(
                    "Oh My Posh is still at {}. Check the lines above.",
                    b.display()
                ));
                false
            }
        }
    }

    fn download_themes(&self, log: &mut Vec<String>) -> bool {
        let Some(binary) = self.find_binary() else {
            log.push("Install Oh My Posh first.".into());
            return false;
        };
        let dir = self.cache_dir(&binary).join("themes");
        let zip = self.ws.temp_dir().join("oh-my-posh-themes.zip");
        let _ = self.ws.make_dir(&dir);
        log.push(format!("Downloading the themes into {}...", dir.display()));
        let r = match self.os() {
            Os::Windows => {
                let Some(ps) = self.powershell() else {
                    log.push("No PowerShell found to unpack the themes.".into());
                    return false;
                };
                let script = format!(
                    "$ProgressPreference = 'SilentlyContinue'; \
                     Invoke-WebRequest -Uri '{THEMES_ZIP}' -OutFile {z}; \
                     Expand-Archive -Path {z} -DestinationPath {d} -Force; \
                     Remove-Item {z}",
                    z = ps_quote(&zip.display().to_string()),
                    d = ps_quote(&dir.display().to_string())
                );
                self.run(&ps, &["-NoProfile", "-NonInteractive", "-Command", &script])
            }
            _ => {
                let line = format!(
                    "curl -fsSL {THEMES_ZIP} -o {z} && unzip -o -q {z} -d {d}; rm -f {z}",
                    z = sh_quote(&zip.display().to_string()),
                    d = sh_quote(&dir.display().to_string())
                );
                self.ws.run("bash", &["-c", &line])
            }
        };
        log.extend(tail(&r));
        let n = self.themes_in(&dir).len();
        if n > 0 {
            log.push(format!("{n} themes ready."));
            true
        } else {
            log.push("No themes arrived. Check the connection and try again.".into());
            false
        }
    }

    fn install_clink(&self, log: &mut Vec<String>) -> bool {
        if self.os() != Os::Windows {
            return false;
        }
        if self.find_clink().is_none() {
            self.winget("install", CLINK_WINGET_ID, log);
        }
        match self.find_clink() {
            Some(clink) => {
                // Clink loads into every new cmd window from now on.
                let r = self.run(&clink, &["autorun", "install"]);
                log.push(if r.success {
                    "Clink is installed and starts with every Command Prompt.".into()
                } else {
                    "Clink is installed; run `clink autorun install` in cmd to start it automatically.".into()
                });
                true
            }
            None => {
                log.push("Clink was not installed. Check the lines above.".into());
                false
            }
        }
    }

    fn install_font(&self, log: &mut Vec<String>) -> bool {
        let Some(binary) = self.find_binary() else {
            log.push("Install Oh My Posh first: it installs the font.".into());
            return false;
        };
        let dir = self.user_font_dir();
        let before = self.ws.list_files(&dir, 2);
        log.push(format!("Installing {FONT_NAME}..."));
        let r = self.run(&binary, &["font", "install", FONT_PACKAGE]);
        log.extend(tail(&r));
        // The files that appeared are this program's to take away again.
        let mut changes = Changes {
            list: self.undo_load(),
        };
        for f in self.ws.list_files(&dir, 2) {
            if !before.contains(&f) {
                changes.remember(&format!("font-file:{}", f.display()), "installed", None);
            }
        }
        self.undo_save(&changes.list);
        if self.font_installed() {
            log.push(format!("{FONT_NAME} is installed."));
            true
        } else {
            log.push("The font did not install. Check the lines above.".into());
            false
        }
    }

    // ---- The terminals: setting the font ----

    fn set_terminal_fonts(&self, log: &mut Vec<String>) -> bool {
        let mut changes = Changes {
            list: self.undo_load(),
        };
        let mut any = false;
        for t in self.terminals() {
            let ok = match t.kind {
                TerminalKind::VsCode => self.vscode_apply(&t, &mut changes, log),
                TerminalKind::WindowsTerminal => self.wt_apply(&t, &mut changes, log),
                TerminalKind::ConsoleHost => self.console_apply(&mut changes, log),
                TerminalKind::GnomeTerminal | TerminalKind::Tilix | TerminalKind::MateTerminal => {
                    self.gsettings_apply(&t, &mut changes, log)
                }
                TerminalKind::Gsettings => self.gs_global_apply(&t, &mut changes, log),
                TerminalKind::File => self.file_apply(&t, &mut changes, log),
                TerminalKind::Hyper => self.hyper_apply(&t, &mut changes, log),
                TerminalKind::ConEmu => self.conemu_apply(&t, &mut changes, log),
                TerminalKind::Putty => self.putty_apply(&t, &mut changes, log),
                TerminalKind::Terminator => self.terminator_apply(&t, &mut changes, log),
                TerminalKind::XResources => self.xres_apply(&t, &mut changes, log),
                TerminalKind::Konsole => self.konsole_apply(&mut changes, log),
                TerminalKind::XfceTerminal => self.xfce_apply(&t, &mut changes, log),
                TerminalKind::AppleTerminal => self.apple_terminal_apply(&t, &mut changes, log),
                TerminalKind::ITerm2 => self.iterm_apply(&mut changes, log),
                TerminalKind::Kitty => self.kitty_apply(&t, &mut changes, log),
                TerminalKind::Alacritty => self.alacritty_apply(&t, &mut changes, log),
                TerminalKind::Tabby => self.tabby_apply(&t, &mut changes, log),
                TerminalKind::BuiltIn => {
                    log.push(format!("{}: has the icons built in; nothing to set.", t.name));
                    false
                }
            };
            any |= ok;
        }
        self.undo_save(&changes.list);
        if !any {
            log.push("No terminal settings were changed.".into());
        }
        log.push(self.terminal_note().into());
        any
    }

    /// A settings file as text, or why it cannot be changed.
    fn settings_text(&self, name: &str, path: &Path, log: &mut Vec<String>) -> Option<String> {
        if !self.ws.exists(path) {
            return Some(String::new());
        }
        let text = self.ws.read_text(path);
        if text.is_none() {
            log.push(format!("{name}: could not read {}.", path.display()));
        }
        text
    }

    fn write_settings(&self, name: &str, path: &Path, text: &str, log: &mut Vec<String>) -> bool {
        match self.ws.write_text(path, text) {
            Ok(()) => true,
            Err(e) => {
                log.push(format!("{name}: not changed: {e}."));
                false
            }
        }
    }

    fn vscode_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let target = format!("vscode:{}", t.target);
        changes.remember(&target, VSCODE_FONT, jsonc::get_raw(&text, &[VSCODE_FONT]));
        changes.remember(
            &target,
            VSCODE_CONTRAST,
            jsonc::get_raw(&text, &[VSCODE_CONTRAST]),
        );
        let font = self.windows_font_name(false);
        // VS Code's terminal.integrated.fontFamily accepts a comma-separated list;
        // fallbacks keep the terminal usable if the primary font is not yet visible
        // (VS Code caches fonts and may need a restart to pick up a new install).
        let family = format!("{font}, Consolas, 'Courier New', monospace");
        self.note(format!(
            "vscode: detected font family {font}; writing {family}"
        ));
        let edited = jsonc::set_string(&text, &[VSCODE_FONT], &family)
            .and_then(|s| jsonc::set_raw(&s, &[VSCODE_CONTRAST], "1"));
        match edited {
            Ok(new) => {
                if self.write_settings(&t.name, &path, &new, log) {
                    log.push(format!(
                        "{}: terminal font set to {font} (with fallbacks); its contrast rule no longer recolours the prompt. If icons still show as boxes or question marks: (1) reload VS Code window (Ctrl+Shift+P, Developer: Reload Window) so it picks up the newly installed font, (2) confirm Oh My Posh is running — open a new VS Code terminal and the prompt should be themed, and (3) check Windows Settings > Personalization > Fonts that a MesloLGM Nerd Font family is actually installed.",
                        t.name
                    ));
                    return true;
                }
                false
            }
            Err(e) => {
                log.push(format!("{}: not changed: {e}.", t.name));
                false
            }
        }
    }

    fn wt_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let font = self.windows_font_name(false);
        match wt_with_font(&text, &format!("wt:{}", t.target), &font, changes) {
            Ok((new, profiles)) => {
                if self.write_settings(&t.name, &path, &new, log) {
                    log.push(format!(
                        "{}: font set to {font} in {} and the colour adjustment turned off.",
                        t.name,
                        profiles_text(profiles)
                    ));
                    return true;
                }
                false
            }
            Err(e) => {
                log.push(format!("{}: not changed: {e}.", t.name));
                false
            }
        }
    }

    fn reg_add(&self, reg: &Path, key: &str, name: &str, ty: &str, data: &str) -> bool {
        self.run(
            reg,
            &["add", key, "/v", name, "/t", ty, "/d", data, "/f"],
        )
        .success
    }

    fn reg_delete(&self, reg: &Path, key: &str, name: &str) -> bool {
        self.run(reg, &["delete", key, "/v", name, "/f"]).success
    }

    fn console_apply(&self, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let Some(reg) = self.ws.find_program("reg") else {
            log.push("Console windows: reg.exe was not found.".into());
            return false;
        };
        let keys = self.console_keys();
        let mut ok = true;
        let mut n = 0;
        let font = self.windows_font_name(true);
        for (i, k) in keys.iter().enumerate() {
            let root = i == 0;
            if !root && reg_value(k, "FaceName").is_none() {
                continue; // inherits the root's font
            }
            let target = format!("console:{}", k.path);
            // 0x36: a TrueType font; 16 pixels tall, width left to the font.
            let mut wanted: Vec<(&str, &str, &str)> = vec![
                ("FaceName", "REG_SZ", font.as_str()),
                ("FontFamily", "REG_DWORD", "54"),
            ];
            if root {
                wanted.push(("FontSize", "REG_DWORD", "1048576"));
            }
            for (name, ty, data) in wanted {
                let before = reg_value(k, name).map(|(ty, data)| format!("{ty} {data}"));
                changes.remember(&target, name, before);
                ok &= self.reg_add(&reg, &k.path, name, ty, data);
            }
            n += 1;
        }
        if keys.is_empty() {
            // Nothing saved yet: the root key is made with the font.
            let target = r"console:HKEY_CURRENT_USER\Console";
            for (name, ty, data) in [
                ("FaceName", "REG_SZ", font.as_str()),
                ("FontFamily", "REG_DWORD", "54"),
                ("FontSize", "REG_DWORD", "1048576"),
            ] {
                changes.remember(target, name, None);
                ok &= self.reg_add(&reg, r"HKEY_CURRENT_USER\Console", name, ty, data);
            }
            n = 1;
        }
        if ok {
            log.push(format!(
                "Console windows: font set to {font} in {} for new windows. A shortcut with its own font settings keeps them.",
                profiles_text(n)
            ));
        } else {
            log.push("Console windows: could not change every font in the registry.".into());
        }
        ok
    }

    fn gsettings_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let Some(gsettings) = self.ws.find_program("gsettings") else {
            return false;
        };
        let list = match t.kind {
            TerminalKind::Tilix => TILIX,
            TerminalKind::MateTerminal => MATE,
            _ => GNOME,
        };
        let profiles = self.gsettings_profiles(&gsettings, list);
        let font = format!("{FONT_NAME} 11");
        let mut ok = !profiles.is_empty();
        for p in &profiles {
            let target = format!("gsettings:{p}");
            changes.remember(&target, "use-system-font", self.ask(&gsettings, &["get", p, "use-system-font"]));
            changes.remember(&target, "font", self.ask(&gsettings, &["get", p, "font"]));
            ok &= self.run(&gsettings, &["set", p, "use-system-font", "false"]).success;
            ok &= self.run(&gsettings, &["set", p, "font", &font]).success;
        }
        if ok {
            log.push(format!(
                "{}: font set to {FONT_NAME} in {}.",
                t.name,
                profiles_text(profiles.len())
            ));
        } else {
            log.push(format!("{}: could not set the font.", t.name));
        }
        ok
    }

    fn konsole_apply(&self, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let mut profiles = self.konsole_profiles();
        let rc_path = self.konsolerc();
        let font = format!("{FONT_NAME},11,-1,5,50,0,0,0,0,0");
        if profiles.is_empty() {
            // None saved yet: one is made, and made the default, since Konsole's built-in
            // profile cannot be changed from outside.
            let made = self.konsole_dir().join("IdleWalker.profile");
            changes.remember(&format!("konsole-file:{}", made.display()), "created", None);
            if self
                .ws
                .write_text(&made, "[General]\nName=IdleWalker\nParentProfile=FALLBACK/\n")
                .is_err()
            {
                log.push(format!("Konsole: could not write {}.", made.display()));
                return false;
            }
            let rc = self.ws.read_text(&rc_path).unwrap_or_default();
            changes.remember(
                &format!("konsole:{}", rc_path.display()),
                "Desktop Entry/DefaultProfile",
                ini_get(&rc, "Desktop Entry", "DefaultProfile"),
            );
            let _ = self.ws.write_text(
                &rc_path,
                &ini_set(&rc, "Desktop Entry", "DefaultProfile", "IdleWalker.profile"),
            );
            profiles.push(made);
        }
        let mut ok = true;
        for p in &profiles {
            let text = self.ws.read_text(p).unwrap_or_default();
            changes.remember(
                &format!("konsole:{}", p.display()),
                "Appearance/Font",
                ini_get(&text, "Appearance", "Font"),
            );
            let new = ini_set(&text, "Appearance", "Font", &font);
            if let Err(e) = self.ws.write_text(p, &new) {
                log.push(format!("Konsole: could not write {}: {e}.", p.display()));
                ok = false;
            }
        }
        if ok {
            log.push(format!(
                "Konsole: font set to {FONT_NAME} in {}.",
                profiles_text(profiles.len())
            ));
        }
        ok
    }

    fn xfce_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let target = format!("xfce:{}", t.target);
        changes.remember(&target, "Configuration/FontName", ini_get(&text, "Configuration", "FontName"));
        changes.remember(&target, "Configuration/FontUseSystem", ini_get(&text, "Configuration", "FontUseSystem"));
        let new = ini_set(&text, "Configuration", "FontName", &format!("{FONT_NAME} 11"));
        let new = ini_set(&new, "Configuration", "FontUseSystem", "FALSE");
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!("{}: font set to {FONT_NAME}.", t.name));
            return true;
        }
        false
    }

    fn kitty_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        changes.remember(&format!("kitty:{}", t.target), "font_family", conf_get(&text, "font_family"));
        let new = conf_set(&text, "font_family", Some(FONT_NAME));
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!("{}: font set to {FONT_NAME}.", t.name));
            return true;
        }
        false
    }

    fn alacritty_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        changes.remember(&format!("alacritty:{}", t.target), "font.normal.family", alacritty_font(&text));
        let new = alacritty_with_font(&text, Some(FONT_NAME));
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!("{}: font set to {FONT_NAME}.", t.name));
            return true;
        }
        false
    }

    /// Tabby has one font for every tab and profile: `terminal.font` in `config.yaml`.
    fn tabby_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let new = yaml_set(&text, "terminal", "font", Some(FONT_NAME));
        if yaml_get(&new, "terminal", "font").as_deref() != Some(FONT_NAME) {
            log.push(format!(
                "{}: the terminal section of {} is not in the shape expected; pick \"{FONT_NAME}\" under Settings > Appearance.",
                t.name, t.target
            ));
            return false;
        }
        changes.remember(
            &format!("tabby:{}", t.target),
            "terminal.font",
            yaml_get(&text, "terminal", "font"),
        );
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!("{}: font set to {FONT_NAME}.", t.name));
            return true;
        }
        false
    }

    /// A terminal whose settings file has a known shape: each key in its recipe is
    /// recorded and set, and the keys that would override them are taken out.
    fn file_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let Some(r) = recipe(&t.name) else {
            return false;
        };
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        if !self.ws.exists(&path) {
            if let Some(dir) = path.parent() {
                let _ = self.ws.make_dir(dir);
            }
        }
        let text = if r.name == "foot" { foot_main(&text) } else { text };
        let target = format!("{}:{}", r.shape.kind(), t.target);
        let mut new = text.clone();
        let mut edits: Vec<(&str, Option<String>)> = r
            .keys
            .iter()
            .map(|(k, f)| (*k, Some(f.text())))
            .collect();
        for k in r.removes {
            if shape_get(r.shape, &text, r.section, k).is_some() {
                edits.push((k, None));
            }
        }
        for (k, value) in edits {
            changes.remember(
                &target,
                &undo_key(r.section, k),
                shape_get(r.shape, &text, r.section, k),
            );
            match shape_set(r.shape, &new, r.section, k, value.as_deref()) {
                Ok(n) => new = n,
                Err(e) => {
                    log.push(format!("{}: not changed: {e}.", t.name));
                    return false;
                }
            }
        }
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!("{}: font set to {FONT_NAME}.", t.name));
            return true;
        }
        false
    }

    fn gs_global_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let (Some(r), Some(gsettings)) = (gs_recipe(&t.name), self.ws.find_program("gsettings")) else {
            return false;
        };
        let mut ok = true;
        for (schema, key, form) in r.settings {
            let target = format!("gsettings:{schema}");
            changes.remember(&target, key, self.ask(&gsettings, &["get", schema, key]));
            ok &= self.run(&gsettings, &["set", schema, key, &form.text()]).success;
        }
        log.push(if ok {
            format!("{}: font set to {FONT_NAME}.", t.name)
        } else {
            format!("{}: could not set the font.", t.name)
        });
        ok
    }

    fn hyper_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        match hyper_with_font(&text, Some(HYPER_FONTS)) {
            Ok(new) => {
                changes.remember(&format!("hyper:{}", t.target), "fontFamily", hyper_font(&text));
                if self.write_settings(&t.name, &path, &new, log) {
                    log.push(format!("{}: font set to {FONT_NAME}.", t.name));
                    return true;
                }
                false
            }
            Err(e) => {
                log.push(format!(
                    "{}: not changed: {e}; pick \"{FONT_NAME}\" in {}.",
                    t.name, t.target
                ));
                false
            }
        }
    }

    fn conemu_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        if t.target == CONEMU_KEY {
            let Some(reg) = self.ws.find_program("reg") else {
                return false;
            };
            let configs = self.reg_children(&reg, CONEMU_KEY);
            let mut ok = true;
            for k in &configs {
                let before = reg_value(k, "FontName").map(|(ty, d)| format!("{ty} {d}"));
                changes.remember(&format!("conemu-reg:{}", k.path), "FontName", before);
                ok &= self.reg_add(&reg, &k.path, "FontName", "REG_SZ", FONT_MONO);
            }
            log.push(if ok {
                format!("{}: font set to {FONT_MONO} in {}.", t.name, profiles_text(configs.len()))
            } else {
                format!("{}: could not change every configuration.", t.name)
            });
            return ok;
        }
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let fonts = conemu_fonts(&text);
        if fonts.is_empty() {
            log.push(format!("{}: no configuration found in {}.", t.name, t.target));
            return false;
        }
        let mut new = text.clone();
        for (config, font) in &fonts {
            changes.remember(&format!("conemu:{}", t.target), config, font.clone());
            new = conemu_with_font(&new, config, Some(FONT_MONO));
        }
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!(
                "{}: font set to {FONT_MONO} in {}.",
                t.name,
                profiles_text(fonts.len())
            ));
            return true;
        }
        false
    }

    /// PuTTY's sessions: registry keys on Windows, where a font needs its four values
    /// or PuTTY ignores it; a file per session elsewhere.
    fn putty_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        if self.os() == Os::Windows {
            let Some(reg) = self.ws.find_program("reg") else {
                return false;
            };
            let mut sessions = self.reg_children(&reg, PUTTY_SESSIONS);
            if sessions.is_empty() {
                sessions.push(RegKey {
                    path: format!("{PUTTY_SESSIONS}\\Default%20Settings"),
                    values: Vec::new(),
                });
            }
            let mut ok = true;
            for k in &sessions {
                let target = format!("putty:{}", k.path);
                let mut wanted = vec![("Font", "REG_SZ", FONT_MONO)];
                for (name, data) in [("FontIsBold", "0"), ("FontCharSet", "0"), ("FontHeight", "11")] {
                    if reg_value(k, name).is_none() {
                        wanted.push((name, "REG_DWORD", data));
                    }
                }
                for (name, ty, data) in wanted {
                    let before = reg_value(k, name).map(|(ty, d)| format!("{ty} {d}"));
                    changes.remember(&target, name, before);
                    ok &= self.reg_add(&reg, &k.path, name, ty, data);
                }
            }
            log.push(if ok {
                format!("{}: font set to {FONT_MONO} in {}.", t.name, profiles_text(sessions.len()))
            } else {
                format!("{}: could not change every session.", t.name)
            });
            return ok;
        }
        let dir = PathBuf::from(&t.target);
        let mut files = self.putty_session_files();
        if files.is_empty() {
            let _ = self.ws.make_dir(&dir);
            files.push(dir.join("Default%20Settings"));
        }
        let font = format!("client:{FONT_NAME} 11");
        let mut ok = true;
        for f in &files {
            let text = self.ws.read_text(f).unwrap_or_default();
            changes.remember(&format!("kv:{}", f.display()), "Font", kv_get(&text, "Font"));
            ok &= self.ws.write_text(f, &kv_set(&text, "Font", Some(&font))).is_ok();
        }
        log.push(if ok {
            format!("{}: font set to {FONT_NAME} in {}.", t.name, profiles_text(files.len()))
        } else {
            format!("{}: could not write every session file.", t.name)
        });
        ok
    }

    fn terminator_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        if !self.ws.exists(&path) {
            if let Some(dir) = path.parent() {
                let _ = self.ws.make_dir(dir);
            }
        }
        let mut profiles = terminator_profiles(&text);
        if profiles.is_empty() {
            profiles.push("default".into());
        }
        let target = format!("terminator:{}", t.target);
        let font = format!("{FONT_NAME} 11");
        let mut new = text.clone();
        for p in &profiles {
            changes.remember(&target, &format!("{p}/font"), terminator_get(&text, p, "font"));
            changes.remember(
                &target,
                &format!("{p}/use_system_font"),
                terminator_get(&text, p, "use_system_font"),
            );
            new = terminator_set(&new, p, "font", Some(&font));
            new = terminator_set(&new, p, "use_system_font", Some("False"));
        }
        if self.write_settings(&t.name, &path, &new, log) {
            log.push(format!(
                "{}: font set to {FONT_NAME} in {}.",
                t.name,
                profiles_text(profiles.len())
            ));
            return true;
        }
        false
    }

    /// xterm and urxvt: their resources in the X resources file, loaded with xrdb.
    fn xres_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let path = PathBuf::from(&t.target);
        let Some(text) = self.settings_text(&t.name, &path, log) else {
            return false;
        };
        let app = xres_app(&t.name);
        let target = format!("xres:{}", t.target);
        // Key, value, and whether an existing value is left alone.
        let settings: Vec<(&str, String, bool)> = if t.name == "xterm" {
            vec![
                ("faceName", FONT_NAME.to_string(), false),
                ("faceSize", "11".to_string(), true),
            ]
        } else {
            vec![("font", format!("xft:{FONT_NAME}:size=11"), false)]
        };
        let mut new = text.clone();
        for (key, value, keep_existing) in settings {
            let old = xres_get(&text, app, key);
            if keep_existing && old.is_some() {
                continue;
            }
            changes.remember(&target, &format!("{app}/{key}"), old);
            new = xres_set(&new, app, key, Some(&value));
        }
        if !self.write_settings(&t.name, &path, &new, log) {
            return false;
        }
        let loaded = self.xrdb_merge(&path);
        log.push(format!(
            "{}: font set to {FONT_NAME} in {}{}.",
            t.name,
            t.target,
            if loaded { " and loaded with xrdb" } else { "; it loads at the next sign-in" }
        ));
        true
    }

    fn xrdb_merge(&self, path: &Path) -> bool {
        let Some(xrdb) = self.ws.find_program("xrdb") else {
            return false;
        };
        self.run(&xrdb, &["-merge", &path.display().to_string()]).success
    }

    /// The PostScript names macOS itself resolves the font to, asked through a fresh
    /// process by each of the font's names; the Mono font's first. For the debug file,
    /// the Meslo font files on the machine too. The window is told when macOS resolves
    /// none of the names, since Terminal cannot then either.
    fn apple_font_probe(&self, osascript: &Path, log: &mut Vec<String>) -> Vec<String> {
        for dir in [self.user_font_dir(), PathBuf::from("/Library/Fonts")] {
            let files: Vec<String> = self
                .ws
                .list_files(&dir, 4)
                .iter()
                .filter(|f| f.display().to_string().to_lowercase().contains("meslo"))
                .map(|f| f.display().to_string())
                .collect();
            self.note(format!(
                "Meslo files under {}: {}",
                dir.display(),
                if files.is_empty() { "none".to_string() } else { files.join(", ") }
            ));
        }
        let answer = self
            .ask_all(osascript, &["-l", "JavaScript", "-e", APPLE_FONT_PROBE])
            .unwrap_or_default();
        let mut resolved: Vec<String> = Vec::new();
        for line in answer.lines() {
            if let Some(rest) = line.split("opens as ").nth(1) {
                let name = rest.trim_end_matches(", found").trim();
                if !name.is_empty() && !resolved.iter().any(|r| r == name) {
                    resolved.push(name.to_string());
                }
            }
        }
        if answer.contains("missing") && resolved.is_empty() {
            log.push("Terminal: macOS does not see the font by any of its names (details in the debug file). Quit Terminal, sign out and in, and try again.".into());
        }
        resolved
    }

    /// Terminal's profiles ("settings sets"), each with its font, through AppleScript.
    fn apple_terminal_fonts(&self, osascript: &Path) -> Vec<(String, String)> {
        self.ask_all(osascript, &["-e", APPLE_TERMINAL_LIST])
            .unwrap_or_default()
            .lines()
            .filter_map(|l| {
                let (name, font) = l.split_once('\t')?;
                Some((name.to_string(), font.trim().to_string()))
            })
            .collect()
    }

    /// One AppleScript that sets every named profile's font, and, given a font, the
    /// default and start-up ones too.
    fn apple_terminal_script(fonts: &[(String, String)], defaults: Option<&str>) -> String {
        let mut lines = vec!["tell application \"Terminal\"".to_string()];
        if let Some(font) = defaults {
            for which in ["default settings", "startup settings"] {
                lines.push(format!(
                    "set font name of {which} to \"{}\"",
                    applescript_text(font)
                ));
            }
        }
        for (name, font) in fonts {
            lines.push(format!(
                "set font name of settings set \"{}\" to \"{}\"",
                applescript_text(name),
                applescript_text(font)
            ));
        }
        lines.push("end tell".into());
        lines.join("\n")
    }

    fn apple_terminal_apply(&self, t: &TerminalStatus, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let Some(osascript) = self.ws.find_program("osascript") else {
            return false;
        };
        log.push("Terminal: setting its font (macOS may ask to let IdleWalker control Terminal)...".into());
        let mut names: Vec<String> = self.apple_font_probe(&osascript, log);
        for n in APPLE_TERMINAL_FONT_NAMES {
            if !names.iter().any(|m| m == n) {
                names.push(n.to_string());
            }
        }
        let mut current = self.apple_terminal_fonts(&osascript);
        if current.is_empty() {
            // Not told: the names from its settings file, their fonts unknown.
            let plist = self
                .ws
                .find_program("defaults")
                .and_then(|d| self.ask_all(&d, &["export", "com.apple.Terminal", "-"]))
                .unwrap_or_default();
            let mut names = plist_profile_names(&plist);
            if names.is_empty() {
                names.push(t.target.clone());
            }
            current = names.into_iter().map(|n| (n, String::new())).collect();
        }
        for (name, font) in &current {
            changes.remember(
                &format!("terminal.app:{name}"),
                "font name",
                (!font.is_empty()).then(|| font.clone()),
            );
        }
        // Terminal answers "done" to a font name it does not know, and keeps the
        // old font. So each name is tried and the profiles read back, until one is
        // reported back as the Nerd Font.
        let mut left_over = Vec::new();
        for font in &names {
            let wanted: Vec<(String, String)> = current
                .iter()
                .map(|(n, _)| (n.clone(), font.clone()))
                .collect();
            let script = Self::apple_terminal_script(&wanted, Some(font));
            let r = self.run(&osascript, &["-e", &script]);
            if !r.success {
                log.extend(tail(&r));
                if apple_events_denied(&r) {
                    log.push("Terminal: macOS did not let IdleWalker control Terminal. Allow it under System Settings > Privacy & Security > Automation, then try again.".into());
                }
                log.push(format!("Terminal: could not set the font; pick {FONT_MONO} under Settings > Profiles > Text."));
                return false;
            }
            let after = self.apple_terminal_fonts(&osascript);
            left_over = after
                .iter()
                .filter(|(_, f)| !is_nerd_font(f))
                .map(|(n, f)| format!("{n} ({f})"))
                .collect();
            if left_over.is_empty() {
                log.push(format!(
                    "Terminal: font set to {FONT_MONO} in {}. Open a new Terminal window; Terminal saves the change to its settings file when it quits.",
                    profiles_text(wanted.len())
                ));
                return true;
            }
        }
        log.push(format!(
            "Terminal: did not take the font; it still reports {}. Terminal may not see the font until it is quit and opened again, or pick {FONT_MONO} under Settings > Profiles > Text.",
            left_over.join(", ")
        ));
        false
    }

    /// iTerm2's settings as `defaults export` prints them, or why not.
    fn iterm_plist(&self, log: &mut Vec<String>) -> Option<(PathBuf, String)> {
        let (Some(defaults), Some(pgrep)) = (
            self.ws.find_program("defaults"),
            self.ws.find_program("pgrep"),
        ) else {
            return None;
        };
        if self.run(&pgrep, &["-x", "iTerm2"]).success {
            log.push("iTerm2: quit iTerm2 first, then try again; it keeps its own copy of the settings while it runs.".into());
            return None;
        }
        let plist = self.ask_all(&defaults, &["export", "com.googlecode.iterm2", "-"]);
        if plist.is_none() {
            log.push("iTerm2: could not read its settings.".into());
        }
        plist.map(|p| (defaults, p))
    }

    fn iterm_import(&self, defaults: &Path, plist: &str) -> bool {
        let tmp = self.ws.temp_dir().join("idlewalker-iterm2.plist");
        let path = tmp.display().to_string();
        let imported = self.ws.write_text(&tmp, plist).is_ok()
            && self
                .run(defaults, &["import", "com.googlecode.iterm2", &path])
                .success;
        self.ws.remove_file(&tmp);
        imported
    }

    fn iterm_apply(&self, changes: &mut Changes, log: &mut Vec<String>) -> bool {
        let Some((defaults, plist)) = self.iterm_plist(log) else {
            return false;
        };
        for f in iterm_font_entries(&plist) {
            changes.remember(&format!("iterm2:{}", f.guid), &f.key, Some(f.value.clone()));
        }
        let (new, n) = iterm_with_fonts(&plist, |f| {
            Some(format!("{FONT_MONO_POSTSCRIPT} {}", font_size_of(&f.value)))
        });
        if n == 0 {
            log.push("iTerm2: no profile saved yet; open its settings once, then try again.".into());
            return false;
        }
        if self.iterm_import(&defaults, &new) {
            log.push(format!(
                "iTerm2: font set to {FONT_MONO} in {}.",
                profiles_text(iterm_fonts(&new).len())
            ));
            return true;
        }
        log.push(format!("iTerm2: could not write its settings; pick {FONT_MONO} under Settings > Profiles > Text."));
        false
    }

    // ---- The terminals: putting the font back ----

    /// Every terminal setting back to what it was, from the record; with `fonts_too`,
    /// the font files this program installed go as well. What could not be put back
    /// stays on record.
    fn restore_terminal_fonts(&self, fonts_too: bool, log: &mut Vec<String>) -> bool {
        let list = self.undo_load();
        if list.is_empty() {
            log.push("No terminal settings of this program's to put back.".into());
            return true;
        }
        let mut keep: Vec<Undo> = Vec::new();
        let mut kinds: Vec<&str> = Vec::new();
        for u in &list {
            let kind = undo_kind(&u.target);
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        let mut all_ok = true;
        for kind in kinds {
            let entries: Vec<&Undo> = list
                .iter()
                .filter(|u| undo_kind(&u.target) == kind)
                .collect();
            let ok = match kind {
                "vscode" => self.json_restore("VS Code", &entries, log),
                "wt" => self.wt_restore(&entries, log),
                "console" => self.reg_restore("Console windows", &entries, log),
                "conemu-reg" => self.reg_restore("ConEmu", &entries, log),
                "putty" => self.reg_restore("PuTTY", &entries, log),
                "ini" => self.shape_restore(Shape::Ini, &entries, log),
                "kv" => self.shape_restore(Shape::Kv, &entries, log),
                "toml" => self.shape_restore(Shape::Toml, &entries, log),
                "json" => self.shape_restore(Shape::Json, &entries, log),
                "hyper" => self.hyper_restore(&entries, log),
                "conemu" => self.conemu_restore(&entries, log),
                "terminator" => self.terminator_restore(&entries, log),
                "xres" => self.xres_restore(&entries, log),
                "gsettings" => self.gsettings_restore(&entries, log),
                "konsole" => self.konsole_restore(&entries, log),
                "xfce" => self.ini_restore("Xfce Terminal", &entries, log),
                "kitty" => self.conf_restore(&entries, log),
                "alacritty" => self.alacritty_restore(&entries, log),
                "tabby" => self.yaml_restore("Tabby", &entries, log),
                "terminal.app" => self.apple_terminal_restore(&entries, log),
                "iterm2" => self.iterm_restore(&entries, log),
                "font-file" => {
                    if fonts_too {
                        self.remove_font_files(&entries, log)
                    } else {
                        keep.extend(entries.iter().map(|u| (*u).clone()));
                        true
                    }
                }
                _ => true,
            };
            if !ok {
                all_ok = false;
                keep.extend(entries.iter().map(|u| (*u).clone()));
            }
        }
        self.undo_save(&keep);
        all_ok
    }

    /// Entries grouped by what they belong to, in the order first seen.
    fn by_target<'a>(entries: &[&'a Undo]) -> Vec<(String, Vec<&'a Undo>)> {
        let mut out: Vec<(String, Vec<&Undo>)> = Vec::new();
        for &u in entries {
            let place = u.target.split_once(':').map_or("", |(_, p)| p).to_string();
            match out.iter_mut().find(|(p, _)| *p == place) {
                Some((_, v)) => v.push(u),
                None => out.push((place, vec![u])),
            }
        }
        out
    }

    fn json_restore(&self, name: &str, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue; // gone: nothing to put back
            };
            let mut new = text;
            for u in list.iter().rev() {
                let r = match &u.before {
                    Some(raw) => jsonc::set_raw(&new, &[u.key.as_str()], raw),
                    None => jsonc::remove(&new, &[u.key.as_str()]),
                };
                match r {
                    Ok(t) => new = t,
                    Err(e) => {
                        log.push(format!("{name}: {file} not put back: {e}."));
                        ok = false;
                    }
                }
            }
            if self.ws.write_text(&path, &new).is_ok() {
                log.push(format!("{name}: settings put back in {file}."));
            } else {
                ok = false;
            }
        }
        ok
    }

    fn wt_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            match wt_without_font(&text, &list) {
                Ok(new) => {
                    if self.ws.write_text(&path, &new).is_ok() {
                        log.push(format!("Windows Terminal: settings put back in {file}."));
                    } else {
                        ok = false;
                    }
                }
                Err(e) => {
                    log.push(format!("Windows Terminal: {file} not put back: {e}."));
                    ok = false;
                }
            }
        }
        ok
    }

    /// Registry values back to what they were, written as `type data`, or deleted.
    fn reg_restore(&self, name: &str, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let Some(reg) = self.ws.find_program("reg") else {
            return false;
        };
        let mut ok = true;
        for (key, list) in Self::by_target(entries) {
            for u in list {
                ok &= match &u.before {
                    Some(v) => {
                        let (ty, data) = v.split_once(' ').unwrap_or((v, ""));
                        self.reg_add(&reg, &key, &u.key, ty, data)
                    }
                    None => self.reg_delete(&reg, &key, &u.key),
                };
            }
        }
        log.push(if ok {
            format!("{name}: font settings put back.")
        } else {
            format!("{name}: could not put every font setting back.")
        });
        ok
    }

    fn gsettings_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let Some(gsettings) = self.ws.find_program("gsettings") else {
            return false;
        };
        let mut ok = true;
        let mut n = 0;
        for (profile, list) in Self::by_target(entries) {
            for u in list {
                let r = match &u.before {
                    Some(v) => self.run(&gsettings, &["set", &profile, &u.key, v]).success,
                    None => self.run(&gsettings, &["reset", &profile, &u.key]).success,
                };
                ok &= r;
            }
            n += 1;
        }
        log.push(format!(
            "Terminals set through gsettings: font settings put back in {}.",
            profiles_text(n)
        ));
        ok
    }

    fn konsole_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            if list.iter().any(|u| u.key == "created") {
                self.ws.remove_file(&path);
                continue;
            }
            ok &= self.ini_restore_file(&path, &list);
        }
        log.push("Konsole: font settings put back.".into());
        ok
    }

    fn ini_restore(&self, name: &str, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            ok &= self.ini_restore_file(&PathBuf::from(&file), &list);
        }
        log.push(format!("{name}: font settings put back."));
        ok
    }

    /// Keys written as `Section/Key` back to their old values in an INI file.
    fn ini_restore_file(&self, path: &Path, list: &[&Undo]) -> bool {
        let Some(text) = self.ws.read_text(path) else {
            return true;
        };
        let mut new = text;
        for u in list.iter().rev() {
            let Some((section, key)) = u.key.split_once('/') else {
                continue;
            };
            new = match &u.before {
                Some(v) => ini_set(&new, section, key, v),
                None => ini_remove(&new, section, key),
            };
        }
        self.ws.write_text(path, &new).is_ok()
    }

    fn conf_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                new = conf_set(&new, &u.key, u.before.as_deref());
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push("kitty: font setting put back.".into());
        ok
    }

    fn alacritty_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                new = alacritty_with_font(&new, u.before.as_deref());
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push("Alacritty: font setting put back.".into());
        ok
    }

    /// Puts back keys recorded as `section.key` in a YAML file.
    fn yaml_restore(&self, name: &str, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                let (section, key) = u.key.split_once('.').unwrap_or(("", &u.key));
                new = yaml_set(&new, section, key, u.before.as_deref());
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push(format!("{name}: font setting put back."));
        ok
    }

    /// Settings of one shape back to what they were, file by file.
    fn shape_restore(&self, shape: Shape, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                let (section, key) = u.key.split_once('/').unwrap_or(("", &u.key));
                match shape_set(shape, &new, section, key, u.before.as_deref()) {
                    Ok(n) => new = n,
                    Err(e) => {
                        log.push(format!("{file}: not put back: {e}."));
                        ok = false;
                    }
                }
            }
            if shape == Shape::Kv && new.trim().is_empty() {
                self.ws.remove_file(&path); // a file this program made for the font alone
            } else {
                ok &= self.ws.write_text(&path, &new).is_ok();
            }
            log.push(format!("{file}: font setting put back."));
        }
        ok
    }

    fn hyper_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                if let Ok(n) = hyper_with_font(&new, u.before.as_deref()) {
                    new = n;
                }
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push("Hyper: font setting put back.".into());
        ok
    }

    fn conemu_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                new = conemu_with_font(&new, &u.key, u.before.as_deref());
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push("ConEmu: font settings put back.".into());
        ok
    }

    fn terminator_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                let Some((profile, key)) = u.key.split_once('/') else {
                    continue;
                };
                new = terminator_set(&new, profile, key, u.before.as_deref());
            }
            ok &= self.ws.write_text(&path, &new).is_ok();
        }
        log.push("Terminator: font settings put back.".into());
        ok
    }

    fn xres_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let mut ok = true;
        for (file, list) in Self::by_target(entries) {
            let path = PathBuf::from(&file);
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let mut new = text;
            for u in list.iter().rev() {
                let Some((app, key)) = u.key.split_once('/') else {
                    continue;
                };
                new = xres_set(&new, app, key, u.before.as_deref());
            }
            if new.trim().is_empty() {
                self.ws.remove_file(&path);
            } else {
                ok &= self.ws.write_text(&path, &new).is_ok();
                self.xrdb_merge(&path);
            }
        }
        log.push("xterm and urxvt: X resources put back; a resource taken out stays loaded until the next sign-in.".into());
        ok
    }

    fn apple_terminal_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let Some(osascript) = self.ws.find_program("osascript") else {
            return false;
        };
        let fonts: Vec<(String, String)> = entries
            .iter()
            .filter(|u| u.key == "font name")
            .filter_map(|u| {
                let name = u.target.split_once(':')?.1.to_string();
                Some((name, u.before.clone()?))
            })
            .collect();
        if fonts.is_empty() {
            return true;
        }
        let script = Self::apple_terminal_script(&fonts, None);
        let r = self.run(&osascript, &["-e", &script]);
        if r.success {
            log.push(format!(
                "Terminal: fonts put back in {}.",
                profiles_text(fonts.len())
            ));
        } else {
            log.extend(tail(&r));
            log.push("Terminal: could not put its fonts back.".into());
        }
        r.success
    }

    fn iterm_restore(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let Some((defaults, plist)) = self.iterm_plist(log) else {
            return false;
        };
        let (new, n) = iterm_with_fonts(&plist, |f| {
            entries
                .iter()
                .find(|u| u.target == format!("iterm2:{}", f.guid) && u.key == f.key)
                .and_then(|u| u.before.clone())
        });
        if n > 0 && !self.iterm_import(&defaults, &new) {
            log.push("iTerm2: could not put its fonts back.".into());
            return false;
        }
        log.push(format!("iTerm2: fonts put back in {}.", profiles_text(n)));
        true
    }

    /// The font files this program installed, off the machine; on Windows, out of the
    /// font registry too, where `oh-my-posh font install` listed them.
    fn remove_font_files(&self, entries: &[&Undo], log: &mut Vec<String>) -> bool {
        let files: Vec<PathBuf> = entries
            .iter()
            .filter_map(|u| u.target.split_once(':').map(|(_, p)| PathBuf::from(p)))
            .collect();
        if files.is_empty() {
            return true;
        }
        if self.os() == Os::Windows {
            if let Some(reg) = self.ws.find_program("reg") {
                let key = r"HKEY_CURRENT_USER\Software\Microsoft\Windows NT\CurrentVersion\Fonts";
                let listed = self.ask_all(&reg, &["query", key]).unwrap_or_default();
                for k in reg_parse(&listed) {
                    for (name, _, data) in &k.values {
                        let is_ours = files.iter().any(|f| {
                            f.file_name()
                                .is_some_and(|n| data.to_lowercase().ends_with(&n.to_string_lossy().to_lowercase()))
                        });
                        if is_ours {
                            self.reg_delete(&reg, key, name);
                        }
                    }
                }
            }
        }
        let mut n = 0;
        for f in &files {
            self.ws.remove_file(f);
            if !self.ws.exists(f) {
                n += 1;
            }
        }
        if self.os() == Os::Linux {
            if let Some(fc) = self.ws.find_program("fc-cache") {
                let _ = self.run(&fc, &["-f"]);
            }
        }
        log.push(format!(
            "{FONT_NAME}: {n} of {} font files this program installed were removed.",
            files.len()
        ));
        n == files.len()
    }

    // ---- The prompt in the shells ----

    /// Windows PowerShell ships unable to run any script, the profile included.
    fn allow_scripts(&self, program: &Path, log: &mut Vec<String>) {
        let policy = self
            .ask(
                program,
                &[
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Get-ExecutionPolicy",
                ],
            )
            .unwrap_or_default();
        if !matches!(policy.as_str(), "Restricted" | "AllSigned" | "Undefined") {
            return;
        }
        let r = self.run(
            program,
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force",
            ],
        );
        log.push(if r.success {
            "PowerShell now allows your own scripts (execution policy RemoteSigned for your account), so the profile loads.".into()
        } else {
            "PowerShell's execution policy blocks profiles; run `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser` yourself, or ask an administrator if a policy is set for the machine.".into()
        });
    }

    /// A login shell reads .bash_profile and not .bashrc: macOS's Terminal always opens
    /// one, and Git Bash on Windows and some Linux terminals do when a .bash_profile is
    /// there. So that file is made to load .bashrc, where the prompt line went.
    fn link_bash_profile(&self, log: &mut Vec<String>) {
        let path = self.home().join(".bash_profile");
        if !self.ws.exists(&path) && self.os() != Os::MacOs {
            return;
        }
        let text = self.ws.read_text(&path).unwrap_or_default();
        if text.contains(".bashrc") {
            return;
        }
        let new = with_block(&text, "#", "[ -f ~/.bashrc ] && . ~/.bashrc");
        if self.ws.write_text(&path, &new).is_ok() {
            log.push("Bash: .bash_profile now loads .bashrc, for terminals that open login shells.".into());
        }
    }

    fn apply(&self, theme: Option<&str>, shells: &[Shell], log: &mut Vec<String>) -> bool {
        let Some(binary) = self.find_binary() else {
            log.push("Install Oh My Posh first.".into());
            return false;
        };
        let theme_file = match theme {
            None => None,
            Some(name) => {
                let dir = self.find_themes_dir(Some(&binary));
                let found = dir
                    .as_deref()
                    .map(|d| self.themes_in(d))
                    .unwrap_or_default()
                    .into_iter()
                    .find(|t| t.name == name);
                match found {
                    Some(t) => Some(t.file),
                    None => {
                        log.push(format!("Theme \"{name}\" was not found."));
                        return false;
                    }
                }
            }
        };
        let clink = (self.os() == Os::Windows)
            .then(|| self.find_clink())
            .flatten();
        let os = self.os();
        let mut done = 0;
        for (shell, program) in self.shell_programs() {
            if !shells.contains(&shell) {
                continue;
            }
            let Some(program) = program else {
                continue;
            };
            let st = self.shell_status(shell, &program, clink.as_deref());
            if let Some(b) = st.blocker {
                log.push(format!("{}: {b}.", shell.name()));
                continue;
            }
            let existing = self.ws.read_text(&st.profile);
            if existing.is_none() && self.ws.exists(&st.profile) {
                log.push(format!(
                    "{}: {} is not a text file; left alone.",
                    shell.name(),
                    st.profile.display()
                ));
                continue;
            }
            let body = format!(
                "{} theme: {}\n{}",
                shell.comment(),
                theme.unwrap_or(DEFAULT_THEME),
                init_line(shell, os, &binary, theme_file.as_deref())
            );
            let mut text = with_block(existing.as_deref().unwrap_or(""), shell.comment(), &body);
            // A byte-order mark, so Windows PowerShell reads a new profile as UTF-8.
            if existing.is_none() && matches!(shell, Shell::Pwsh | Shell::WindowsPowerShell) {
                text.insert(0, '\u{feff}');
            }
            match self.ws.write_text(&st.profile, &text) {
                Ok(()) => {
                    done += 1;
                    log.push(format!(
                        "{}: prompt set in {}.",
                        shell.name(),
                        st.profile.display()
                    ));
                }
                Err(e) => {
                    log.push(format!(
                        "{}: could not write {}: {e}.",
                        shell.name(),
                        st.profile.display()
                    ));
                    continue;
                }
            }
            if os == Os::Windows && matches!(shell, Shell::Pwsh | Shell::WindowsPowerShell) {
                self.allow_scripts(&program, log);
            }
            if shell == Shell::Bash {
                self.link_bash_profile(log);
            }
        }
        if done > 0 {
            log.push("Open a new terminal window to see the prompt.".into());
        } else {
            log.push("No shell was set up.".into());
        }
        done > 0
    }

    fn remove(&self, log: &mut Vec<String>) -> bool {
        let clink = (self.os() == Os::Windows)
            .then(|| self.find_clink())
            .flatten();
        let mut files: Vec<(String, PathBuf, &str)> = self
            .shells(clink.as_deref())
            .into_iter()
            .map(|s| (s.shell.name().to_string(), s.profile, s.shell.comment()))
            .collect();
        files.push(("Bash".into(), self.home().join(".bash_profile"), "#"));
        let mut any = false;
        for (name, path, comment) in files {
            let Some(text) = self.ws.read_text(&path) else {
                continue;
            };
            let Some(new) = without_block(&text, comment) else {
                continue;
            };
            match self.ws.write_text(&path, &new) {
                Ok(()) => {
                    any = true;
                    log.push(format!("{name}: prompt removed from {}.", path.display()));
                }
                Err(e) => log.push(format!("{name}: could not write {}: {e}.", path.display())),
            }
        }
        // What `oh-my-posh init nu` wrote for Nushell to load at start-up.
        if let Some((_, Some(nu))) = self
            .shell_programs()
            .into_iter()
            .find(|(s, _)| *s == Shell::Nu)
        {
            let file = self.nu_autoload_dir(&nu).join(NU_AUTOLOAD_FILE);
            if self.ws.exists(&file) {
                self.ws.remove_file(&file);
                if !self.ws.exists(&file) {
                    any = true;
                    log.push(format!("Nushell: removed {}.", file.display()));
                }
            }
        }
        if !any {
            log.push("Nothing to remove: no shell was set up here.".into());
        }
        any
    }
}

/// Which program a record entry belongs to; a Konsole profile this program made counts
/// as Konsole's.
fn undo_kind(target: &str) -> &str {
    match target.split_once(':').map_or("", |(k, _)| k) {
        "konsole-file" => "konsole",
        k => k,
    }
}

/// A terminal that needs nothing done.
fn built_in(name: &str) -> TerminalStatus {
    TerminalStatus {
        name: name.into(),
        kind: TerminalKind::BuiltIn,
        target: String::new(),
        font_set: true,
        detail: "icons built in".into(),
    }
}

fn profiles_text(n: usize) -> String {
    if n == 1 {
        "1 profile".into()
    } else {
        format!("{n} profiles")
    }
}

/// The size at the end of an iTerm2 font string ("Menlo-Regular 13").
fn font_size_of(font: &str) -> &str {
    font.rsplit(' ')
        .next()
        .filter(|z| z.parse::<f64>().is_ok())
        .unwrap_or("12")
}

/// A terminal's gsettings: the list schema, its list and default keys, the profile
/// schema and the profile path root.
type GsettingsList = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);
const GNOME: GsettingsList = (
    "org.gnome.Terminal.ProfilesList",
    "list",
    "default",
    "org.gnome.Terminal.Legacy.Profile",
    "/org/gnome/terminal/legacy/profiles:/:",
);
const TILIX: GsettingsList = (
    "com.gexperts.Tilix.ProfilesList",
    "list",
    "default",
    "com.gexperts.Tilix.Profile",
    "/com/gexperts/Tilix/profiles/",
);
const MATE: GsettingsList = (
    "org.mate.terminal.global",
    "profile-list",
    "default-profile",
    "org.mate.terminal.profile",
    "/org/mate/terminal/profiles/",
);

/// A terminal whose font is one or two gsettings keys, for every window at once.
struct GsRecipe {
    name: &'static str,
    /// The program whose presence means the terminal is installed.
    program: &'static str,
    /// Schema, key and what to put there.
    settings: &'static [(&'static str, &'static str, Form)],
}

const GS_RECIPES: &[GsRecipe] = &[
    GsRecipe {
        name: "Ptyxis",
        program: "ptyxis",
        settings: &[
            ("org.gnome.Ptyxis", "use-system-font", Form::Literal("false")),
            ("org.gnome.Ptyxis", "font-name", Form::NameSize),
        ],
    },
    GsRecipe {
        name: "GNOME Console",
        program: "kgx",
        settings: &[
            ("org.gnome.Console", "use-system-font", Form::Literal("false")),
            ("org.gnome.Console", "custom-font", Form::NameSize),
        ],
    },
    GsRecipe {
        name: "Guake",
        program: "guake",
        settings: &[
            ("guake.general", "use-default-font", Form::Literal("false")),
            ("guake.style.font", "style", Form::NameSize),
        ],
    },
    GsRecipe {
        name: "elementary Terminal",
        program: "io.elementary.terminal",
        settings: &[("io.elementary.terminal.settings", "font", Form::NameSize)],
    },
];

fn gs_recipe(name: &str) -> Option<&'static GsRecipe> {
    GS_RECIPES.iter().find(|r| r.name == name)
}

// ---- Terminals with one settings file of a known shape ----

/// How a font is written in a setting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Form {
    /// `MesloLGM Nerd Font`.
    Name,
    /// `MesloLGM Nerd Font Mono`, for programs that want a fixed-pitch Windows font.
    Mono,
    /// `MesloLGM Nerd Font 11`: a Pango or GTK font description.
    NameSize,
    /// `MesloLGM Nerd Font:size=11`: a fontconfig pattern.
    Fontconfig,
    /// `"MesloLGM Nerd Font"`: a TOML or JSON string.
    Quoted,
    /// A fixed value that goes with the font, such as `false` for "use the system font".
    Literal(&'static str),
}

impl Form {
    fn text(self) -> String {
        match self {
            Form::Name => FONT_NAME.into(),
            Form::Mono => FONT_MONO.into(),
            Form::NameSize => format!("{FONT_NAME} 11"),
            Form::Fontconfig => format!("{FONT_NAME}:size=11"),
            Form::Quoted => format!("\"{FONT_NAME}\""),
            Form::Literal(v) => v.into(),
        }
    }

    /// Whether a value found in the settings is what this form asks for.
    fn is_set(self, value: &str) -> bool {
        match self {
            Form::Literal(v) => value.trim().eq_ignore_ascii_case(v),
            _ => is_nerd_font(&yaml_unquote(value)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    Ini,
    /// `Key=Value` lines with no sections (mintty, PuTTY's session files).
    Kv,
    Toml,
    Json,
}

impl Shape {
    /// The record's kind for entries of this shape.
    fn kind(self) -> &'static str {
        match self {
            Shape::Ini => "ini",
            Shape::Kv => "kv",
            Shape::Toml => "toml",
            Shape::Json => "json",
        }
    }
}

struct Recipe {
    name: &'static str,
    shape: Shape,
    /// The INI section, TOML table or JSON parent key; empty for the top level.
    section: &'static str,
    keys: &'static [(&'static str, Form)],
    /// Keys that would override the ones above, and so are taken out.
    removes: &'static [&'static str],
}

const RECIPES: &[Recipe] = &[
    Recipe {
        name: "LXTerminal",
        shape: Shape::Ini,
        section: "general",
        keys: &[("fontname", Form::NameSize)],
        removes: &[],
    },
    Recipe {
        name: "QTerminal",
        shape: Shape::Ini,
        section: "General",
        keys: &[("fontFamily", Form::Name)],
        removes: &["font"],
    },
    Recipe {
        name: "Sakura",
        shape: Shape::Ini,
        section: "sakura",
        keys: &[("font", Form::NameSize)],
        removes: &[],
    },
    Recipe {
        name: "Deepin Terminal",
        shape: Shape::Ini,
        section: "basic.interface.font",
        keys: &[("value", Form::Name)],
        removes: &[],
    },
    Recipe {
        name: "foot",
        shape: Shape::Ini,
        section: "main",
        keys: &[("font", Form::Fontconfig)],
        removes: &[],
    },
    Recipe {
        name: "Git Bash (mintty)",
        shape: Shape::Kv,
        section: "",
        keys: &[("Font", Form::Mono)],
        removes: &[],
    },
    Recipe {
        name: "MSYS2 (mintty)",
        shape: Shape::Kv,
        section: "",
        keys: &[("Font", Form::Mono)],
        removes: &[],
    },
    Recipe {
        name: "Cygwin (mintty)",
        shape: Shape::Kv,
        section: "",
        keys: &[("Font", Form::Mono)],
        removes: &[],
    },
    Recipe {
        name: "wsltty",
        shape: Shape::Kv,
        section: "",
        keys: &[("Font", Form::Mono)],
        removes: &[],
    },
    Recipe {
        name: "Rio",
        shape: Shape::Toml,
        section: "fonts",
        keys: &[("family", Form::Quoted)],
        removes: &[],
    },
    Recipe {
        name: "Warp",
        shape: Shape::Toml,
        section: "appearance.text",
        keys: &[("font_name", Form::Quoted)],
        removes: &[],
    },
    Recipe {
        name: "Zed",
        shape: Shape::Json,
        section: "terminal",
        keys: &[("font_family", Form::Quoted)],
        removes: &[],
    },
    Recipe {
        name: "Wave",
        shape: Shape::Json,
        section: "",
        keys: &[("term:fontfamily", Form::Quoted)],
        removes: &[],
    },
    Recipe {
        name: "Hyper (hyper.json)",
        shape: Shape::Json,
        section: "config",
        keys: &[("fontFamily", Form::Quoted)],
        removes: &[],
    },
];

fn recipe(name: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.name == name)
}

fn json_path<'a>(section: &'a str, key: &'a str) -> Vec<&'a str> {
    if section.is_empty() {
        vec![key]
    } else {
        vec![section, key]
    }
}

/// The record's key for a setting: `Section/Key`, or the key alone at the top level.
fn undo_key(section: &str, key: &str) -> String {
    if section.is_empty() {
        key.to_string()
    } else {
        format!("{section}/{key}")
    }
}

fn shape_get(shape: Shape, text: &str, section: &str, key: &str) -> Option<String> {
    match shape {
        Shape::Ini => ini_get(text, section, key),
        Shape::Kv => kv_get(text, key),
        Shape::Toml => toml_get(text, section, key),
        Shape::Json => jsonc::get_raw(text, &json_path(section, key)),
    }
}

/// The text with a setting put to `value`, or taken out for `None`.
fn shape_set(
    shape: Shape,
    text: &str,
    section: &str,
    key: &str,
    value: Option<&str>,
) -> Result<String, String> {
    Ok(match (shape, value) {
        (Shape::Ini, Some(v)) => ini_set(text, section, key, v),
        (Shape::Ini, None) => ini_remove(text, section, key),
        (Shape::Kv, v) => kv_set(text, key, v),
        (Shape::Toml, v) => toml_set(text, section, key, v),
        (Shape::Json, Some(v)) => jsonc::set_raw(text, &json_path(section, key), v)?,
        (Shape::Json, None) => jsonc::remove(text, &json_path(section, key))?,
    })
}

/// Whether a settings file already has everything a recipe asks for.
fn recipe_font_set(r: &Recipe, text: &str) -> bool {
    r.keys
        .iter()
        .all(|(k, f)| shape_get(r.shape, text, r.section, k).is_some_and(|v| f.is_set(&v)))
        && r
            .removes
            .iter()
            .all(|k| shape_get(r.shape, text, r.section, k).is_none())
}

/// foot's settings with the unnamed first section given its name, `[main]`, so that
/// it can be edited like any other.
fn foot_main(text: &str) -> String {
    let has_main = text.lines().any(|l| l.trim() == "[main]");
    let loose = text
        .lines()
        .take_while(|l| !l.trim_start().starts_with('['))
        .any(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        });
    if has_main || !loose {
        return text.to_string();
    }
    format!("[main]\n{text}")
}

/// The strings in a list as gsettings prints it: `['a', 'b']`, `@as []`, or one `'a'`.
fn gvariant_strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(s) = rest.find('\'') {
        let after = &rest[s + 1..];
        let Some(e) = after.find('\'') else {
            break;
        };
        out.push(after[..e].to_string());
        rest = &after[e + 1..];
    }
    out
}

// ---- Windows Terminal: every profile ----------------------------------------------------

/// Whether every profile uses the Nerd Font with the colour adjustment off, and how many
/// profiles have a font of their own (the defaults count as one).
fn wt_status(text: &str) -> (bool, usize) {
    let face = jsonc::get_string(text, &WT_FACE)
        .or_else(|| jsonc::get_string(text, &WT_LEGACY_FACE));
    let colours = jsonc::get_raw(text, &WT_COLOURS).is_some_and(|v| v == "\"never\"");
    let mut all = face.is_some_and(|f| is_nerd_font(&f)) && colours;
    let mut n = 1;
    for open in jsonc::objects_in(text, &WT_LIST) {
        let own = jsonc::get_string_at(text, open, &["font", "face"])
            .or_else(|| jsonc::get_string_at(text, open, &["fontFace"]));
        if let Some(f) = own {
            n += 1;
            all &= is_nerd_font(&f);
        }
    }
    (all, n)
}

/// The profile's name for the record: its guid, else its name.
fn wt_profile_id(text: &str, open: usize) -> Option<String> {
    jsonc::get_string_at(text, open, &["guid"])
        .or_else(|| jsonc::get_string_at(text, open, &["name"]))
}

/// The settings with the Nerd Font in the defaults and in every profile that has a font
/// of its own, the colour adjustment off, and each old value remembered.
fn wt_with_font(text: &str, target: &str, font: &str, changes: &mut Changes) -> Result<(String, usize), String> {
    let mut t = text.to_string();
    changes.remember(target, "defaults.font.face", jsonc::get_raw(&t, &WT_FACE));
    t = jsonc::set_string(&t, &WT_FACE, font)?;
    if let Some(old) = jsonc::get_raw(&t, &WT_LEGACY_FACE) {
        changes.remember(target, "defaults.fontFace", Some(old));
        t = jsonc::remove(&t, &WT_LEGACY_FACE)?;
    }
    changes.remember(
        target,
        "defaults.adjustIndistinguishableColors",
        jsonc::get_raw(&t, &WT_COLOURS),
    );
    t = jsonc::set_raw(&t, &WT_COLOURS, "\"never\"")?;
    let mut n = 1;
    // From the last profile to the first, so the earlier ones' positions stay right.
    for open in jsonc::objects_in(&t, &WT_LIST).into_iter().rev() {
        let Some(id) = wt_profile_id(&t, open) else {
            continue;
        };
        let face = jsonc::get_raw_at(&t, open, &["font", "face"]);
        let legacy = jsonc::get_raw_at(&t, open, &["fontFace"]);
        if face.is_none() && legacy.is_none() {
            continue; // follows the defaults
        }
        if let Some(l) = legacy {
            changes.remember(target, &format!("list:{id}:fontFace"), Some(l));
            t = jsonc::remove_at(&t, open, &["fontFace"])?;
        }
        changes.remember(target, &format!("list:{id}:font.face"), face);
        t = jsonc::set_raw_at(&t, open, &["font", "face"], &jsonc::quoted(font))?;
        n += 1;
    }
    Ok((t, n))
}

/// A member set back or taken out; a `font` object left empty goes with it.
fn json_put_back(text: &str, open: usize, path: &[&str], before: Option<&str>) -> Result<String, String> {
    match before {
        Some(raw) => jsonc::set_raw_at(text, open, path, raw),
        None => {
            let t = jsonc::remove_at(text, open, path)?;
            let parent = &path[..path.len() - 1];
            let empty = !parent.is_empty()
                && jsonc::get_raw_at(&t, open, parent)
                    .is_some_and(|raw| raw.trim_start_matches('{').trim_end_matches('}').trim().is_empty());
            if empty {
                jsonc::remove_at(&t, open, parent)
            } else {
                Ok(t)
            }
        }
    }
}

/// The settings with the record's values back in place.
fn wt_without_font(text: &str, list: &[&Undo]) -> Result<String, String> {
    let mut t = text.to_string();
    for u in list.iter().rev() {
        let before = u.before.as_deref();
        let root = jsonc_root(&t);
        t = match u.key.as_str() {
            "defaults.font.face" => json_put_back(&t, root, &WT_FACE, before)?,
            "defaults.fontFace" => json_put_back(&t, root, &WT_LEGACY_FACE, before)?,
            "defaults.adjustIndistinguishableColors" => json_put_back(&t, root, &WT_COLOURS, before)?,
            key => {
                let Some(rest) = key.strip_prefix("list:") else {
                    continue;
                };
                let Some((id, what)) = rest.rsplit_once(':') else {
                    continue;
                };
                let Some(open) = jsonc::objects_in(&t, &WT_LIST)
                    .into_iter()
                    .find(|o| wt_profile_id(&t, *o).as_deref() == Some(id))
                else {
                    continue;
                };
                match what {
                    "font.face" => json_put_back(&t, open, &["font", "face"], before)?,
                    "fontFace" => json_put_back(&t, open, &["fontFace"], before)?,
                    _ => continue,
                }
            }
        };
    }
    Ok(t)
}

/// The `{` of a settings file's object, or where one would start.
fn jsonc_root(text: &str) -> usize {
    text.find('{').unwrap_or(0)
}

// ---- The Windows registry, as reg.exe prints it -------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegKey {
    path: String,
    /// Name, type and data.
    values: Vec<(String, String, String)>,
}

/// The keys and values in `reg query` output: a key's path on a line of its own, each
/// value under it as name, type and data separated by four spaces.
fn reg_parse(text: &str) -> Vec<RegKey> {
    let mut out: Vec<RegKey> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.starts_with("HKEY_") {
            out.push(RegKey {
                path: line.to_string(),
                values: Vec::new(),
            });
            continue;
        }
        let Some(key) = out.last_mut() else {
            continue;
        };
        let parts: Vec<&str> = line
            .trim_start()
            .split("    ")
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() >= 2 && parts[1].starts_with("REG_") {
            key.values.push((
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts.get(2).map(|d| d.trim().to_string()).unwrap_or_default(),
            ));
        }
    }
    out
}

fn reg_value<'a>(key: &'a RegKey, name: &str) -> Option<(&'a str, &'a str)> {
    key.values
        .iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, ty, data)| (ty.as_str(), data.as_str()))
}

// ---- Settings files of other shapes ---------------------------------------------------

/// The value of a key in a section of an INI file (Konsole's profiles, konsolerc,
/// Xfce Terminal's terminalrc).
fn ini_get(text: &str, section: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l == format!("[{section}]");
            continue;
        }
        if inside {
            if let Some((k, v)) = l.split_once('=') {
                if k.trim() == key {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

/// The lines of a section: its header's index and where the next section starts.
fn ini_section(lines: &[String], section: &str) -> Option<(usize, usize)> {
    let header = format!("[{section}]");
    let start = lines.iter().position(|l| l.trim() == header)?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim().starts_with('['))
        .map_or(lines.len(), |p| start + 1 + p);
    Some((start, end))
}

/// The text with a key set in a section, the section made at the end if it is missing.
fn ini_set(text: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some((start, end)) = ini_section(&lines, section) else {
        if !lines.is_empty() && !lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!("[{section}]"));
        lines.push(format!("{key}={value}"));
        return lines.join("\n") + "\n";
    };
    for line in &mut lines[start + 1..end] {
        if line.split_once('=').is_some_and(|(k, _)| k.trim() == key) {
            *line = format!("{key}={value}");
            return lines.join("\n") + "\n";
        }
    }
    lines.insert(start + 1, format!("{key}={value}"));
    lines.join("\n") + "\n"
}

/// The text without a key in a section; a section left empty goes with it.
fn ini_remove(text: &str, section: &str, key: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some((start, end)) = ini_section(&lines, section) else {
        return text.to_string();
    };
    let Some(at) = lines[start + 1..end]
        .iter()
        .position(|l| l.split_once('=').is_some_and(|(k, _)| k.trim() == key))
    else {
        return text.to_string();
    };
    lines.remove(start + 1 + at);
    if lines[start + 1..end - 1].iter().all(|l| l.trim().is_empty()) {
        lines.drain(start..end - 1);
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

/// The value after a key on its own line (kitty.conf: `font_family Menlo`).
fn conf_get(text: &str, key: &str) -> Option<String> {
    text.lines().map(str::trim).find_map(|l| {
        let rest = l.strip_prefix(key)?;
        if rest.is_empty() {
            return Some(String::new());
        }
        rest.starts_with(char::is_whitespace)
            .then(|| rest.trim().to_string())
    })
}

/// The text with the key's line set, added at the end, or taken out for `None`.
fn conf_set(text: &str, key: &str, value: Option<&str>) -> String {
    let is_key = |l: &str| {
        let l = l.trim();
        l == key || l.strip_prefix(key).is_some_and(|r| r.starts_with(char::is_whitespace))
    };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let at = lines.iter().position(|l| is_key(l));
    match (at, value) {
        (Some(i), Some(v)) => lines[i] = format!("{key} {v}"),
        (Some(i), None) => {
            lines.remove(i);
        }
        (None, Some(v)) => lines.push(format!("{key} {v}")),
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

/// A top-level YAML section (`terminal:`) in a file's lines: the header's index, the
/// index past its last line, and the indent its own keys sit at.
fn yaml_section(lines: &[String], section: &str) -> Option<(usize, usize, usize)> {
    let start = lines.iter().position(|l| {
        let l = l.trim_end();
        l == format!("{section}:") || l.starts_with(&format!("{section}: "))
    })?;
    let mut end = start + 1;
    while end < lines.len() {
        let l = &lines[end];
        if l.trim().is_empty() || l.starts_with([' ', '\t']) || l.starts_with('#') {
            end += 1;
        } else {
            break;
        }
    }
    while end > start + 1 && (lines[end - 1].trim().is_empty() || lines[end - 1].starts_with('#')) {
        end -= 1;
    }
    let indent = lines[start + 1..end]
        .iter()
        .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map_or(2, |l| l.len() - l.trim_start().len());
    Some((start, end, indent))
}

/// The line in a section that holds `key:` at the section's own indent.
fn yaml_child(lines: &[String], (start, end, indent): (usize, usize, usize), key: &str) -> Option<usize> {
    (start + 1..end).find(|&i| {
        let l = &lines[i];
        l.len() - l.trim_start().len() == indent
            && l
                .trim_start()
                .strip_prefix(key)
                .and_then(|r| r.strip_prefix(':'))
                .is_some_and(|r| r.is_empty() || r.starts_with(char::is_whitespace))
    })
}

/// The value of `key` in a top-level section of a YAML file, as written (quotes kept).
fn yaml_get(text: &str, section: &str, key: &str) -> Option<String> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let at = yaml_child(&lines, yaml_section(&lines, section)?, key)?;
    let (_, value) = lines[at].split_once(':')?;
    Some(value.trim().to_string())
}

/// A YAML scalar without its quotes.
fn yaml_unquote(value: &str) -> String {
    let v = value.trim();
    for q in ['\'', '"'] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return v[1..v.len() - 1].to_string();
        }
    }
    v.to_string()
}

/// The YAML text with `key` in a top-level section set, added (the section too, if it
/// is missing or empty), or taken out for `None` (and the section with it, when nothing
/// else is left in it).
fn yaml_set(text: &str, section: &str, key: &str, value: Option<&str>) -> String {
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    match (yaml_section(&lines, section), value) {
        (Some(s), Some(v)) => {
            let (start, _, indent) = s;
            match yaml_child(&lines, s, key) {
                Some(i) => lines[i] = format!("{}{key}: {v}", " ".repeat(indent)),
                None => {
                    let header = lines[start].trim_end();
                    let inline = header[section.len() + 1..].trim();
                    if inline == "{}" {
                        lines[start] = format!("{section}:");
                    } else if !inline.is_empty() {
                        return text.to_string(); // a flow mapping with content: left alone
                    }
                    lines.insert(start + 1, format!("{}{key}: {v}", " ".repeat(indent)));
                }
            }
        }
        (Some(s), None) => {
            let (start, _, _) = s;
            let Some(i) = yaml_child(&lines, s, key) else {
                return text.to_string();
            };
            lines.remove(i);
            let left = yaml_section(&lines, section)
                .is_some_and(|(a, b, _)| lines[a + 1..b].iter().any(|l| !l.trim().is_empty()));
            if !left {
                lines.remove(start);
            }
        }
        (None, Some(v)) => {
            lines.push(format!("{section}:"));
            lines.push(format!("  {key}: {v}"));
        }
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join(nl) + nl
}

// ---- Key=Value files (mintty, PuTTY's session files) ----

fn kv_get(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_string())
    })
}

/// The text with a key's line set, added at the end, or taken out for `None`.
fn kv_set(text: &str, key: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let at = lines
        .iter()
        .position(|l| l.split_once('=').is_some_and(|(k, _)| k.trim() == key));
    match (at, value) {
        (Some(i), Some(v)) => lines[i] = format!("{key}={v}"),
        (Some(i), None) => {
            lines.remove(i);
        }
        (None, Some(v)) => lines.push(format!("{key}={v}")),
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

// ---- TOML tables (Rio, Warp) ----

fn toml_is_key(line: &str, key: &str) -> bool {
    let t = line.trim_start();
    !t.starts_with('#') && t.split_once('=').is_some_and(|(k, _)| k.trim() == key)
}

fn toml_value(line: &str) -> String {
    line.split_once('=')
        .map_or(String::new(), |(_, v)| v.trim().to_string())
}

/// A `[table]` header's index and where the next header starts.
fn toml_table(lines: &[String], table: &str) -> Option<(usize, usize)> {
    let start = lines.iter().position(|l| l.trim() == format!("[{table}]"))?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .map_or(lines.len(), |p| start + 1 + p);
    Some((start, end))
}

/// A `table.key = value` line before the first header.
fn toml_dotted(lines: &[String], table: &str, key: &str) -> Option<usize> {
    let first = lines
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .unwrap_or(lines.len());
    lines[..first]
        .iter()
        .position(|l| toml_is_key(l, &format!("{table}.{key}")))
}

/// The raw value of `key` in `[table]`, or of `table.key` at the top.
fn toml_get(text: &str, table: &str, key: &str) -> Option<String> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    if let Some(i) = toml_dotted(&lines, table, key) {
        return Some(toml_value(&lines[i]));
    }
    let (start, end) = toml_table(&lines, table)?;
    lines[start + 1..end]
        .iter()
        .find(|l| toml_is_key(l, key))
        .map(|l| toml_value(l))
}

/// The text with `key` in `[table]` set to a raw value, the table made at the end if
/// it is missing, or the key taken out for `None` (and the table, when left empty).
fn toml_set(text: &str, table: &str, key: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if let Some(i) = toml_dotted(&lines, table, key) {
        match value {
            Some(v) => lines[i] = format!("{table}.{key} = {v}"),
            None => {
                lines.remove(i);
            }
        }
    } else if let Some((start, end)) = toml_table(&lines, table) {
        let at = lines[start + 1..end]
            .iter()
            .position(|l| toml_is_key(l, key))
            .map(|p| start + 1 + p);
        match (at, value) {
            (Some(i), Some(v)) => lines[i] = format!("{key} = {v}"),
            (Some(i), None) => {
                lines.remove(i);
                if lines[start + 1..end - 1].iter().all(|l| l.trim().is_empty()) {
                    lines.drain(start..end - 1);
                    while lines.last().is_some_and(|l| l.trim().is_empty()) {
                        lines.pop();
                    }
                }
            }
            (None, Some(v)) => lines.insert(start + 1, format!("{key} = {v}")),
            (None, None) => return text.to_string(),
        }
    } else {
        let Some(v) = value else {
            return text.to_string();
        };
        if !lines.is_empty() && !lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!("[{table}]"));
        lines.push(format!("{key} = {v}"));
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

// ---- Hyper's .hyper.js ----

/// The font list written for Hyper: the Nerd Font first, then its own defaults.
const HYPER_FONTS: &str =
    "'\"MesloLGM Nerd Font\", Menlo, \"DejaVu Sans Mono\", Consolas, \"Lucida Console\", monospace'";

fn hyper_is_font_line(line: &str) -> bool {
    let t = line.trim_start();
    t.strip_prefix("fontFamily")
        .is_some_and(|r| r.trim_start().starts_with(':'))
}

/// The JavaScript expression after `fontFamily:`, without its trailing comma.
fn hyper_font(text: &str) -> Option<String> {
    let line = text.lines().find(|l| hyper_is_font_line(l))?;
    let (_, value) = line.split_once(':')?;
    Some(value.trim().trim_end_matches(',').trim().to_string())
}

/// Whether the first family in a `fontFamily` expression is a Nerd Font.
fn hyper_is_nerd(raw: &str) -> bool {
    yaml_unquote(raw)
        .split(',')
        .next()
        .is_some_and(|f| is_nerd_font(&yaml_unquote(f)))
}

/// The file with `fontFamily` set to a JavaScript expression, put first in `config`
/// when it is missing, or taken out for `None`.
fn hyper_with_font(text: &str, value: Option<&str>) -> Result<String, String> {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let indent_of = |l: &str| l.len() - l.trim_start().len();
    let at = lines.iter().position(|l| hyper_is_font_line(l));
    match (at, value) {
        (Some(i), Some(v)) => {
            let indent = indent_of(&lines[i]);
            lines[i] = format!("{}fontFamily: {v},", " ".repeat(indent));
        }
        (Some(i), None) => {
            lines.remove(i);
        }
        (None, Some(v)) => {
            let config = lines
                .iter()
                .position(|l| {
                    let t = l.trim_start();
                    t.strip_prefix("config")
                        .is_some_and(|r| r.trim_start().starts_with(':'))
                        && l.contains('{')
                })
                .ok_or_else(|| "no `config: {` found".to_string())?;
            let indent = indent_of(&lines[config]) + 2;
            lines.insert(config + 1, format!("{}fontFamily: {v},", " ".repeat(indent)));
        }
        (None, None) => return Ok(text.to_string()),
    }
    Ok(lines.join("\n") + "\n")
}

// ---- ConEmu.xml ----

const CONEMU_KEY: &str = r"HKEY_CURRENT_USER\Software\ConEmu";
const PUTTY_SESSIONS: &str = r"HKEY_CURRENT_USER\Software\SimonTatham\PuTTY\Sessions";

fn xml_attr(line: &str, name: &str) -> Option<String> {
    let k = line.find(&format!("{name}=\""))? + name.len() + 2;
    let end = line[k..].find('"')? + k;
    Some(
        line[k..end]
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&"),
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Every configuration in a ConEmu.xml (the keys under Software/ConEmu) with its
/// FontName, when it has one.
fn conemu_fonts(text: &str) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    let mut depth = 0;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("<key ") {
            depth += 1;
            if depth == 3 {
                if let Some(name) = xml_attr(t, "name") {
                    out.push((name, None));
                }
            }
            if t.ends_with("/>") {
                depth -= 1;
            }
        } else if t.starts_with("</key") {
            depth -= 1;
        } else if depth == 3
            && t.starts_with("<value ")
            && xml_attr(t, "name").as_deref() == Some("FontName")
        {
            if let Some(last) = out.last_mut() {
                last.1 = xml_attr(t, "data");
            }
        }
    }
    out
}

/// The file with one configuration's FontName set, added if it had none, or taken out
/// for `None`.
fn conemu_with_font(text: &str, config: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let (mut depth, mut inside, mut done) = (0, false, false);
    let is_font = |t: &str| t.starts_with("<value ") && xml_attr(t, "name").as_deref() == Some("FontName");
    for line in text.lines() {
        let t = line.trim();
        let indent = &line[..line.len() - line.trim_start().len()];
        if t.starts_with("<key ") {
            depth += 1;
            if depth == 3 && xml_attr(t, "name").as_deref() == Some(config) {
                inside = true;
            }
            lines.push(line.to_string());
            if t.ends_with("/>") {
                depth -= 1;
            }
        } else if t.starts_with("</key") {
            if depth == 3 && inside {
                if let (false, Some(v)) = (done, value) {
                    lines.push(format!(
                        "{indent}\t<value name=\"FontName\" type=\"string\" data=\"{}\"/>",
                        xml_escape(v)
                    ));
                }
                done = true;
                inside = false;
            }
            depth -= 1;
            lines.push(line.to_string());
        } else if inside && depth == 3 && !done && is_font(t) {
            done = true;
            if let Some(v) = value {
                lines.push(format!(
                    "{indent}<value name=\"FontName\" type=\"string\" data=\"{}\"/>",
                    xml_escape(v)
                ));
            }
        } else {
            lines.push(line.to_string());
        }
    }
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    lines.join(nl) + nl
}

// ---- Terminator's config: [profiles] with a [[name]] for each ----

/// The `[profiles]` block: its header's index and where the next section starts.
fn terminator_block(lines: &[String]) -> Option<(usize, usize)> {
    let start = lines.iter().position(|l| l.trim() == "[profiles]")?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| {
            let t = l.trim();
            t.starts_with('[') && !t.starts_with("[[")
        })
        .map_or(lines.len(), |p| start + 1 + p);
    Some((start, end))
}

/// Each profile's name and the index of its `[[name]]` line.
fn terminator_heads(lines: &[String]) -> Vec<(String, usize)> {
    let Some((start, end)) = terminator_block(lines) else {
        return Vec::new();
    };
    (start + 1..end)
        .filter_map(|i| {
            let name = lines[i].trim().strip_prefix("[[")?.strip_suffix("]]")?;
            Some((name.trim().to_string(), i))
        })
        .collect()
}

fn terminator_profiles(text: &str) -> Vec<String> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    terminator_heads(&lines).into_iter().map(|(n, _)| n).collect()
}

/// A profile's lines: its head's index and where the next head or the block ends.
fn terminator_range(lines: &[String], profile: &str) -> Option<(usize, usize)> {
    let (_, end) = terminator_block(lines)?;
    let heads = terminator_heads(lines);
    let at = heads.iter().position(|(n, _)| n == profile)?;
    let to = heads.get(at + 1).map_or(end, |(_, i)| *i);
    Some((heads[at].1, to))
}

fn terminator_key(lines: &[String], (from, to): (usize, usize), key: &str) -> Option<usize> {
    (from + 1..to).find(|&i| {
        let t = lines[i].trim();
        !t.starts_with('#') && t.split_once('=').is_some_and(|(k, _)| k.trim() == key)
    })
}

fn terminator_get(text: &str, profile: &str, key: &str) -> Option<String> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let i = terminator_key(&lines, terminator_range(&lines, profile)?, key)?;
    lines[i].split_once('=').map(|(_, v)| v.trim().to_string())
}

/// The text with a profile's key set (the profile and the block made if missing), or
/// taken out for `None` (and the profile and block with it, when left empty).
fn terminator_set(text: &str, profile: &str, key: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let indent_of = |l: &str| l.len() - l.trim_start().len();
    if terminator_range(&lines, profile).is_none() {
        if value.is_none() {
            return text.to_string();
        }
        match terminator_block(&lines) {
            Some((_, end)) => lines.insert(end, format!("  [[{profile}]]")),
            None => {
                if !lines.is_empty() && !lines.last().is_some_and(|l| l.trim().is_empty()) {
                    lines.push(String::new());
                }
                lines.push("[profiles]".into());
                lines.push(format!("  [[{profile}]]"));
            }
        }
    }
    let Some(range) = terminator_range(&lines, profile) else {
        return text.to_string();
    };
    match (terminator_key(&lines, range, key), value) {
        (Some(i), Some(v)) => {
            lines[i] = format!("{}{key} = {v}", " ".repeat(indent_of(&lines[i])));
        }
        (Some(i), None) => {
            lines.remove(i);
            let (from, to) = (range.0, range.1 - 1);
            if lines[from + 1..to].iter().all(|l| l.trim().is_empty()) {
                lines.drain(from..to);
                if let Some((bs, be)) = terminator_block(&lines) {
                    if lines[bs + 1..be].iter().all(|l| l.trim().is_empty()) {
                        lines.drain(bs..be);
                        while lines.last().is_some_and(|l| l.trim().is_empty()) {
                            lines.pop();
                        }
                    }
                }
            }
        }
        (None, Some(v)) => {
            let indent = (range.0 + 1..range.1)
                .find(|&i| !lines[i].trim().is_empty())
                .map_or(indent_of(&lines[range.0]) + 2, |i| indent_of(&lines[i]));
            lines.insert(range.0 + 1, format!("{}{key} = {v}", " ".repeat(indent)));
        }
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

// ---- X resources (xterm, urxvt) ----

fn xres_app(name: &str) -> &'static str {
    if name == "xterm" {
        "XTerm"
    } else {
        "URxvt"
    }
}

/// Whether a line sets `app*key` or `app.key`, by any path between and in any case.
fn xres_is(line: &str, app: &str, key: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with('!') || t.starts_with('#') {
        return false;
    }
    let Some((name, _)) = t.split_once(':') else {
        return false;
    };
    let name = name.trim().to_lowercase();
    let (app, key) = (app.to_lowercase(), key.to_lowercase());
    name.len() > app.len() + key.len()
        && name.starts_with(&app)
        && name.ends_with(&key)
        && name[app.len()..].starts_with(['*', '.'])
        && name[..name.len() - key.len()].ends_with(['*', '.'])
}

fn xres_get(text: &str, app: &str, key: &str) -> Option<String> {
    text.lines()
        .find(|l| xres_is(l, app, key))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_string())
}

/// The text with a resource set (its line kept, else `App*key` added at the end), or
/// taken out for `None`.
fn xres_set(text: &str, app: &str, key: &str, value: Option<&str>) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let at = lines.iter().position(|l| xres_is(l, app, key));
    match (at, value) {
        (Some(i), Some(v)) => {
            let name = lines[i].split_once(':').map_or("", |(n, _)| n).to_string();
            lines[i] = format!("{name}: {v}");
        }
        (Some(i), None) => {
            lines.remove(i);
        }
        (None, Some(v)) => lines.push(format!("{app}*{key}: {v}")),
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

fn toml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The quoted string after `family =` in a line, with where it sits.
fn toml_family_in(line: &str) -> Option<(usize, usize)> {
    let k = line.find("family")?;
    let after = &line[k + 6..];
    let eq = after.find('=')?;
    let rest = &after[eq + 1..];
    let q = rest.find('"')?;
    let start = k + 6 + eq + 1 + q + 1;
    let mut i = start;
    let b = line.as_bytes();
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some((start, i)),
            _ => i += 1,
        }
    }
    None
}

/// Where Alacritty's normal font family is set, in any of the three TOML spellings: the
/// line and the span of the family's text in it.
fn alacritty_family_span(text: &str) -> Option<(usize, usize, usize)> {
    let mut section = String::new();
    for (n, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.starts_with('[') {
            section = l.trim_matches(['[', ']']).trim().to_string();
            continue;
        }
        let spot = match section.as_str() {
            "font.normal" if l.starts_with("family") => toml_family_in(line),
            "font" if l.starts_with("normal") => toml_family_in(line),
            "" if l.starts_with("font.normal.family") => toml_family_in(line),
            _ => None,
        };
        if let Some((s, e)) = spot {
            return Some((n, s, e));
        }
    }
    None
}

fn alacritty_font(text: &str) -> Option<String> {
    let (n, s, e) = alacritty_family_span(text)?;
    let line = text.lines().nth(n)?;
    Some(line[s..e].replace("\\\"", "\"").replace("\\\\", "\\"))
}

/// Alacritty's settings with the normal font family set, or, for `None`, with the line
/// this program added taken out.
fn alacritty_with_font(text: &str, family: Option<&str>) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let span = alacritty_family_span(text);
    match (span, family) {
        (Some((n, s, e)), Some(f)) => {
            let line = lines[n].clone();
            lines[n] = format!("{}{}{}", &line[..s - 1], toml_quote(f), &line[e + 1..]);
        }
        (Some((n, _, _)), None) => {
            let own = format!("normal = {{ family = {} }}", toml_quote(FONT_NAME));
            if lines[n].trim() == own {
                lines.remove(n);
                // The [font] header this program added, if nothing else is under it.
                if let Some((start, end)) = ini_section(&lines, "font") {
                    if lines[start + 1..end].iter().all(|l| l.trim().is_empty()) {
                        lines.drain(start..end);
                        while lines.last().is_some_and(|l| l.trim().is_empty()) {
                            lines.pop();
                        }
                    }
                }
            }
        }
        (None, Some(f)) => {
            let own = format!("normal = {{ family = {} }}", toml_quote(f));
            match ini_section(&lines, "font") {
                Some((start, _)) => lines.insert(start + 1, own),
                None => {
                    if !lines.is_empty() && !lines.last().is_some_and(|l| l.trim().is_empty()) {
                        lines.push(String::new());
                    }
                    lines.push("[font]".into());
                    lines.push(own);
                }
            }
        }
        (None, None) => return text.to_string(),
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n") + "\n"
}

// ---- macOS: Terminal and iTerm2 -------------------------------------------------------

fn applescript_text(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Terminal's profile names: the keys of its "Window Settings" dictionary.
fn plist_profile_names(plist: &str) -> Vec<String> {
    let Some(w) = plist.find("<key>Window Settings</key>") else {
        return Vec::new();
    };
    let rest = &plist[w..];
    let Some(d) = rest.find("<dict>") else {
        return Vec::new();
    };
    let mut rest = &rest[d + 6..];
    let mut depth = 1;
    let mut out = Vec::new();
    while let Some(i) = rest.find('<') {
        rest = &rest[i..];
        let Some(j) = rest.find('>') else {
            break;
        };
        match &rest[1..j] {
            "dict" | "array" => depth += 1,
            "/dict" | "/array" => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            "key" if depth == 1 => {
                if let Some(e) = rest[j + 1..].find("</key>") {
                    out.push(rest[j + 1..j + 1 + e].to_string());
                }
            }
            _ => {}
        }
        rest = &rest[j + 1..];
    }
    out
}

/// One font named in an iTerm2 profile: the profile's guid, the key ("Normal Font" or
/// "Non Ascii Font"), the value, and where the value sits.
struct ItermFont {
    guid: String,
    key: String,
    value: String,
    start: usize,
    end: usize,
}

/// Every font iTerm2's profiles name, as `defaults export` prints them. A profile's
/// keys come sorted, so its Guid is the last one before its fonts.
fn iterm_font_entries(plist: &str) -> Vec<ItermFont> {
    let mut out = Vec::new();
    let mut from = 0;
    loop {
        let next = ["<key>Normal Font</key>", "<key>Non Ascii Font</key>"]
            .iter()
            .filter_map(|k| plist[from..].find(k).map(|i| (from + i, *k)))
            .min_by_key(|(i, _)| *i);
        let Some((i, key)) = next else {
            break;
        };
        let after = &plist[i + key.len()..];
        let (Some(s), Some(e)) = (after.find("<string>"), after.find("</string>")) else {
            break;
        };
        if e < s {
            break;
        }
        let start = i + key.len() + s + 8;
        let end = i + key.len() + e;
        let guid = plist[..i]
            .rfind("<key>Guid</key>")
            .and_then(|g| {
                let a = &plist[g..i];
                let s = a.find("<string>")? + 8;
                let e = a[s..].find("</string>")? + s;
                Some(a[s..e].to_string())
            })
            .unwrap_or_default();
        out.push(ItermFont {
            guid,
            key: key[5..key.len() - 6].to_string(),
            value: plist[start..end].to_string(),
            start,
            end,
        });
        from = end;
    }
    out
}

/// Every "Normal Font" iTerm2's profiles name.
fn iterm_fonts(plist: &str) -> Vec<String> {
    iterm_font_entries(plist)
        .into_iter()
        .filter(|f| f.key == "Normal Font")
        .map(|f| f.value)
        .collect()
}

/// The settings with each font replaced by what `pick` gives for it, and how many were.
fn iterm_with_fonts(plist: &str, pick: impl Fn(&ItermFont) -> Option<String>) -> (String, usize) {
    let mut out = String::new();
    let mut at = 0;
    let mut n = 0;
    for f in iterm_font_entries(plist) {
        if let Some(new) = pick(&f) {
            out.push_str(&plist[at..f.start]);
            out.push_str(&new);
            at = f.end;
            n += 1;
        }
    }
    out.push_str(&plist[at..]);
    (out, n)
}

/// The AppleScript that lists Terminal's profiles with their fonts, one per line,
/// name and font parted by a tab. The tab is made outside the `tell` block: inside
/// it, `tab` is Terminal's own window-tab class and comes out as the word "tab".
const APPLE_TERMINAL_LIST: &str = "set sep to character id 9\n\
                                   set nl to character id 10\n\
                                   set out to \"\"\n\
                                   tell application \"Terminal\"\n\
                                   repeat with s in settings sets\n\
                                   set out to out & (name of s) & sep & (font name of s) & nl\n\
                                   end repeat\n\
                                   end tell\n\
                                   return out";

/// JavaScript for osascript: asks AppKit, in a fresh process, to open the font by each
/// of its names, and prints one line per name ending in "found" or "missing".
const APPLE_FONT_PROBE: &str = "ObjC.import('AppKit');\n\
    var names = ['MesloLGMNFM-Regular', 'MesloLGM Nerd Font Mono', 'MesloLGMNerdFontMono-Regular', 'MesloLGMNF-Regular', 'MesloLGM Nerd Font', 'MesloLGMNerdFont-Regular'];\n\
    var out = [];\n\
    for (var i = 0; i < names.length; i++) {\n\
    var f = $.NSFont.fontWithNameSize(names[i], 12);\n\
    out.push(names[i] + ': ' + (f.isNil() ? 'missing' : 'opens as ' + ObjC.unwrap(f.fontName) + ', found'));\n\
    }\n\
    var all = ObjC.unwrap($.NSFontManager.sharedFontManager.availableFonts);\n\
    var meslo = [];\n\
    for (var j = 0; j < all.length; j++) { var n = ObjC.unwrap(all[j]); if (n.indexOf('Meslo') >= 0) meslo.push(n); }\n\
    out.push('available fonts with Meslo in the name: ' + (meslo.length ? meslo.join(', ') : 'none'));\n\
    out.join('\\n');";

/// Whether osascript was refused by macOS's Automation permission rather than by the
/// script: "Not authorized to send Apple events to Terminal. (-1743)".
fn apple_events_denied(r: &RunResult) -> bool {
    let text = format!("{}\n{}", r.stdout, r.stderr);
    text.contains("-1743") || text.contains("Not authorized to send Apple events")
}

/// Whether Terminal's profile has a Nerd Font: the font is an archived object in the
/// settings, but its name sits in it as plain text, so every run of plain text in it
/// is looked at.
fn apple_terminal_font_is_nerd(plist: &str, profile: &str) -> bool {
    let Some(w) = plist.find("<key>Window Settings</key>") else {
        return false;
    };
    let Some(p) = plist[w..].find(&format!("<key>{profile}</key>")) else {
        return false;
    };
    let from = &plist[w + p..];
    let Some(f) = from.find("<key>Font</key>") else {
        return false;
    };
    let after = &from[f..];
    let (Some(s), Some(e)) = (after.find("<data>"), after.find("</data>")) else {
        return false;
    };
    if e < s {
        return false;
    }
    let bytes = base64_decode(&after[s + 6..e]);
    String::from_utf8_lossy(&bytes)
        .split(|c: char| !c.is_ascii_graphic())
        .any(|run| run.len() >= 4 && is_nerd_font(run))
}

fn base64_decode(text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Mutex;

    /// A machine in memory: files, programs and canned answers.
    struct Fake {
        os: Os,
        home: PathBuf,
        files: Mutex<BTreeMap<PathBuf, String>>,
        programs: HashMap<String, PathBuf>,
        /// By program file name and either the whole argument line or the first argument.
        answers: HashMap<(String, String), String>,
        /// Files a program leaves behind, by its first argument.
        drops: Vec<(String, PathBuf)>,
        runs: Mutex<Vec<String>>,
        env: HashMap<String, String>,
        /// Answers given in turn to one whole argument line, the last one for good:
        /// a program whose answer changes as the machine is changed.
        turns: Mutex<HashMap<(String, String), Vec<String>>>,
    }

    impl Fake {
        fn new(os: Os, home: &str) -> Self {
            Fake {
                os,
                home: PathBuf::from(home),
                files: Mutex::default(),
                turns: Mutex::default(),
                programs: HashMap::new(),
                answers: HashMap::new(),
                drops: Vec::new(),
                runs: Mutex::default(),
                env: HashMap::new(),
            }
        }
        fn file(&self, path: &str, text: &str) {
            self.files
                .lock()
                .unwrap()
                .insert(PathBuf::from(path), text.into());
        }
        fn text(&self, path: &str) -> String {
            self.files
                .lock()
                .unwrap()
                .get(&PathBuf::from(path))
                .cloned()
                .unwrap_or_default()
        }
        fn runs(&self) -> Vec<String> {
            self.runs.lock().unwrap().clone()
        }
        fn program(&mut self, name: &str, path: &str) {
            self.programs.insert(name.into(), path.into());
            self.file(path, "");
        }
        fn answer(&mut self, program: &str, arg: &str, out: &str) {
            self.answers
                .insert((program.into(), arg.into()), out.into());
        }
        fn answer_in_turn(&self, program: &str, args: &str, outs: &[&str]) {
            self.turns.lock().unwrap().insert(
                (program.into(), args.into()),
                outs.iter().map(|s| s.to_string()).collect(),
            );
        }
    }

    impl Workstation for Fake {
        fn os(&self) -> Os {
            self.os
        }
        fn home_dir(&self) -> Option<PathBuf> {
            Some(self.home.clone())
        }
        fn temp_dir(&self) -> PathBuf {
            PathBuf::from("/tmp")
        }
        fn app_dir(&self) -> PathBuf {
            self.home.join(".idlewalker")
        }
        fn env(&self, name: &str) -> Option<String> {
            self.env.get(name).cloned()
        }
        fn find_program(&self, name: &str) -> Option<PathBuf> {
            // As a real PATH search: a program that was deleted is no longer found.
            self.programs
                .get(name)
                .filter(|p| self.files.lock().unwrap().contains_key(*p))
                .cloned()
        }
        fn run(&self, program: &str, args: &[&str]) -> RunResult {
            self.runs
                .lock()
                .unwrap()
                .push(format!("{program} {}", args.join(" ")));
            let name = Path::new(program)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            for (arg, path) in &self.drops {
                if args.first() == Some(&arg.as_str()) {
                    self.files.lock().unwrap().insert(path.clone(), String::new());
                }
            }
            let whole = (name.clone(), args.join(" "));
            if let Some(outs) = self.turns.lock().unwrap().get_mut(&whole) {
                let out = if outs.len() > 1 { outs.remove(0) } else { outs[0].clone() };
                return RunResult {
                    success: true,
                    stdout: out,
                    stderr: String::new(),
                };
            }
            let first = (name, args.first().map_or("", |a| a).to_string());
            match self.answers.get(&whole).or_else(|| self.answers.get(&first)) {
                Some(out) => RunResult {
                    success: true,
                    stdout: out.clone(),
                    stderr: String::new(),
                },
                None => RunResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "no such program".into(),
                },
            }
        }
        fn read_text(&self, path: &Path) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
        fn write_text(&self, path: &Path, text: &str) -> Result<(), String> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), text.into());
            Ok(())
        }
        fn exists(&self, path: &Path) -> bool {
            let files = self.files.lock().unwrap();
            files.contains_key(path) || files.keys().any(|k| k.starts_with(path))
        }
        fn make_dir(&self, _path: &Path) -> Result<(), String> {
            Ok(())
        }
        fn remove_file(&self, path: &Path) {
            self.files.lock().unwrap().remove(path);
        }
        fn remove_dir(&self, path: &Path) -> Result<(), String> {
            self.files
                .lock()
                .unwrap()
                .retain(|k, _| !k.starts_with(path));
            Ok(())
        }
        fn list_files(&self, dir: &Path, _depth: u32) -> Vec<PathBuf> {
            self.files
                .lock()
                .unwrap()
                .keys()
                .filter(|k| k.parent() == Some(dir))
                .cloned()
                .collect()
        }
        fn final_address(&self, _url: &str) -> Option<String> {
            Some(format!("{RELEASES}/tag/v99.1.0"))
        }
    }

    fn linux_box() -> Fake {
        let mut f = Fake::new(Os::Linux, "/home/ann");
        f.program("oh-my-posh", "/home/ann/.local/bin/oh-my-posh");
        f.answer("oh-my-posh", "version", "26.9.0\n");
        f.answer("oh-my-posh", "cache", "/home/ann/.cache/oh-my-posh\n");
        f.file("/bin/bash", "");
        f.file("/usr/bin/zsh", "");
        f.file(
            "/home/ann/.cache/oh-my-posh/themes/jandedobbeleer.omp.json",
            "{}",
        );
        f.file("/home/ann/.cache/oh-my-posh/themes/atomic.omp.json", "{}");
        f.file("/home/ann/.bashrc", "export FOO=1\n");
        f
    }

    #[test]
    fn each_shell_gets_its_own_line() {
        let b = Path::new("/usr/local/bin/oh-my-posh");
        let t = Path::new("/home/ann/.cache/oh-my-posh/themes/x.omp.json");
        assert_eq!(
            init_line(Shell::Bash, Os::Linux, b, Some(t)),
            "eval \"$('/usr/local/bin/oh-my-posh' init bash --config '/home/ann/.cache/oh-my-posh/themes/x.omp.json')\""
        );
        assert_eq!(
            init_line(Shell::Zsh, Os::MacOs, b, None),
            "eval \"$('/usr/local/bin/oh-my-posh' init zsh)\""
        );
        assert_eq!(
            init_line(Shell::Fish, Os::Linux, b, Some(t)),
            "'/usr/local/bin/oh-my-posh' init fish --config '/home/ann/.cache/oh-my-posh/themes/x.omp.json' | source"
        );
        assert_eq!(
            init_line(Shell::Nu, Os::Linux, b, Some(t)),
            "^\"/usr/local/bin/oh-my-posh\" init nu --config \"/home/ann/.cache/oh-my-posh/themes/x.omp.json\""
        );
    }

    #[test]
    fn windows_paths_suit_each_shell() {
        let b = Path::new(r"C:\Users\Ann Lee\AppData\Local\Microsoft\WindowsApps\oh-my-posh.exe");
        let t = Path::new(r"C:\Users\Ann Lee\themes\x.omp.json");
        assert_eq!(
            init_line(Shell::Pwsh, Os::Windows, b, Some(t)),
            r"& 'C:\Users\Ann Lee\AppData\Local\Microsoft\WindowsApps\oh-my-posh.exe' init pwsh --config 'C:\Users\Ann Lee\themes\x.omp.json' | Invoke-Expression"
        );
        assert_eq!(
            init_line(Shell::Bash, Os::Windows, b, Some(t)),
            "eval \"$('C:/Users/Ann Lee/AppData/Local/Microsoft/WindowsApps/oh-my-posh.exe' init bash --config 'C:/Users/Ann Lee/themes/x.omp.json')\""
        );
        // A path with a space is wrapped twice for cmd; one without is not quoted at all.
        assert_eq!(
            init_line(Shell::Cmd, Os::Windows, b, Some(t)),
            "load(io.popen('\"\"C:/Users/Ann Lee/AppData/Local/Microsoft/WindowsApps/oh-my-posh.exe\" init cmd --config \"C:/Users/Ann Lee/themes/x.omp.json\"\"'):read(\"*a\"))()"
        );
        let plain = Path::new(r"C:\omp\oh-my-posh.exe");
        assert_eq!(
            init_line(Shell::Cmd, Os::Windows, plain, None),
            "load(io.popen('C:/omp/oh-my-posh.exe init cmd'):read(\"*a\"))()"
        );
    }

    #[test]
    fn a_block_is_replaced_not_repeated_and_comes_out_clean() {
        let original = "export FOO=1\n";
        let once = with_block(original, "#", "# theme: a\nline one");
        let twice = with_block(&once, "#", "# theme: b\nline two");
        assert_eq!(twice.matches(BEGIN).count(), 1);
        assert!(!twice.contains("line one"));
        assert_eq!(block_body(&twice, "#"), Some("# theme: b\nline two"));
        assert_eq!(
            theme_in(block_body(&twice, "#").unwrap(), "#").as_deref(),
            Some("b")
        );
        assert_eq!(without_block(&twice, "#").as_deref(), Some(original));
        assert_eq!(without_block(original, "#"), None);
        // A block in the middle of a file leaves what follows alone.
        let middle = format!("{twice}alias x=y\n");
        assert_eq!(
            without_block(&middle, "#").as_deref(),
            Some("export FOO=1\nalias x=y\n")
        );
        // A file without a final newline gets one before the block.
        assert!(with_block("x", "#", "l").starts_with("x\n\n# "));
    }

    #[test]
    fn versions_compare_as_numbers() {
        assert!(is_newer("26.10.0", "26.9.0"));
        assert!(!is_newer("26.9.0", "26.9.0"));
        assert!(!is_newer("v26.9.0", "26.9.1"));
        assert!(!is_newer("nonsense", "1.0.0"));
        assert_eq!(parse_version("v1.2.3-beta"), Some((1, 2, 3)));
    }

    #[test]
    fn nerd_fonts_are_known_by_their_names() {
        for f in [
            "MesloLGM Nerd Font",
            "MesloLGM Nerd Font Mono",
            "MesloLGM NF",
            "FiraCode NFM",
            "MesloLGMNerdFontMono-Regular 11",
            // The PostScript names Nerd Fonts give since version 3.
            "MesloLGMNFM-Regular",
            "MesloLGMNF-Bold",
            "MesloLGMNFP-Italic",
            "MesloLGMNFM-Regular 13",
            "FiraCodeNFM-Reg",
        ] {
            assert!(is_nerd_font(f), "{f}");
        }
        for f in [
            "Consolas",
            "Cascadia Mono",
            "Fun Font",
            "Menlo-Regular 13",
            "SFMonoTerminal-Regular",
            "AndaleMono",
            "NF",
        ] {
            assert!(!is_nerd_font(f), "{f}");
        }
    }

    #[test]
    fn the_record_round_trips_with_awkward_values() {
        let list = vec![
            Undo {
                target: "vscode:C:\\Users\\ann\\settings.json".into(),
                key: VSCODE_FONT.into(),
                before: Some("\"Consolas\"".into()),
            },
            Undo {
                target: "kitty:/home/ann/.config/kitty/kitty.conf".into(),
                key: "font_family".into(),
                before: None,
            },
            Undo {
                target: "terminal.app:Odd\tname".into(),
                key: "font name".into(),
                before: Some("a\nb\\c".into()),
            },
        ];
        assert_eq!(undo_parse(&undo_text(&list)), list);
        assert!(undo_parse("# only a comment\n\n").is_empty());
    }

    #[test]
    fn status_reads_the_machine() {
        let posh = Posh::new(Arc::new(linux_box()));
        let s = posh.status();
        assert_eq!(s.version.as_deref(), Some("26.9.0"));
        assert_eq!(
            s.themes_dir.as_deref(),
            Some(Path::new("/home/ann/.cache/oh-my-posh/themes"))
        );
        assert_eq!(
            s.themes.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            ["atomic", "jandedobbeleer"]
        );
        let shells: Vec<Shell> = s.shells.iter().map(|s| s.shell).collect();
        assert_eq!(shells, [Shell::Bash, Shell::Zsh]);
        assert!(s.shells.iter().all(|s| !s.configured));
        assert!(!s.font_installed);
        assert!(!s.undo_available);
        assert_eq!(posh.latest().as_deref(), Some("99.1.0"));
    }

    #[test]
    fn apply_sets_the_ticked_shells_and_remove_undoes_it() {
        let fake = Arc::new(linux_box());
        let posh = Posh::new(fake.clone());
        let done = posh.perform(&Task::Apply {
            theme: Some("atomic".into()),
            shells: vec![Shell::Bash, Shell::Zsh],
        });
        assert!(done.ok, "{:?}", done.log);
        let bashrc = fake.text("/home/ann/.bashrc");
        assert!(bashrc.starts_with("export FOO=1\n\n# >>> IdleWalker"));
        assert!(bashrc
            .contains("init bash --config '/home/ann/.cache/oh-my-posh/themes/atomic.omp.json'"));
        assert!(fake.text("/home/ann/.zshrc").contains("init zsh --config"));
        let st = &done.status.shells;
        assert!(st
            .iter()
            .all(|s| s.configured && s.theme.as_deref() == Some("atomic")));
        // No .bash_profile on Linux: none is made.
        assert!(!fake.exists(Path::new("/home/ann/.bash_profile")));

        // The default theme has no --config.
        let done = posh.perform(&Task::Apply {
            theme: None,
            shells: vec![Shell::Bash],
        });
        let bashrc = fake.text("/home/ann/.bashrc");
        assert!(!bashrc.contains("--config"));
        assert_eq!(bashrc.matches(BEGIN).count(), 1);
        assert_eq!(done.status.shells[0].theme.as_deref(), Some(DEFAULT_THEME));

        let done = posh.perform(&Task::Remove);
        assert!(done.ok);
        assert_eq!(fake.text("/home/ann/.bashrc"), "export FOO=1\n");
        assert_eq!(fake.text("/home/ann/.zshrc"), "");
        assert!(done.status.shells.iter().all(|s| !s.configured));
    }

    #[test]
    fn a_bash_profile_that_hides_bashrc_is_made_to_load_it() {
        let fake = Arc::new(linux_box());
        fake.file("/home/ann/.bash_profile", "export PATH=$PATH:~/bin\n");
        let posh = Posh::new(fake.clone());
        let done = posh.perform(&Task::Apply {
            theme: None,
            shells: vec![Shell::Bash],
        });
        assert!(done.ok, "{:?}", done.log);
        let profile = fake.text("/home/ann/.bash_profile");
        assert!(profile.contains("[ -f ~/.bashrc ] && . ~/.bashrc"), "{profile}");
        posh.perform(&Task::Remove);
        assert_eq!(fake.text("/home/ann/.bash_profile"), "export PATH=$PATH:~/bin\n");
    }

    #[test]
    fn nushell_is_asked_its_version_and_its_autoload_file_is_removed() {
        let mut f = linux_box();
        f.program("nu", "/usr/bin/nu");
        f.answer("nu", "--version", "0.103.0\n");
        f.answer("nu", "-c $nu.config-path", "/home/ann/.config/nushell/config.nu\n");
        f.answer(
            "nu",
            "-c $nu.data-dir | path join vendor autoload",
            "/home/ann/.local/share/nushell/vendor/autoload\n",
        );
        f.file("/home/ann/.local/share/nushell/vendor/autoload/oh-my-posh.nu", "# omp");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        let nu = s.shells.iter().find(|s| s.shell == Shell::Nu).unwrap();
        assert!(nu.blocker.as_deref().unwrap().contains("0.104.0"), "{:?}", nu.blocker);
        assert_eq!(
            nu.profile,
            PathBuf::from("/home/ann/.config/nushell/config.nu")
        );
        let done = posh.perform(&Task::Remove);
        assert!(done.ok, "{:?}", done.log);
        assert!(!fake.exists(Path::new(
            "/home/ann/.local/share/nushell/vendor/autoload/oh-my-posh.nu"
        )));
    }

    #[test]
    fn uninstall_clears_the_shells_the_binary_and_the_themes() {
        let fake = Arc::new(linux_box());
        let posh = Posh::new(fake.clone());
        posh.perform(&Task::Apply {
            theme: Some("atomic".into()),
            shells: vec![Shell::Bash],
        });
        let done = posh.perform(&Task::Uninstall);
        assert!(done.ok, "{:?}", done.log);
        assert_eq!(fake.text("/home/ann/.bashrc"), "export FOO=1\n");
        assert!(!fake.exists(Path::new("/home/ann/.local/bin/oh-my-posh")));
        assert!(!fake.exists(Path::new("/home/ann/.cache/oh-my-posh")));
        assert!(done
            .log
            .iter()
            .any(|l| l.starts_with("Deleted /home/ann/.local/bin/oh-my-posh")));
        assert!(done
            .log
            .iter()
            .any(|l| l.starts_with("Removed the cache, init scripts and themes")));
        assert!(done.status.themes.is_empty());

        // Nothing installed: nothing to do, and no complaint.
        let bare = Posh::new(Arc::new(Fake::new(Os::Linux, "/home/ann")));
        assert!(bare.perform(&Task::Uninstall).ok);
    }

    #[test]
    fn ini_keys_are_read_set_added_and_removed() {
        let rc = "[Desktop Entry]\nDefaultProfile=Work.profile\n\n[General]\nx=1\n";
        assert_eq!(
            ini_get(rc, "Desktop Entry", "DefaultProfile").as_deref(),
            Some("Work.profile")
        );
        assert_eq!(ini_get(rc, "General", "DefaultProfile"), None);
        let set = ini_set(rc, "Desktop Entry", "DefaultProfile", "Other.profile");
        assert_eq!(
            set,
            "[Desktop Entry]\nDefaultProfile=Other.profile\n\n[General]\nx=1\n"
        );
        let added = ini_set(rc, "General", "y", "2");
        assert_eq!(
            added,
            "[Desktop Entry]\nDefaultProfile=Work.profile\n\n[General]\ny=2\nx=1\n"
        );
        let new_section = ini_set("[General]\nName=a\n", "Appearance", "Font", "F,11");
        assert_eq!(
            new_section,
            "[General]\nName=a\n\n[Appearance]\nFont=F,11\n"
        );
        assert_eq!(ini_set("", "A", "k", "v"), "[A]\nk=v\n");
        // Removing the only key in a section takes the section away too.
        assert_eq!(
            ini_remove(&new_section, "Appearance", "Font"),
            "[General]\nName=a\n"
        );
        assert_eq!(ini_remove(&added, "General", "y"), rc);
        assert_eq!(ini_remove(rc, "General", "missing"), rc);
    }

    #[test]
    fn kitty_and_alacritty_and_xfce_files_are_edited_and_put_back() {
        let kitty = "# fonts\nfont_size 12\nfont_family Menlo\n";
        assert_eq!(conf_get(kitty, "font_family").as_deref(), Some("Menlo"));
        assert_eq!(conf_get(kitty, "font_size").as_deref(), Some("12"));
        let set = conf_set(kitty, "font_family", Some(FONT_NAME));
        assert_eq!(set, "# fonts\nfont_size 12\nfont_family MesloLGM Nerd Font\n");
        assert_eq!(conf_set(&set, "font_family", Some("Menlo")), kitty);
        let added = conf_set("font_size 12\n", "font_family", Some("X"));
        assert_eq!(added, "font_size 12\nfont_family X\n");
        assert_eq!(conf_set(&added, "font_family", None), "font_size 12\n");

        // Alacritty, in each of its spellings.
        let inline = "[font]\nnormal = { family = \"Menlo\", style = \"Regular\" }\nsize = 11\n";
        assert_eq!(alacritty_font(inline).as_deref(), Some("Menlo"));
        let set = alacritty_with_font(inline, Some(FONT_NAME));
        assert_eq!(
            set,
            "[font]\nnormal = { family = \"MesloLGM Nerd Font\", style = \"Regular\" }\nsize = 11\n"
        );
        assert_eq!(alacritty_with_font(&set, Some("Menlo")), inline);
        let table = "[font.normal]\nfamily = \"Menlo\"\n";
        assert_eq!(alacritty_font(table).as_deref(), Some("Menlo"));
        assert!(alacritty_with_font(table, Some("X")).contains("family = \"X\""));
        let dotted = "font.normal.family = \"Menlo\"\n[window]\nopacity = 0.9\n";
        assert_eq!(alacritty_font(dotted).as_deref(), Some("Menlo"));
        assert!(alacritty_with_font(dotted, Some("X")).starts_with("font.normal.family = \"X\"\n"));
        // Nothing set yet: a [font] table is added, and taken away again.
        let bare = "[window]\nopacity = 0.9\n";
        let added = alacritty_with_font(bare, Some(FONT_NAME));
        assert_eq!(
            added,
            "[window]\nopacity = 0.9\n\n[font]\nnormal = { family = \"MesloLGM Nerd Font\" }\n"
        );
        assert_eq!(alacritty_font(&added).as_deref(), Some(FONT_NAME));
        assert_eq!(alacritty_with_font(&added, None), bare);
        assert_eq!(alacritty_with_font("", Some("X")), "[font]\nnormal = { family = \"X\" }\n");

        // Xfce Terminal's keys.
        let rc = "[Configuration]\nMiscAlwaysShowTabs=FALSE\nFontUseSystem=TRUE\n";
        let set = ini_set(&ini_set(rc, "Configuration", "FontName", "F 11"), "Configuration", "FontUseSystem", "FALSE");
        assert_eq!(ini_get(&set, "Configuration", "FontName").as_deref(), Some("F 11"));
        assert_eq!(ini_get(&set, "Configuration", "FontUseSystem").as_deref(), Some("FALSE"));
    }

    #[test]
    fn tabby_yaml_is_edited_and_put_back() {
        let yaml = "version: 1\nterminal:\n  fontSize: 14\n  font: Consolas\n  colorScheme:\n    name: x\n    font: nope\n\nhotkeys:\n  copy:\n    - Ctrl-C\n";
        assert_eq!(yaml_get(yaml, "terminal", "font").as_deref(), Some("Consolas"));
        assert_eq!(yaml_get(yaml, "hotkeys", "font"), None);
        let set = yaml_set(yaml, "terminal", "font", Some(FONT_NAME));
        assert_eq!(set, yaml.replace("font: Consolas", "font: MesloLGM Nerd Font"));
        assert_eq!(yaml_set(&set, "terminal", "font", Some("Consolas")), yaml);
        assert_eq!(yaml_unquote("'Fira Code'"), "Fira Code");
        assert_eq!(yaml_unquote("\"Fira Code\""), "Fira Code");
        assert_eq!(yaml_unquote("Fira Code"), "Fira Code");
        // No font yet: it goes first in the section, and leaves it again.
        let bare = "version: 1\nterminal:\n  fontSize: 14\nhotkeys: {}\n";
        let added = yaml_set(bare, "terminal", "font", Some("X"));
        assert_eq!(added, "version: 1\nterminal:\n  font: X\n  fontSize: 14\nhotkeys: {}\n");
        assert_eq!(yaml_set(&added, "terminal", "font", None), bare);
        // No section: added at the end, and taken away with it.
        let none = "version: 1\n";
        let added = yaml_set(none, "terminal", "font", Some("X"));
        assert_eq!(added, "version: 1\nterminal:\n  font: X\n");
        assert_eq!(yaml_set(&added, "terminal", "font", None), none);
        // An empty section as js-yaml writes it, and an empty file.
        assert_eq!(yaml_set("terminal: {}\n", "terminal", "font", Some("X")), "terminal:\n  font: X\n");
        assert_eq!(yaml_set("", "terminal", "font", Some("X")), "terminal:\n  font: X\n");
        // Four-space indents and CRLF are kept.
        let wide = "terminal:\r\n    fontSize: 14\r\n";
        assert_eq!(
            yaml_set(wide, "terminal", "font", Some("X")),
            "terminal:\r\n    font: X\r\n    fontSize: 14\r\n"
        );
    }

    #[test]
    fn tabby_is_found_by_its_settings_file_and_put_back() {
        let f = Fake::new(Os::Linux, "/home/ann");
        f.file("/home/ann/.config/tabby/config.yaml", "version: 1\nterminal:\n  font: 'Fira Code'\n  fontSize: 14\n");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        assert_eq!(s.terminals.len(), 1);
        assert_eq!(s.terminals[0].name, "Tabby");
        assert!(!s.terminals[0].font_set);

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        assert_eq!(
            fake.text("/home/ann/.config/tabby/config.yaml"),
            "version: 1\nterminal:\n  font: MesloLGM Nerd Font\n  fontSize: 14\n"
        );
        assert!(posh.status().terminals[0].font_set);
        let undo = fake.text("/home/ann/.idlewalker/terminal-undo.txt");
        assert!(undo.contains("tabby:") && undo.contains("\tterminal.font\t='Fira Code'\n"), "{undo}");

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        assert_eq!(
            fake.text("/home/ann/.config/tabby/config.yaml"),
            "version: 1\nterminal:\n  font: 'Fira Code'\n  fontSize: 14\n"
        );
    }

    /// On Windows the program is found by its installer's folder, and the settings
    /// folder follows TABBY_CONFIG_DIRECTORY when it is set. Windows paths are joined
    /// with backslashes only on a Windows host, so this runs there alone.
    #[cfg(windows)]
    #[test]
    fn windows_tabby_is_found_by_its_installer_and_its_settings_variable() {
        let mut f = Fake::new(Os::Windows, r"C:\Users\ann");
        f.env.insert("LOCALAPPDATA".into(), r"C:\Users\ann\AppData\Local".into());
        f.env.insert("TABBY_CONFIG_DIRECTORY".into(), r"D:\tabby-data".into());
        f.file(r"C:\Users\ann\AppData\Local\Programs\Tabby\Tabby.exe", "");
        let posh = Posh::new(Arc::new(f));
        let s = posh.status();
        let tabby = s.terminals.iter().find(|t| t.name == "Tabby").expect("Tabby listed");
        assert_eq!(tabby.target, r"D:\tabby-data\config.yaml");
    }

    #[test]
    fn plain_settings_files_of_each_shape_are_edited_and_put_back() {
        // Key=Value (mintty).
        let rc = "BoldAsFont=no\nFont=Consolas\n";
        assert_eq!(kv_get(rc, "Font").as_deref(), Some("Consolas"));
        let set = kv_set(rc, "Font", Some(FONT_MONO));
        assert_eq!(set, "BoldAsFont=no\nFont=MesloLGM Nerd Font Mono\n");
        assert_eq!(kv_set(&set, "Font", Some("Consolas")), rc);
        assert_eq!(kv_set("", "Font", Some("X")), "Font=X\n");
        assert_eq!(kv_set("Font=X\n", "Font", None), "");

        // TOML (Rio, Warp).
        let rio = "[window]\nopacity = 0.9\n\n[fonts]\nsize = 14.0\n\n[fonts.regular]\nfamily = \"cascadiacode\"\n";
        assert_eq!(toml_get(rio, "fonts", "family"), None);
        let set = toml_set(rio, "fonts", "family", Some("\"X\""));
        assert_eq!(
            set,
            "[window]\nopacity = 0.9\n\n[fonts]\nfamily = \"X\"\nsize = 14.0\n\n[fonts.regular]\nfamily = \"cascadiacode\"\n"
        );
        assert_eq!(toml_get(&set, "fonts", "family").as_deref(), Some("\"X\""));
        assert_eq!(toml_set(&set, "fonts", "family", None), rio);
        let dotted = "fonts.family = \"a\"\n[fonts]\nsize = 1\n";
        assert_eq!(toml_get(dotted, "fonts", "family").as_deref(), Some("\"a\""));
        assert!(toml_set(dotted, "fonts", "family", Some("\"b\"")).starts_with("fonts.family = \"b\"\n"));
        let warp = toml_set("", "appearance.text", "font_name", Some("\"X\""));
        assert_eq!(warp, "[appearance.text]\nfont_name = \"X\"\n");
        assert_eq!(toml_set(&warp, "appearance.text", "font_name", None), "");

        // The recipes read and judge them.
        let r = recipe("QTerminal").unwrap();
        assert!(
            !recipe_font_set(r, "[General]\nfontFamily=MesloLGM Nerd Font\nfont=@Variant(old)\n"),
            "a legacy font key overrides the family"
        );
        assert!(recipe_font_set(r, "[General]\nfontFamily=MesloLGM Nerd Font\n"));
        assert!(recipe_font_set(
            recipe("Zed").unwrap(),
            "{ \"terminal\": { \"font_family\": \"Hack Nerd Font\" } }"
        ));
        assert!(!recipe_font_set(recipe("Rio").unwrap(), "[fonts]\nfamily = \"Fira Code\"\n"));

        // foot's first section has no name until it needs one.
        assert_eq!(
            foot_main("# comment\nfont=monospace:size=8\n\n[colors]\nforeground=ffffff\n"),
            "[main]\n# comment\nfont=monospace:size=8\n\n[colors]\nforeground=ffffff\n"
        );
        assert_eq!(foot_main("[main]\nfont=x\n"), "[main]\nfont=x\n");
        assert_eq!(foot_main("# only a comment\n"), "# only a comment\n");
        assert_eq!(foot_main(""), "");
    }

    #[test]
    fn hyper_conemu_terminator_and_x_resources_are_edited_and_put_back() {
        let js = "module.exports = {\n  config: {\n    fontSize: 12,\n    fontFamily: 'Menlo, \"DejaVu Sans Mono\", monospace',\n    cursorColor: 'red',\n  },\n  plugins: [],\n};\n";
        assert_eq!(hyper_font(js).as_deref(), Some("'Menlo, \"DejaVu Sans Mono\", monospace'"));
        assert!(!hyper_is_nerd(&hyper_font(js).unwrap()));
        let set = hyper_with_font(js, Some(HYPER_FONTS)).unwrap();
        assert!(set.contains(&format!("    fontFamily: {HYPER_FONTS},\n")), "{set}");
        assert!(hyper_is_nerd(&hyper_font(&set).unwrap()));
        assert_eq!(
            hyper_with_font(&set, Some("'Menlo, \"DejaVu Sans Mono\", monospace'")).unwrap(),
            js
        );
        let bare = "module.exports = {\n  config: {\n    fontSize: 12,\n  },\n};\n";
        let added = hyper_with_font(bare, Some("'X'")).unwrap();
        assert_eq!(added, "module.exports = {\n  config: {\n    fontFamily: 'X',\n    fontSize: 12,\n  },\n};\n");
        assert_eq!(hyper_with_font(&added, None).unwrap(), bare);
        assert!(hyper_with_font("", Some("'X'")).is_err());

        // ConEmu: a FontName in a deeper key (HotKeys) is not a configuration's.
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<key name=\"Software\">\r\n\t<key name=\"ConEmu\">\r\n\t\t<key name=\".Vanilla\" build=\"1\">\r\n\t\t\t<value name=\"FontName\" type=\"string\" data=\"Consolas\"/>\r\n\t\t\t<key name=\"HotKeys\">\r\n\t\t\t\t<value name=\"FontName\" type=\"string\" data=\"not this one\"/>\r\n\t\t\t</key>\r\n\t\t</key>\r\n\t\t<key name=\"Work\">\r\n\t\t\t<value name=\"FontSize\" type=\"ulong\" data=\"16\"/>\r\n\t\t</key>\r\n\t</key>\r\n</key>\r\n";
        assert_eq!(
            conemu_fonts(xml),
            [
                (".Vanilla".to_string(), Some("Consolas".to_string())),
                ("Work".to_string(), None)
            ]
        );
        let set = conemu_with_font(&conemu_with_font(xml, ".Vanilla", Some(FONT_MONO)), "Work", Some(FONT_MONO));
        assert_eq!(
            conemu_fonts(&set),
            [
                (".Vanilla".to_string(), Some(FONT_MONO.to_string())),
                ("Work".to_string(), Some(FONT_MONO.to_string()))
            ]
        );
        assert!(set.contains("\t\t\t<value name=\"FontName\" type=\"string\" data=\"MesloLGM Nerd Font Mono\"/>\r\n\t\t</key>\r\n\t</key>"), "{set}");
        assert!(set.contains("data=\"not this one\""));
        assert_eq!(
            conemu_with_font(&conemu_with_font(&set, "Work", None), ".Vanilla", Some("Consolas")),
            xml
        );

        // Terminator: every profile under [profiles], with the file's own indents.
        let cfg = "[global_config]\n  title_hide_sizetext = True\n[profiles]\n  [[default]]\n    cursor_color = \"#aaaaaa\"\n    font = Mono 10\n  [[work]]\n    scrollback_lines = 5000\n[layouts]\n  [[default]]\n    [[[window0]]]\n      type = Window\n";
        assert_eq!(terminator_profiles(cfg), ["default", "work"]);
        assert_eq!(terminator_get(cfg, "default", "font").as_deref(), Some("Mono 10"));
        assert_eq!(terminator_get(cfg, "work", "font"), None);
        let mut set = cfg.to_string();
        for p in ["default", "work"] {
            set = terminator_set(&set, p, "font", Some("X 11"));
            set = terminator_set(&set, p, "use_system_font", Some("False"));
        }
        assert_eq!(
            set,
            "[global_config]\n  title_hide_sizetext = True\n[profiles]\n  [[default]]\n    use_system_font = False\n    cursor_color = \"#aaaaaa\"\n    font = X 11\n  [[work]]\n    use_system_font = False\n    font = X 11\n    scrollback_lines = 5000\n[layouts]\n  [[default]]\n    [[[window0]]]\n      type = Window\n"
        );
        let mut back = set;
        for p in ["default", "work"] {
            back = terminator_set(&back, p, "use_system_font", None);
        }
        back = terminator_set(&back, "work", "font", None);
        back = terminator_set(&back, "default", "font", Some("Mono 10"));
        assert_eq!(back, cfg);
        let made = terminator_set("", "default", "font", Some("X 11"));
        assert_eq!(made, "[profiles]\n  [[default]]\n    font = X 11\n");
        assert_eq!(terminator_set(&made, "default", "font", None), "");

        // X resources, by whatever path and case the file uses.
        let xres = "! fonts\nxterm*faceName: DejaVu Sans Mono\nXTerm.vt100.faceSize: 12\nURxvt.font: xft:Hack:size=10\n";
        assert_eq!(xres_get(xres, "XTerm", "faceName").as_deref(), Some("DejaVu Sans Mono"));
        assert_eq!(xres_get(xres, "XTerm", "faceSize").as_deref(), Some("12"));
        assert_eq!(xres_get(xres, "URxvt", "font").as_deref(), Some("xft:Hack:size=10"));
        let set = xres_set(xres, "XTerm", "faceName", Some(FONT_NAME));
        assert_eq!(
            set,
            "! fonts\nxterm*faceName: MesloLGM Nerd Font\nXTerm.vt100.faceSize: 12\nURxvt.font: xft:Hack:size=10\n"
        );
        assert_eq!(xres_set(&set, "XTerm", "faceName", Some("DejaVu Sans Mono")), xres);
        assert_eq!(xres_set("", "URxvt", "font", Some("x")), "URxvt*font: x\n");
        assert!(!xres_is("*faceName: x", "XTerm", "faceName"));
    }

    #[test]
    fn linux_desktop_terminals_of_every_shape_are_set_and_put_back() {
        let mut f = Fake::new(Os::Linux, "/home/ann");
        f.program("gsettings", "/usr/bin/gsettings");
        f.program("xrdb", "/usr/bin/xrdb");
        // MATE Terminal lists its profiles under its own key names.
        f.answer("gsettings", "get org.mate.terminal.global profile-list", "['default', 'root']\n");
        for id in ["default", "root"] {
            let p = format!("org.mate.terminal.profile:/org/mate/terminal/profiles/{id}/");
            f.answer("gsettings", &format!("get {p} use-system-font"), "true\n");
            f.answer("gsettings", &format!("get {p} font"), "'Monospace 12'\n");
        }
        // Ptyxis is known by its schema, GNOME Console by its program.
        f.answer("gsettings", "get org.gnome.Ptyxis use-system-font", "true\n");
        f.answer("gsettings", "get org.gnome.Ptyxis font-name", "'Monospace 10'\n");
        f.program("kgx", "/usr/bin/kgx");
        f.answer("gsettings", "get org.gnome.Console use-system-font", "true\n");
        f.answer("gsettings", "get org.gnome.Console custom-font", "''\n");
        f.answer("gsettings", "set", "");
        f.answer("gsettings", "reset", "");
        f.program("terminator", "/usr/bin/terminator"); // no config yet
        f.program("foot", "/usr/bin/foot");
        f.file("/home/ann/.config/foot/foot.ini", "font=monospace:size=8\n\n[colors]\nforeground=ffffff\n");
        f.program("lxterminal", "/usr/bin/lxterminal"); // no config yet
        f.file("/home/ann/.config/qterminal.org/qterminal.ini", "[General]\nfontFamily=Hack\nfont=@Variant(old)\n");
        f.file("/home/ann/.hyper.js", "module.exports = {\n  config: {\n    fontSize: 12,\n    fontFamily: 'Menlo, monospace',\n  },\n};\n");
        f.program("zed", "/usr/bin/zed"); // no settings yet
        f.file("/home/ann/.config/rio/config.toml", "[fonts]\nsize = 14.0\n");
        f.program("xterm", "/usr/bin/xterm");
        f.file("/home/ann/.Xresources", "xterm*faceName: DejaVu Sans Mono\nxterm*faceSize: 12\n");
        f.program("putty", "/usr/bin/putty");
        f.file("/home/ann/.putty/sessions/work", "HostName=example.org\nFont=client:Monospace 10\n");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        let names: Vec<&str> = s.terminals.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            ["MATE Terminal", "Ptyxis", "GNOME Console", "Terminator", "LXTerminal", "QTerminal", "foot", "xterm", "PuTTY", "Hyper", "Rio", "Zed"]
        );
        assert!(s.terminals.iter().all(|t| !t.font_set), "{:?}", s.terminals);
        assert_eq!(s.terminals[0].detail, "2 profiles");

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        let runs = fake.runs();
        for want in [
            "set org.mate.terminal.profile:/org/mate/terminal/profiles/root/ font MesloLGM Nerd Font 11",
            "set org.mate.terminal.profile:/org/mate/terminal/profiles/default/ use-system-font false",
            "set org.gnome.Ptyxis use-system-font false",
            "set org.gnome.Ptyxis font-name MesloLGM Nerd Font 11",
            "set org.gnome.Console custom-font MesloLGM Nerd Font 11",
        ] {
            assert!(runs.iter().any(|r| r.ends_with(want)), "{want}: {runs:?}");
        }
        assert!(
            runs.iter().any(|r| r.starts_with("/usr/bin/xrdb -merge ") && r.ends_with(".Xresources")),
            "{runs:?}"
        );
        assert_eq!(
            fake.text("/home/ann/.config/terminator/config"),
            "[profiles]\n  [[default]]\n    use_system_font = False\n    font = MesloLGM Nerd Font 11\n"
        );
        assert_eq!(
            fake.text("/home/ann/.config/foot/foot.ini"),
            "[main]\nfont=MesloLGM Nerd Font:size=11\n\n[colors]\nforeground=ffffff\n"
        );
        assert_eq!(
            fake.text("/home/ann/.config/lxterminal/lxterminal.conf"),
            "[general]\nfontname=MesloLGM Nerd Font 11\n"
        );
        assert_eq!(
            fake.text("/home/ann/.config/qterminal.org/qterminal.ini"),
            "[General]\nfontFamily=MesloLGM Nerd Font\n"
        );
        assert!(fake.text("/home/ann/.hyper.js").contains(&format!("    fontFamily: {HYPER_FONTS},\n")));
        let zed = fake.text("/home/ann/.config/zed/settings.json");
        assert_eq!(
            jsonc::get_string(&zed, &["terminal", "font_family"]).as_deref(),
            Some(FONT_NAME),
            "{zed}"
        );
        assert_eq!(
            fake.text("/home/ann/.config/rio/config.toml"),
            "[fonts]\nfamily = \"MesloLGM Nerd Font\"\nsize = 14.0\n"
        );
        assert_eq!(
            fake.text("/home/ann/.Xresources"),
            "xterm*faceName: MesloLGM Nerd Font\nxterm*faceSize: 12\n"
        );
        assert_eq!(
            fake.text("/home/ann/.putty/sessions/work"),
            "HostName=example.org\nFont=client:MesloLGM Nerd Font 11\n"
        );
        // The fake's gsettings answers do not change on `set`; every file-backed one is.
        let after = posh.status();
        let by_file = |t: &TerminalStatus| {
            !matches!(t.kind, TerminalKind::MateTerminal | TerminalKind::Gsettings)
        };
        assert!(after.terminals.iter().filter(|t| by_file(t)).all(|t| t.font_set), "{:?}", after.terminals);

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.ends_with("root/ font 'Monospace 12'")), "{runs:?}");
        assert!(runs.iter().any(|r| r.ends_with("set org.gnome.Ptyxis use-system-font true")));
        assert!(runs.iter().any(|r| r.ends_with("set org.gnome.Console custom-font ''")));
        assert_eq!(fake.text("/home/ann/.config/terminator/config"), "");
        assert_eq!(
            fake.text("/home/ann/.config/foot/foot.ini"),
            "[main]\nfont=monospace:size=8\n\n[colors]\nforeground=ffffff\n"
        );
        assert_eq!(fake.text("/home/ann/.config/lxterminal/lxterminal.conf"), "");
        let qt = fake.text("/home/ann/.config/qterminal.org/qterminal.ini");
        assert_eq!(ini_get(&qt, "General", "fontFamily").as_deref(), Some("Hack"));
        assert_eq!(ini_get(&qt, "General", "font").as_deref(), Some("@Variant(old)"));
        assert_eq!(
            fake.text("/home/ann/.hyper.js"),
            "module.exports = {\n  config: {\n    fontSize: 12,\n    fontFamily: 'Menlo, monospace',\n  },\n};\n"
        );
        let zed = fake.text("/home/ann/.config/zed/settings.json");
        assert_eq!(jsonc::get_raw(&zed, &["terminal", "font_family"]), None, "{zed}");
        assert_eq!(fake.text("/home/ann/.config/rio/config.toml"), "[fonts]\nsize = 14.0\n");
        assert_eq!(
            fake.text("/home/ann/.Xresources"),
            "xterm*faceName: DejaVu Sans Mono\nxterm*faceSize: 12\n"
        );
        assert_eq!(
            fake.text("/home/ann/.putty/sessions/work"),
            "HostName=example.org\nFont=client:Monospace 10\n"
        );
        assert!(fake.text("/home/ann/.idlewalker/terminal-undo.txt").is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn windows_conemu_putty_mintty_and_warp_are_set_and_put_back() {
        let mut f = Fake::new(Os::Windows, r"C:\Users\ann");
        f.env.insert("LOCALAPPDATA".into(), r"C:\Users\ann\AppData\Local".into());
        f.env.insert("APPDATA".into(), r"C:\Users\ann\AppData\Roaming".into());
        f.env.insert("ProgramFiles".into(), r"C:\Program Files".into());
        f.env.insert("USERNAME".into(), "ann".into());
        f.env.insert("CMDER_ROOT".into(), r"C:\tools\cmder".into());
        f.program("reg", r"C:\Windows\System32\reg.exe");
        let xml = "<key name=\"Software\">\n\t<key name=\"ConEmu\">\n\t\t<key name=\".Vanilla\">\n\t\t\t<value name=\"FontName\" type=\"string\" data=\"Consolas\"/>\n\t\t</key>\n\t</key>\n</key>\n";
        f.file(r"C:\Users\ann\AppData\Roaming\ConEmu.xml", xml);
        f.file(r"C:\tools\cmder\config\user-ConEmu.xml", xml);
        f.answer("reg.exe", r"query HKEY_CURRENT_USER\Software\ConEmu /s", "");
        f.answer(
            "reg.exe",
            r"query HKEY_CURRENT_USER\Software\SimonTatham\PuTTY\Sessions /s",
            "\r\nHKEY_CURRENT_USER\\Software\\SimonTatham\\PuTTY\\Sessions\r\n\r\nHKEY_CURRENT_USER\\Software\\SimonTatham\\PuTTY\\Sessions\\Default%20Settings\r\n    Font    REG_SZ    Courier New\r\n    FontHeight    REG_DWORD    0xa\r\n",
        );
        f.answer("reg.exe", "add", "");
        f.answer("reg.exe", "delete", "");
        f.file(r"C:\Program Files\Git\usr\bin\mintty.exe", "");
        f.file(r"C:\Users\ann\.minttyrc", "BoldAsFont=no\n");
        f.file(r"C:\msys64\usr\bin\mintty.exe", "");
        f.file(r"C:\Users\ann\AppData\Local\Programs\Warp\Warp.exe", "");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        let names: Vec<&str> = s.terminals.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            ["Console windows (cmd, PowerShell)", "ConEmu", "Cmder", "PuTTY", "Git Bash (mintty)", "MSYS2 (mintty)", "Warp"]
        );

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        for file in [r"C:\Users\ann\AppData\Roaming\ConEmu.xml", r"C:\tools\cmder\config\user-ConEmu.xml"] {
            assert_eq!(
                conemu_fonts(&fake.text(file)),
                [(".Vanilla".to_string(), Some(FONT_MONO.to_string()))]
            );
        }
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.ends_with(r"add HKEY_CURRENT_USER\Software\SimonTatham\PuTTY\Sessions\Default%20Settings /v Font /t REG_SZ /d MesloLGM Nerd Font Mono /f")), "{runs:?}");
        assert!(runs.iter().any(|r| r.contains(r"Default%20Settings /v FontIsBold /t REG_DWORD /d 0 /f")));
        assert!(!runs.iter().any(|r| r.contains(r"Default%20Settings /v FontHeight")), "a height already there is kept");
        assert_eq!(fake.text(r"C:\Users\ann\.minttyrc"), "BoldAsFont=no\nFont=MesloLGM Nerd Font Mono\n");
        assert_eq!(fake.text(r"C:\msys64\home\ann\.minttyrc"), "Font=MesloLGM Nerd Font Mono\n");
        assert_eq!(
            fake.text(r"C:\Users\ann\AppData\Local\warp\Warp\config\settings.toml"),
            "[appearance.text]\nfont_name = \"MesloLGM Nerd Font\"\n"
        );

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        assert_eq!(fake.text(r"C:\Users\ann\AppData\Roaming\ConEmu.xml"), xml);
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.ends_with(r"Default%20Settings /v Font /t REG_SZ /d Courier New /f")), "{runs:?}");
        assert!(runs.iter().any(|r| r.ends_with(r"delete HKEY_CURRENT_USER\Software\SimonTatham\PuTTY\Sessions\Default%20Settings /v FontIsBold /f")));
        assert_eq!(fake.text(r"C:\Users\ann\.minttyrc"), "BoldAsFont=no\n");
        assert!(
            !fake.files.lock().unwrap().contains_key(Path::new(r"C:\msys64\home\ann\.minttyrc")),
            "a file made for the font alone goes"
        );
        assert_eq!(fake.text(r"C:\Users\ann\AppData\Local\warp\Warp\config\settings.toml"), "");
    }

    #[test]
    fn gsettings_lists_are_read() {
        assert_eq!(
            gvariant_strings("['b1dcc9dd-5262-4d8d-a863-c897e6d979b9', 'x']"),
            ["b1dcc9dd-5262-4d8d-a863-c897e6d979b9", "x"]
        );
        assert!(gvariant_strings("@as []").is_empty());
        assert_eq!(gvariant_strings("'one'"), ["one"]);
    }

    #[test]
    fn the_console_registry_is_read_as_reg_prints_it() {
        let printed = "\r\nHKEY_CURRENT_USER\\Console\r\n    FaceName    REG_SZ    Consolas\r\n    FontSize    REG_DWORD    0x100000\r\n    EmptyValue    REG_SZ    \r\n\r\nHKEY_CURRENT_USER\\Console\\%SystemRoot%_system32_cmd.exe\r\n    FaceName    REG_SZ    Lucida Console\r\n\r\nHKEY_CURRENT_USER\\Console\\Git Bash\r\n    ScreenBufferSize    REG_DWORD    0x23290078\r\n";
        let keys = reg_parse(printed);
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0].path, "HKEY_CURRENT_USER\\Console");
        assert_eq!(reg_value(&keys[0], "FaceName"), Some(("REG_SZ", "Consolas")));
        assert_eq!(reg_value(&keys[0], "FontSize"), Some(("REG_DWORD", "0x100000")));
        assert_eq!(reg_value(&keys[0], "EmptyValue"), Some(("REG_SZ", "")));
        assert_eq!(
            reg_value(&keys[1], "FaceName"),
            Some(("REG_SZ", "Lucida Console"))
        );
        assert_eq!(reg_value(&keys[2], "FaceName"), None);
    }

    const WT: &str = r#"{
    "$schema": "https://aka.ms/terminal-profiles-schema",
    "profiles":
    {
        "defaults": { "fontFace": "Cascadia Code" },
        "list":
        [
            { "name": "PowerShell", "guid": "{p}", "font": { "face": "Consolas", "size": 11 } },
            { "name": "cmd", "guid": "{c}" },
            { "name": "Git Bash", "fontFace": "Lucida Console" }
        ]
    }
}"#;

    #[test]
    fn windows_terminal_gets_the_font_in_every_profile_and_gives_it_back() {
        assert_eq!(wt_status(WT), (false, 3));
        let mut changes = Changes { list: Vec::new() };
        let (set, n) = wt_with_font(WT, "wt:x", FONT_NAME, &mut changes).unwrap();
        assert_eq!(n, 3);
        assert_eq!(wt_status(&set), (true, 3));
        assert_eq!(jsonc::get_string(&set, &WT_FACE).as_deref(), Some(FONT_NAME));
        assert_eq!(jsonc::get_raw(&set, &WT_COLOURS).as_deref(), Some("\"never\""));
        assert!(!set.contains("fontFace"), "{set}");
        assert!(set.contains("\"face\": \"MesloLGM Nerd Font\", \"size\": 11"));
        // cmd follows the defaults, so it was left alone.
        assert!(set.contains("{ \"name\": \"cmd\", \"guid\": \"{c}\" }"));
        let keys: Vec<&str> = changes.list.iter().map(|u| u.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "defaults.font.face",
                "defaults.fontFace",
                "defaults.adjustIndistinguishableColors",
                "list:Git Bash:fontFace",
                "list:Git Bash:font.face",
                "list:{p}:font.face",
            ]
        );
        let entries: Vec<&Undo> = changes.list.iter().collect();
        let back = wt_without_font(&set, &entries).unwrap();
        assert_eq!(wt_status(&back), (false, 3));
        assert_eq!(
            jsonc::get_string(&back, &WT_LEGACY_FACE).as_deref(),
            Some("Cascadia Code")
        );
        assert_eq!(jsonc::get_raw(&back, &WT_COLOURS), None);
        assert!(back.contains("\"face\": \"Consolas\", \"size\": 11"), "{back}");
        assert!(back.contains("\"fontFace\": \"Lucida Console\""), "{back}");
        assert!(!back.contains("Nerd Font"), "{back}");
        // The font object that was made for the defaults is gone again.
        assert_eq!(jsonc::get_raw(&back, &["profiles", "defaults", "font"]), None);
    }

    const ITERM: &str = "<dict>\n<key>New Bookmarks</key>\n<array>\n<dict>\n<key>Guid</key>\n<string>A-1</string>\n<key>Name</key>\n<string>Default</string>\n<key>Non Ascii Font</key>\n<string>Monaco 12</string>\n<key>Normal Font</key>\n<string>Menlo-Regular 13</string>\n</dict>\n<dict>\n<key>Guid</key>\n<string>B-2</string>\n<key>Normal Font</key>\n<string>MesloLGMNerdFontMono-Regular 11</string>\n</dict>\n</array>\n</dict>\n";

    #[test]
    fn iterm2_profiles_keep_their_size_and_get_the_font_and_give_it_back() {
        assert_eq!(
            iterm_fonts(ITERM),
            ["Menlo-Regular 13", "MesloLGMNerdFontMono-Regular 11"]
        );
        let entries = iterm_font_entries(ITERM);
        let seen: Vec<(&str, &str, &str)> = entries
            .iter()
            .map(|f| (f.guid.as_str(), f.key.as_str(), f.value.as_str()))
            .collect();
        assert_eq!(
            seen,
            [
                ("A-1", "Non Ascii Font", "Monaco 12"),
                ("A-1", "Normal Font", "Menlo-Regular 13"),
                ("B-2", "Normal Font", "MesloLGMNerdFontMono-Regular 11"),
            ]
        );
        let (out, n) = iterm_with_fonts(ITERM, |f| {
            Some(format!("{FONT_MONO_POSTSCRIPT} {}", font_size_of(&f.value)))
        });
        assert_eq!(n, 3);
        assert_eq!(
            iterm_fonts(&out),
            ["MesloLGMNFM-Regular 13", "MesloLGMNFM-Regular 11"]
        );
        assert!(out.contains("<key>Non Ascii Font</key>\n<string>MesloLGMNFM-Regular 12</string>"));
        assert!(out.contains("<key>Name</key>\n<string>Default</string>"));
        let (back, m) = iterm_with_fonts(&out, |f| {
            entries
                .iter()
                .find(|e| e.guid == f.guid && e.key == f.key)
                .map(|e| e.value.clone())
        });
        assert_eq!(m, 3);
        assert_eq!(back, ITERM);
        assert_eq!(iterm_with_fonts("<dict></dict>", |_| Some("X".into())).1, 0);
    }

    /// A keyed archive holding the font name, as base64.
    fn archived(name: &str) -> String {
        let bytes = format!("bplist00\u{1}{name}\u{2}NSFont");
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.as_bytes().chunks(3) {
            let mut v = 0u32;
            for i in 0..3 {
                v = (v << 8) | *chunk.get(i).unwrap_or(&0) as u32;
            }
            for i in 0..4 {
                if i <= chunk.len() {
                    out.push(table[((v >> (18 - 6 * i)) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    fn terminal_plist(font: &str) -> String {
        format!(
            "<dict>\n<key>Default Window Settings</key>\n<string>Basic</string>\n<key>Window Settings</key>\n<dict>\n<key>Basic</key>\n<dict>\n<key>Font</key>\n<data>\n{}\n</data>\n<key>Nested</key>\n<dict>\n<key>Inner</key>\n<string>x</string>\n</dict>\n</dict>\n<key>Pro</key>\n<dict>\n<key>Font</key>\n<data>\n{}\n</data>\n</dict>\n</dict>\n</dict>\n",
            archived(font),
            archived("Menlo-Regular")
        )
    }

    #[test]
    fn terminal_app_s_profiles_and_archived_fonts_are_read() {
        assert_eq!(
            plist_profile_names(&terminal_plist("Menlo-Regular")),
            ["Basic", "Pro"]
        );
        assert!(!apple_terminal_font_is_nerd(&terminal_plist("Menlo-Regular"), "Basic"));
        assert!(!apple_terminal_font_is_nerd(&terminal_plist("SFMonoTerminal-Regular"), "Basic"));
        assert!(apple_terminal_font_is_nerd(&terminal_plist(FONT_MONO_POSTSCRIPT), "Basic"));
        assert!(apple_terminal_font_is_nerd(&terminal_plist("MesloLGMNerdFontMono-Regular"), "Basic"));
        assert!(!apple_terminal_font_is_nerd(&terminal_plist(FONT_MONO_POSTSCRIPT), "Pro"));
        assert_eq!(base64_decode("aGVsbG8="), b"hello");
        // The real archive from a Mac, holding "SFMonoTerminal-Regular".
        let real = "<dict><key>Window Settings</key><dict><key>Basic</key><dict><key>Font</key><data>\nYnBsaXN0MDDUAQIDBAUGBwpYJHZlcnNpb25ZJGFyY2hpdmVyVCR0b3BYJG9iamVjdHMSAAGGoF8QD05TS2V5ZWRBcmNoaXZlctEICVRyb290gAGkCwwVFlUkbnVsbNQNDg8QERITFFZOU1NpemVYTlNmRmxhZ3NWTlNOYW1lViRjbGFzcyNAJgAAAAAAABAQgAKAA18QFlNGTW9ub1Rlcm1pbmFsLVJlZ3VsYXLSFxgZGlokY2xhc3NuYW1lWCRjbGFzc2VzVk5TRm9udKIZG1hOU09iamVjdAgRGiQpMjdJTFFTWF5nbnd+hY6QkpStsr3GzdAAAAAAAAABAQAAAAAAAAAcAAAAAAAAAAAAAAAAAAAA2Q==\n</data></dict></dict></dict>";
        assert!(!apple_terminal_font_is_nerd(real, "Basic"));
        assert!(String::from_utf8_lossy(&base64_decode(real.split("<data>").nth(1).unwrap().split("</data>").next().unwrap())).contains("SFMonoTerminal-Regular"));
    }

    #[test]
    fn a_mac_sets_terminal_and_iterm2_fonts_and_puts_them_back() {
        let mut f = Fake::new(Os::MacOs, "/Users/ann");
        for p in ["defaults", "osascript", "pgrep"] {
            f.program(p, &format!("/usr/bin/{p}"));
        }
        f.file(
            "/Users/ann/Library/Preferences/com.googlecode.iterm2.plist",
            "",
        );
        f.answer("defaults", "read", "Basic\n");
        // One export answer serves both domains: it holds both shapes.
        f.answers.insert(
            ("defaults".into(), "export".into()),
            format!("{}{ITERM}", terminal_plist("Menlo-Regular")),
        );
        f.answer("defaults", "import", "");
        f.answer("osascript", "-e", "");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        let names: Vec<&str> = s.terminals.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Terminal", "iTerm2"]);
        assert!(s.terminals.iter().all(|t| !t.font_set));
        assert_eq!(s.terminals[0].detail, "2 profiles");
        assert!(s.terminal_note.contains("256 colours"));

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        let runs = fake.runs();
        let script = runs
            .iter()
            .filter(|r| r.starts_with("/usr/bin/osascript") && r.contains("set font name of"))
            .last()
            .expect("Terminal was told through AppleScript");
        assert!(script
            .contains("set font name of default settings to \"MesloLGMNFM-Regular\""));
        assert!(script.contains("settings set \"Basic\""), "{script}");
        assert!(script.contains("settings set \"Pro\""), "{script}");
        assert!(runs
            .iter()
            .any(|r| r.contains("import com.googlecode.iterm2 ")
                && r.ends_with("idlewalker-iterm2.plist")));
        assert!(log
            .iter()
            .any(|l| l.starts_with("iTerm2: font set to MesloLGM Nerd Font Mono in 2 profiles")));
        assert!(
            !fake.exists(Path::new("/tmp/idlewalker-iterm2.plist")),
            "temp file removed"
        );
        // The record holds iTerm2's fonts by profile; Terminal's were not told (the
        // fake's AppleScript lists nothing), so there is nothing to put back there.
        let record = undo_parse(&fake.text("/Users/ann/.idlewalker/terminal-undo.txt"));
        assert!(record.iter().any(|u| u.target == "iterm2:A-1"
            && u.key == "Normal Font"
            && u.before.as_deref() == Some("Menlo-Regular 13")));
        assert!(record
            .iter()
            .any(|u| u.target == "terminal.app:Basic" && u.before.is_none()));
        assert!(posh.status().undo_available);

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        assert!(log.iter().any(|l| l.starts_with("iTerm2: fonts put back in 3")), "{log:?}");
        assert!(!posh.status().undo_available);
    }

    #[test]
    fn a_running_terminal_is_asked_for_its_fonts_once_they_have_been_set() {
        let mut f = Fake::new(Os::MacOs, "/Users/ann");
        for p in ["defaults", "osascript", "pgrep"] {
            f.program(p, &format!("/usr/bin/{p}"));
        }
        f.answer("defaults", "read", "Basic\n");
        // The file on disk keeps saying Menlo throughout: Terminal writes it later.
        f.answers.insert(
            ("defaults".into(), "export".into()),
            terminal_plist("Menlo-Regular"),
        );
        f.answer("pgrep", "-x Terminal", "123\n");
        f.answer("osascript", "-e", "");
        // Terminal, asked for its profiles: Menlo before the font is set, and the Nerd
        // Font from then on.
        let menlo = "Basic\tMenlo-Regular\nPro\tMenlo-Regular\n";
        let nerd = "Basic\tMesloLGMNFM-Regular\nPro\tMesloLGMNFM-Regular\n";
        f.answer_in_turn("osascript", &format!("-e {APPLE_TERMINAL_LIST}"), &[menlo, nerd]);
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        assert!(!posh.status().terminals[0].font_set);
        assert!(
            !fake.runs().iter().any(|r| r.contains("settings sets")),
            "Terminal is not asked before the font was ever set: that would start it"
        );

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        assert!(log.iter().any(|l| l.starts_with("Terminal: font set to MesloLGM Nerd Font Mono in 2 profiles")), "{log:?}");
        let sets = fake.runs().iter().filter(|r| r.contains("set font name of")).count();
        assert_eq!(sets, 1, "the first name was taken, so no other was tried");
        let s = posh.status();
        assert!(s.terminals[0].font_set, "Terminal's own answer counts, not the stale file");
        assert_eq!(s.terminals[0].detail, "2 profiles");

        // Terminal not running: the file is all there is, and it still says Menlo.
        let mut g = Fake::new(Os::MacOs, "/Users/ann");
        for (path, text) in fake.files.lock().unwrap().iter() {
            g.file(&path.display().to_string(), text);
        }
        g.programs = fake.programs.clone();
        g.answers = fake.answers.clone();
        g.answers.remove(&("pgrep".into(), "-x Terminal".into()));
        assert!(!Posh::new(Arc::new(g)).status().terminals[0].font_set);
    }

    #[test]
    fn a_terminal_that_keeps_its_old_font_is_offered_each_name_and_then_told_the_truth() {
        let mut f = Fake::new(Os::MacOs, "/Users/ann");
        for p in ["defaults", "osascript", "pgrep"] {
            f.program(p, &format!("/usr/bin/{p}"));
        }
        f.answer("defaults", "read", "Basic\n");
        f.answers.insert(
            ("defaults".into(), "export".into()),
            terminal_plist("Menlo-Regular"),
        );
        f.answer("osascript", "-e", "");
        // macOS resolves the family name to a PostScript name of its own: that one
        // goes first. Terminal says "done" to every name, and takes the third.
        f.answer(
            "osascript",
            &format!("-l JavaScript -e {APPLE_FONT_PROBE}"),
            "MesloLGMNFM-Regular: missing\nMesloLGM Nerd Font Mono: opens as MesloLGMNFM-Reg, found\nMesloLGMNerdFontMono-Regular: missing\n",
        );
        let menlo = "Basic\tMenlo-Regular\n";
        let nerd = "Basic\tMesloLGMNFM-Reg\n";
        f.answer_in_turn(
            "osascript",
            &format!("-e {APPLE_TERMINAL_LIST}"),
            &[menlo, menlo, menlo, nerd],
        );
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        let scripts: Vec<String> = fake
            .runs()
            .iter()
            .filter(|r| r.contains("set font name of"))
            .cloned()
            .collect();
        assert_eq!(scripts.len(), 3, "{scripts:?}");
        assert!(scripts[0].contains("to \"MesloLGMNFM-Reg\""), "{}", scripts[0]);
        assert!(scripts[1].contains("to \"MesloLGMNFM-Regular\""), "{}", scripts[1]);
        assert!(scripts[2].contains("set font name of default settings to \"MesloLGMNerdFontMono-Regular\""));
        assert!(scripts[2].contains("set font name of settings set \"Basic\" to \"MesloLGMNerdFontMono-Regular\""));
        assert!(!log.iter().any(|l| l.contains("macOS does not see the font")), "{log:?}");
        // The record still holds the font that was there before any of that.
        let record = undo_parse(&fake.text("/Users/ann/.idlewalker/terminal-undo.txt"));
        assert!(record.iter().any(|u| u.target == "terminal.app:Basic"
            && u.before.as_deref() == Some("Menlo-Regular")));

        // A Terminal that never takes it: every name is tried, and the window says so
        // instead of claiming the font was set.
        let mut f = Fake::new(Os::MacOs, "/Users/ann");
        for p in ["defaults", "osascript"] {
            f.program(p, &format!("/usr/bin/{p}"));
        }
        f.answer("defaults", "read", "Basic\n");
        f.answers.insert(
            ("defaults".into(), "export".into()),
            terminal_plist("Menlo-Regular"),
        );
        f.answer("osascript", "-e", "");
        f.answer(
            "osascript",
            &format!("-l JavaScript -e {APPLE_FONT_PROBE}"),
            "MesloLGMNFM-Regular: missing\nMesloLGM Nerd Font Mono: missing\navailable fonts with Meslo in the name: none\n",
        );
        f.answer_in_turn("osascript", &format!("-e {APPLE_TERMINAL_LIST}"), &[menlo]);
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let mut log = Vec::new();
        assert!(!posh.set_terminal_fonts(&mut log), "{log:?}");
        assert_eq!(
            fake.runs().iter().filter(|r| r.contains("set font name of")).count(),
            APPLE_TERMINAL_FONT_NAMES.len()
        );
        assert!(log.iter().any(|l| l.starts_with("Terminal: macOS does not see the font")), "{log:?}");
        assert!(
            log.iter().any(|l| l.starts_with("Terminal: did not take the font; it still reports Basic (Menlo-Regular).")),
            "{log:?}"
        );
        assert!(!log.iter().any(|l| l.starts_with("Terminal: font set")), "{log:?}");
    }

    #[test]
    fn a_refused_apple_event_is_told_apart_from_a_broken_script() {
        let denied = RunResult {
            success: false,
            stdout: String::new(),
            stderr: "execution error: Not authorized to send Apple events to Terminal. (-1743)"
                .into(),
        };
        assert!(apple_events_denied(&denied));
        let syntax = RunResult {
            success: false,
            stdout: String::new(),
            stderr: "syntax error: Expected end of line (-2741)".into(),
        };
        assert!(!apple_events_denied(&syntax));
    }

    #[test]
    fn a_mac_that_cannot_run_the_script_is_told_to_set_the_font_by_hand() {
        let mut f = Fake::new(Os::MacOs, "/Users/ann");
        for p in ["defaults", "osascript"] {
            f.program(p, &format!("/usr/bin/{p}"));
        }
        f.answer("defaults", "read", "Basic\n");
        f.answers.insert(
            ("defaults".into(), "export".into()),
            terminal_plist("Menlo-Regular"),
        );
        // No answer for osascript: the fake fails it.
        let posh = Posh::new(Arc::new(f));
        let mut log = Vec::new();
        assert!(!posh.set_terminal_fonts(&mut log), "{log:?}");
        assert!(log.iter().any(|l| l.contains("Settings > Profiles > Text")));
        assert!(!log.iter().any(|l| l.contains("Automation")), "{log:?}");
    }

    #[test]
    fn linux_konsole_gets_a_profile_and_becomes_the_default_and_is_put_back() {
        let f = Fake::new(Os::Linux, "/home/ann");
        f.file(
            "/home/ann/.config/konsolerc",
            "[General]\nConfigVersion=1\n",
        );
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        assert_eq!(s.terminals.len(), 1);
        assert_eq!(s.terminals[0].name, "Konsole");
        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        let profile = fake.text("/home/ann/.local/share/konsole/IdleWalker.profile");
        assert_eq!(
            ini_get(&profile, "Appearance", "Font").as_deref(),
            Some("MesloLGM Nerd Font,11,-1,5,50,0,0,0,0,0")
        );
        assert_eq!(
            ini_get(&profile, "General", "ParentProfile").as_deref(),
            Some("FALLBACK/")
        );
        let rc = fake.text("/home/ann/.config/konsolerc");
        assert_eq!(
            ini_get(&rc, "Desktop Entry", "DefaultProfile").as_deref(),
            Some("IdleWalker.profile")
        );
        assert!(posh.status().terminals[0].font_set);

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        assert!(!fake.exists(Path::new(
            "/home/ann/.local/share/konsole/IdleWalker.profile"
        )));
        assert_eq!(
            fake.text("/home/ann/.config/konsolerc"),
            "[General]\nConfigVersion=1\n"
        );
    }

    #[test]
    fn linux_gnome_terminal_and_kitty_are_set_in_every_profile_and_put_back() {
        let mut f = Fake::new(Os::Linux, "/home/ann");
        f.program("gsettings", "/usr/bin/gsettings");
        f.program("kitty", "/usr/bin/kitty");
        f.answer(
            "gsettings",
            "get org.gnome.Terminal.ProfilesList list",
            "['aaa', 'bbb']\n",
        );
        for id in ["aaa", "bbb"] {
            let p = format!("org.gnome.Terminal.Legacy.Profile:/org/gnome/terminal/legacy/profiles:/:{id}/");
            f.answer("gsettings", &format!("get {p} use-system-font"), "true\n");
            f.answer("gsettings", &format!("get {p} font"), "'Monospace 12'\n");
        }
        f.answer("gsettings", "set", "");
        f.answer("gsettings", "reset", "");
        f.file("/home/ann/.config/kitty/kitty.conf", "font_size 12\n");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        let names: Vec<&str> = s.terminals.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["GNOME Terminal", "kitty"]);
        assert_eq!(s.terminals[0].detail, "2 profiles");

        let mut log = Vec::new();
        assert!(posh.set_terminal_fonts(&mut log), "{log:?}");
        let runs = fake.runs();
        for id in ["aaa", "bbb"] {
            assert!(runs.iter().any(|r| r.contains(&format!("{id}/ font MesloLGM Nerd Font 11"))), "{runs:?}");
            assert!(runs.iter().any(|r| r.contains(&format!("{id}/ use-system-font false"))));
        }
        assert_eq!(
            fake.text("/home/ann/.config/kitty/kitty.conf"),
            "font_size 12\nfont_family MesloLGM Nerd Font\n"
        );
        assert!(posh.status().terminals[1].font_set);

        let mut log = Vec::new();
        assert!(posh.restore_terminal_fonts(false, &mut log), "{log:?}");
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.ends_with("aaa/ font 'Monospace 12'")), "{runs:?}");
        assert!(runs.iter().any(|r| r.ends_with("bbb/ use-system-font true")));
        assert_eq!(fake.text("/home/ann/.config/kitty/kitty.conf"), "font_size 12\n");
    }

    #[test]
    fn winget_gets_the_flags_each_verb_accepts() {
        let mut f = Fake::new(Os::Windows, r"C:\Users\ann");
        f.env
            .insert("LOCALAPPDATA".into(), r"C:\Users\ann\AppData\Local".into());
        let apps = r"C:\Users\ann\AppData\Local\Microsoft\WindowsApps";
        f.program("winget", &format!(r"{apps}\winget.exe"));
        f.program("oh-my-posh", &format!(r"{apps}\oh-my-posh.exe"));
        f.answer("winget.exe", "install", "ok");
        f.answer("winget.exe", "uninstall", "ok");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        posh.perform(&Task::Uninstall);
        let runs = fake.runs();
        let uninstall = runs
            .iter()
            .find(|r| r.contains("winget.exe uninstall"))
            .expect("winget uninstall ran");
        assert!(!uninstall.contains("--accept-package-agreements"));
        assert!(uninstall.contains("--silent"));
        assert!(uninstall.contains("--accept-source-agreements"));
    }

    #[test]
    fn an_unknown_theme_or_a_missing_install_is_refused() {
        let fake = Arc::new(linux_box());
        let posh = Posh::new(fake.clone());
        let done = posh.perform(&Task::Apply {
            theme: Some("nope".into()),
            shells: vec![Shell::Bash],
        });
        assert!(!done.ok);
        assert_eq!(fake.text("/home/ann/.bashrc"), "export FOO=1\n");

        let bare = Fake::new(Os::Linux, "/home/ann");
        let posh = Posh::new(Arc::new(bare));
        let done = posh.perform(&Task::Apply {
            theme: None,
            shells: vec![Shell::Bash],
        });
        assert!(!done.ok);
        assert!(done.status.binary.is_none());
        assert_eq!(done.status.installer, "the Oh My Posh install script");
    }

    #[test]
    fn the_font_this_program_installed_is_the_one_it_removes() {
        let mut f = linux_box();
        f.file("/home/ann/.local/share/fonts/Other.ttf", "");
        f.answer("oh-my-posh", "font", "done\n");
        for name in ["MesloLGMNerdFont-Regular.ttf", "MesloLGMNerdFontMono-Regular.ttf"] {
            f.drops.push((
                "font".into(),
                PathBuf::from(format!("/home/ann/.local/share/fonts/{name}")),
            ));
        }
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let done = posh.perform(&Task::InstallFont);
        assert!(done.ok, "{:?}", done.log);
        assert!(done.status.font_installed);
        let record = undo_parse(&fake.text("/home/ann/.idlewalker/terminal-undo.txt"));
        assert_eq!(record.len(), 2);
        assert!(record
            .iter()
            .all(|u| u.target.starts_with("font-file:/home/ann/.local/share/fonts/Meslo")));
        // Putting the terminals back keeps the font; uninstalling takes it, and only it.
        let done = posh.perform(&Task::RestoreTerminalFonts);
        assert!(done.ok, "{:?}", done.log);
        assert!(done.status.font_installed && done.status.undo_available);
        let done = posh.perform(&Task::Uninstall);
        assert!(done.ok, "{:?}", done.log);
        assert!(!done.status.font_installed);
        assert!(fake.exists(Path::new("/home/ann/.local/share/fonts/Other.ttf")));
        assert!(!done.status.undo_available);
    }

    // Joined with backslashes, which only a Windows host's paths compare equal to.
    #[cfg(windows)]
    #[test]
    fn windows_knows_its_shells_profiles_and_terminals() {
        let mut f = Fake::new(Os::Windows, r"C:\Users\ann");
        f.env
            .insert("LOCALAPPDATA".into(), r"C:\Users\ann\AppData\Local".into());
        f.env
            .insert("APPDATA".into(), r"C:\Users\ann\AppData\Roaming".into());
        f.env
            .insert("ProgramFiles".into(), r"C:\Program Files".into());
        f.env.insert("SystemRoot".into(), r"C:\Windows".into());
        f.file(
            r"C:\Users\ann\AppData\Local\Microsoft\WindowsApps\oh-my-posh.exe",
            "",
        );
        f.file(r"C:\Program Files\Git\bin\bash.exe", "");
        f.file(
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            "",
        );
        f.file(
            r"C:\Users\ann\AppData\Roaming\Code\User\settings.json",
            "{\n    \"a\": 1\n}\n",
        );
        f.file(
            r"C:\Users\ann\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json",
            "{ \"profiles\": { \"defaults\": {}, \"list\": [] } }",
        );
        f.file(
            r"C:\Users\ann\AppData\Local\Microsoft\Windows\Fonts\MesloLGMNerdFont-Regular.ttf",
            "",
        );
        f.program("reg", r"C:\Windows\System32\reg.exe");
        f.answer(
            "reg.exe",
            "query",
            "\r\nHKEY_CURRENT_USER\\Console\r\n    FaceName    REG_SZ    Consolas\r\n\r\nHKEY_CURRENT_USER\\Console\\%SystemRoot%_system32_cmd.exe\r\n    FaceName    REG_SZ    Lucida Console\r\n    FontSize    REG_DWORD    0xe0000\r\n\r\nHKEY_CURRENT_USER\\Console\\Git Bash\r\n    WindowSize    REG_DWORD    0x1e0078\r\n",
        );
        f.answer("reg.exe", "add", "");
        f.answer("reg.exe", "delete", "");
        f.answers.insert(
            ("powershell.exe".into(), "-NoProfile".into()),
            "C:\\Users\\ann\\OneDrive\\Documents\\WindowsPowerShell\\profile.ps1\r\n".into(),
        );
        f.answer("oh-my-posh.exe", "version", "31.9.1");
        let fake = Arc::new(f);
        let posh = Posh::new(fake.clone());
        let s = posh.status();
        assert_eq!(s.version.as_deref(), Some("31.9.1"));
        assert!(s.font_installed);
        assert!(s.clink.is_none());
        let names: Vec<&str> = s.shells.iter().map(|s| s.shell.name()).collect();
        assert_eq!(names, ["Windows PowerShell", "Bash", "Command Prompt"]);
        assert_eq!(
            s.shells[0].profile,
            PathBuf::from(r"C:\Users\ann\OneDrive\Documents\WindowsPowerShell\profile.ps1")
        );
        assert_eq!(s.shells[1].profile, PathBuf::from(r"C:\Users\ann\.bashrc"));
        assert!(s.shells[2].blocker.is_some(), "cmd needs Clink");
        let kinds: Vec<TerminalKind> = s.terminals.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            [
                TerminalKind::VsCode,
                TerminalKind::WindowsTerminal,
                TerminalKind::ConsoleHost
            ]
        );
        assert!(s.terminals.iter().all(|t| !t.font_set));
        assert_eq!(s.terminals[2].detail, "2 profiles");

        let done = posh.perform(&Task::SetTerminalFonts);
        assert!(done.ok, "{:?}", done.log);
        let code = fake.text(r"C:\Users\ann\AppData\Roaming\Code\User\settings.json");
        assert_eq!(
            code,
            "{\n    \"terminal.integrated.minimumContrastRatio\": 1,\n    \"terminal.integrated.fontFamily\": \"MesloLGM Nerd Font, Consolas, 'Courier New', monospace\",\n    \"a\": 1\n}\n"
        );
        let wt = fake.text(r"C:\Users\ann\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json");
        assert_eq!(wt_status(&wt), (true, 1));
        // The root key and the subkey with a font of its own were set; the other was not.
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.contains(r"add HKEY_CURRENT_USER\Console /v FaceName /t REG_SZ /d MesloLGM Nerd Font Mono /f")), "{runs:?}");
        assert!(runs.iter().any(|r| r.contains(r"add HKEY_CURRENT_USER\Console\%SystemRoot%_system32_cmd.exe /v FaceName")));
        // The debug file holds every run of the task, what it printed, and the state after.
        let debug = fake.text(r"C:\Users\ann\.idlewalker\terminal-debug.txt");
        assert!(debug.starts_with("IdleWalker terminal prompt: SetTerminalFonts\nOS: Windows\n"), "{debug}");
        assert!(debug.contains("$ C:\\Windows\\System32\\reg.exe add HKEY_CURRENT_USER\\Console /v FaceName /t REG_SZ /d 'MesloLGM Nerd Font Mono' /f\n  exit: ok"), "{debug}");
        assert!(debug.contains("---- the state afterwards ----"));
        assert!(done.log.iter().any(|l| l.ends_with("terminal-debug.txt.")), "{:?}", done.log);
        assert!(!runs.iter().any(|r| r.contains(r"add HKEY_CURRENT_USER\Console\Git Bash")));
        assert!(runs.iter().any(|r| r.contains(r"add HKEY_CURRENT_USER\Console /v FontSize")));
        assert!(!runs.iter().any(|r| r.contains(r"cmd.exe /v FontSize")));
        // The record knows what was there, and the console's old font goes back by it.
        let record = undo_parse(&fake.text(r"C:\Users\ann\.idlewalker\terminal-undo.txt"));
        assert!(record.iter().any(|u| u.target == r"console:HKEY_CURRENT_USER\Console"
            && u.key == "FaceName"
            && u.before.as_deref() == Some("REG_SZ Consolas")));
        assert!(record.iter().any(|u| u.target == r"console:HKEY_CURRENT_USER\Console"
            && u.key == "FontSize"
            && u.before.is_none()));
        let done = posh.perform(&Task::RestoreTerminalFonts);
        assert!(done.ok, "{:?}", done.log);
        let runs = fake.runs();
        assert!(runs.iter().any(|r| r.contains(r"add HKEY_CURRENT_USER\Console /v FaceName /t REG_SZ /d Consolas /f")), "{runs:?}");
        assert!(runs.iter().any(|r| r.contains(r"delete HKEY_CURRENT_USER\Console /v FontSize /f")));
        assert_eq!(
            fake.text(r"C:\Users\ann\AppData\Roaming\Code\User\settings.json"),
            "{\n    \"a\": 1\n}\n"
        );
        assert!(!done.status.undo_available);

        // A new PowerShell profile starts with a byte-order mark; cmd is skipped for want of Clink.
        let done = posh.perform(&Task::Apply {
            theme: None,
            shells: vec![Shell::WindowsPowerShell, Shell::Cmd],
        });
        assert!(done.ok, "{:?}", done.log);
        let profile = fake.text(r"C:\Users\ann\OneDrive\Documents\WindowsPowerShell\profile.ps1");
        assert!(profile.starts_with("\u{feff}# >>> IdleWalker"));
        assert!(profile.contains("init pwsh | Invoke-Expression"));
        assert!(done
            .log
            .iter()
            .any(|l| l.starts_with("Command Prompt: needs Clink")));
        // Nothing but the profile question and the policy check ran in PowerShell.
        assert!(fake
            .runs()
            .iter()
            .any(|r| r.contains("Get-ExecutionPolicy")));
    }
}
