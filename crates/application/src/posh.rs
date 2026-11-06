//! The terminal prompt: Oh My Posh, set up from a window instead of a dozen terminal
//! commands. Finds or installs it and keeps it current; fetches its themes; puts a theme
//! into every shell on the machine (PowerShell 7, Windows PowerShell, bash, zsh, fish,
//! cmd through Clink, nu), so it shows in every terminal that runs them, VS Code's
//! included; installs the Nerd Font its glyphs need; and tells VS Code, Windows Terminal
//! and GNOME Terminal to use that font. What it writes into a profile sits between two
//! marker lines, so it can be replaced or taken out again without touching anything
//! else. Everything it does to the machine goes through the `Workstation` port, and all
//! of it is slow: run it off the UI thread.
use crate::jsonc;
use crate::ports::{Os, RunResult, Workstation};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const RELEASES: &str = "https://github.com/JanDeDobbeleer/oh-my-posh/releases";
pub const THEME_GALLERY: &str = "https://ohmyposh.dev/docs/themes";
const THEMES_ZIP: &str = "https://cdn.ohmyposh.dev/releases/latest/themes.zip";
const INSTALL_SH: &str = "https://ohmyposh.dev/install.sh";
const INSTALL_PS1: &str = "https://ohmyposh.dev/install.ps1";
pub const FONT_NAME: &str = "MesloLGM Nerd Font";
const FONT_PACKAGE: &str = "meslo";
const WINGET_ID: &str = "JanDeDobbeleer.OhMyPosh";
const CLINK_WINGET_ID: &str = "chrisant996.Clink";
const BEGIN: &str = ">>> IdleWalker: Oh My Posh >>>";
const END: &str = "<<< IdleWalker: Oh My Posh <<<";
/// The theme line's word for Oh My Posh's built-in default.
pub const DEFAULT_THEME: &str = "(default)";

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
    GnomeTerminal,
}

#[derive(Clone, Debug)]
pub struct TerminalStatus {
    pub name: String,
    pub kind: TerminalKind,
    /// The settings file, or the GNOME profile id.
    pub target: String,
    pub font_set: bool,
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
    /// Windows only: Clink, which gives cmd its prompt.
    pub clink: Option<PathBuf>,
    /// How an install or update would be done.
    pub installer: &'static str,
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
    /// The prompt out of every shell, then Oh My Posh off the machine.
    Uninstall,
    InstallFont,
    SetTerminalFonts,
    InstallClink,
}

/// What a task did, and the state after it.
#[derive(Clone, Debug)]
pub struct Done {
    pub ok: bool,
    pub log: Vec<String>,
    pub status: Status,
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

/// The line that starts the prompt in a shell's profile.
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

// ---- The use case ----------------------------------------------------------------------

#[derive(Clone)]
pub struct Posh {
    ws: Arc<dyn Workstation>,
}

impl Posh {
    pub fn new(ws: Arc<dyn Workstation>) -> Self {
        Posh { ws }
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

    fn run(&self, program: &Path, args: &[&str]) -> RunResult {
        let p = program.display().to_string();
        self.ws.run(&p, args)
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

    /// Where Oh My Posh keeps its cache; the themes go beside it.
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
    /// answer follows a redirected Documents folder or a custom config location.
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
            Shell::Cmd => self.local_app_data().join("clink").join("oh-my-posh.lua"),
            Shell::Nu => asked(&["-c", "$nu.config-path"]).unwrap_or_else(|| match self.os() {
                Os::Windows => self.app_data().join("nushell").join("config.nu"),
                Os::MacOs => home
                    .join("Library")
                    .join("Application Support")
                    .join("nushell")
                    .join("config.nu"),
                Os::Linux => self.config_home().join("nushell").join("config.nu"),
            }),
        }
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
        ShellStatus {
            shell,
            profile,
            configured: body.is_some(),
            theme: body.and_then(|b| theme_in(b, shell.comment())),
            blocker: (shell == Shell::Cmd && clink.is_none())
                .then(|| "needs Clink: install it below".to_string()),
        }
    }

    fn shells(&self, clink: Option<&Path>) -> Vec<ShellStatus> {
        self.shell_programs()
            .into_iter()
            .filter_map(|(shell, program)| program.map(|p| self.shell_status(shell, &p, clink)))
            .collect()
    }

    // ---- The font and the terminals ----

    fn font_installed(&self) -> bool {
        let home = self.home();
        let dirs: Vec<PathBuf> = match self.os() {
            Os::Windows => vec![
                self.local_app_data()
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
                self.windir().join("Fonts"),
            ],
            Os::MacOs => vec![
                home.join("Library").join("Fonts"),
                PathBuf::from("/Library/Fonts"),
            ],
            Os::Linux => vec![
                home.join(".local").join("share").join("fonts"),
                home.join(".fonts"),
                PathBuf::from("/usr/share/fonts"),
                PathBuf::from("/usr/local/share/fonts"),
            ],
        };
        dirs.iter().any(|d| {
            self.ws.list_files(d, 4).iter().any(|f| {
                let name = f
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase().replace(' ', ""))
                    .unwrap_or_default();
                name.contains("meslo") && name.contains("nerdfont")
            })
        })
    }

    fn terminals(&self) -> Vec<TerminalStatus> {
        let mut out = Vec::new();
        let home = self.home();
        let editors = ["Code", "Code - Insiders", "VSCodium", "Cursor", "Windsurf"];
        let shown = [
            "VS Code",
            "VS Code Insiders",
            "VSCodium",
            "Cursor",
            "Windsurf",
        ];
        for (folder, name) in editors.iter().zip(shown) {
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
            out.push(TerminalStatus {
                name: name.into(),
                kind: TerminalKind::VsCode,
                target: settings.display().to_string(),
                font_set: jsonc::get_string(&text, &["terminal.integrated.fontFamily"])
                    .is_some_and(|f| f.contains("Nerd Font")),
            });
        }
        if self.os() == Os::Windows {
            let packages = self.local_app_data().join("Packages");
            let candidates = [
                (
                    "Windows Terminal",
                    packages
                        .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
                        .join("LocalState")
                        .join("settings.json"),
                ),
                (
                    "Windows Terminal Preview",
                    packages
                        .join("Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe")
                        .join("LocalState")
                        .join("settings.json"),
                ),
                (
                    "Windows Terminal",
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
                out.push(TerminalStatus {
                    name: name.into(),
                    kind: TerminalKind::WindowsTerminal,
                    target: settings.display().to_string(),
                    font_set: jsonc::get_string(&text, &["profiles", "defaults", "font", "face"])
                        .is_some_and(|f| f.contains("Nerd Font")),
                });
            }
        }
        if self.os() == Os::Linux {
            if let Some(gsettings) = self.ws.find_program("gsettings") {
                let id = self.ask(
                    &gsettings,
                    &["get", "org.gnome.Terminal.ProfilesList", "default"],
                );
                if let Some(id) = id {
                    let uuid = id.trim().trim_matches('\'').to_string();
                    let font = self
                        .ask(&gsettings, &["get", &gnome_profile(&uuid), "font"])
                        .unwrap_or_default();
                    out.push(TerminalStatus {
                        name: "GNOME Terminal".into(),
                        kind: TerminalKind::GnomeTerminal,
                        target: uuid,
                        font_set: font.contains("Nerd Font"),
                    });
                }
            }
        }
        out
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
            installer: self.installer(),
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
            Task::InstallClink => self.install_clink(&mut log),
        };
        let mut status = self.status();
        status.latest = latest;
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

    /// The MSIX build from winget or the Store, taken off through PowerShell when winget
    /// itself will not: it is the same package either way.
    fn remove_appx(&self, log: &mut Vec<String>) -> bool {
        let Some(ps) = self.shell_programs().into_iter().find_map(|(s, p)| {
            (s == Shell::WindowsPowerShell || s == Shell::Pwsh)
                .then_some(p)
                .flatten()
        }) else {
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
                let Some(ps) = self.shell_programs().into_iter().find_map(|(s, p)| {
                    (s == Shell::WindowsPowerShell || s == Shell::Pwsh)
                        .then_some(p)
                        .flatten()
                }) else {
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

    /// Takes the prompt out of every shell, then Oh My Posh off the machine the way it
    /// came: the package manager that installed it, or the files the install script put
    /// there. Its theme cache goes too; the font stays, since other programs may use it.
    fn uninstall(&self, log: &mut Vec<String>) -> bool {
        self.remove(log);
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
                // The install script's layout: Programs\oh-my-posh\{bin,themes}. winget's
                // older, non-MSIX builds lived there too and are listed by it.
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
                    "Removed the cache and themes at {}.",
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
                log.push(format!(
                    "Oh My Posh is uninstalled. {FONT_NAME} was left installed."
                ));
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
                let Some(ps) = self.shell_programs().into_iter().find_map(|(s, p)| {
                    (s == Shell::WindowsPowerShell || s == Shell::Pwsh)
                        .then_some(p)
                        .flatten()
                }) else {
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
        log.push(format!("Installing {FONT_NAME}..."));
        let r = self.run(&binary, &["font", "install", FONT_PACKAGE]);
        log.extend(tail(&r));
        if self.font_installed() {
            log.push(format!("{FONT_NAME} is installed."));
            true
        } else {
            log.push("The font did not install. Check the lines above.".into());
            false
        }
    }

    fn set_terminal_fonts(&self, log: &mut Vec<String>) -> bool {
        let mut any = false;
        for t in self.terminals() {
            match t.kind {
                TerminalKind::VsCode | TerminalKind::WindowsTerminal => {
                    let path = PathBuf::from(&t.target);
                    let text = if self.ws.exists(&path) {
                        match self.ws.read_text(&path) {
                            Some(text) => text,
                            None => {
                                log.push(format!("{}: could not read {}.", t.name, t.target));
                                continue;
                            }
                        }
                    } else {
                        String::new()
                    };
                    let keys: &[&str] = if t.kind == TerminalKind::VsCode {
                        &["terminal.integrated.fontFamily"]
                    } else {
                        &["profiles", "defaults", "font", "face"]
                    };
                    match jsonc::set_string(&text, keys, FONT_NAME)
                        .and_then(|new| self.ws.write_text(&path, &new))
                    {
                        Ok(()) => {
                            any = true;
                            log.push(format!("{}: terminal font set to {FONT_NAME}.", t.name));
                        }
                        Err(e) => log.push(format!("{}: not changed: {e}.", t.name)),
                    }
                }
                TerminalKind::GnomeTerminal => {
                    let Some(gsettings) = self.ws.find_program("gsettings") else {
                        continue;
                    };
                    let profile = gnome_profile(&t.target);
                    let a = self.run(&gsettings, &["set", &profile, "use-system-font", "false"]);
                    let font = format!("{FONT_NAME} 11");
                    let b = self.run(&gsettings, &["set", &profile, "font", &font]);
                    if a.success && b.success {
                        any = true;
                        log.push(format!("GNOME Terminal: font set to {FONT_NAME}."));
                    } else {
                        log.push("GNOME Terminal: could not set the font.".into());
                    }
                }
            }
        }
        if !any {
            log.push("No terminal settings found to change.".into());
        }
        log.push(format!(
            "Other terminals: pick \"{FONT_NAME}\" in their own settings."
        ));
        any
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
            "PowerShell's execution policy blocks profiles; run `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser` yourself.".into()
        });
    }

    /// macOS opens login shells, which read .bash_profile and not .bashrc.
    fn link_bash_profile(&self, log: &mut Vec<String>) {
        let path = self.home().join(".bash_profile");
        let text = self.ws.read_text(&path).unwrap_or_default();
        if text.contains(".bashrc") {
            return;
        }
        let new = with_block(&text, "#", "[ -f ~/.bashrc ] && . ~/.bashrc");
        if self.ws.write_text(&path, &new).is_ok() {
            log.push(
                "Bash: .bash_profile now loads .bashrc, as Terminal opens login shells.".into(),
            );
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
            if os == Os::MacOs && shell == Shell::Bash {
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
        if self.os() == Os::MacOs {
            files.push(("Bash".into(), self.home().join(".bash_profile"), "#"));
        }
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
        if !any {
            log.push("Nothing to remove: no shell was set up here.".into());
        }
        any
    }
}

fn gnome_profile(uuid: &str) -> String {
    format!("org.gnome.Terminal.Legacy.Profile:/org/gnome/terminal/legacy/profiles:/:{uuid}/")
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
        /// By program file name and first argument.
        answers: HashMap<(String, String), String>,
        runs: Mutex<Vec<String>>,
        env: HashMap<String, String>,
    }

    impl Fake {
        fn new(os: Os, home: &str) -> Self {
            Fake {
                os,
                home: PathBuf::from(home),
                files: Mutex::default(),
                programs: HashMap::new(),
                answers: HashMap::new(),
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
            let key = (name, args.first().map_or("", |a| a).to_string());
            match self.answers.get(&key) {
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
        f.programs.insert(
            "oh-my-posh".into(),
            "/home/ann/.local/bin/oh-my-posh".into(),
        );
        f.answers
            .insert(("oh-my-posh".into(), "version".into()), "26.9.0\n".into());
        f.answers.insert(
            ("oh-my-posh".into(), "cache".into()),
            "/home/ann/.cache/oh-my-posh\n".into(),
        );
        f.file("/home/ann/.local/bin/oh-my-posh", "");
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
            .any(|l| l.starts_with("Removed the cache and themes")));
        assert!(done.status.themes.is_empty());

        // Nothing installed: nothing to do, and no complaint.
        let bare = Posh::new(Arc::new(Fake::new(Os::Linux, "/home/ann")));
        assert!(bare.perform(&Task::Uninstall).ok);
    }

    #[test]
    fn winget_gets_the_flags_each_verb_accepts() {
        let mut f = Fake::new(Os::Windows, r"C:\Users\ann");
        f.env
            .insert("LOCALAPPDATA".into(), r"C:\Users\ann\AppData\Local".into());
        let apps = r"C:\Users\ann\AppData\Local\Microsoft\WindowsApps";
        f.programs
            .insert("winget".into(), format!(r"{apps}\winget.exe").into());
        f.programs.insert(
            "oh-my-posh".into(),
            format!(r"{apps}\oh-my-posh.exe").into(),
        );
        f.file(&format!(r"{apps}\winget.exe"), "");
        f.file(&format!(r"{apps}\oh-my-posh.exe"), "");
        f.answers
            .insert(("winget.exe".into(), "install".into()), "ok".into());
        f.answers
            .insert(("winget.exe".into(), "uninstall".into()), "ok".into());
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
        f.answers.insert(
            ("powershell.exe".into(), "-NoProfile".into()),
            "C:\\Users\\ann\\OneDrive\\Documents\\WindowsPowerShell\\profile.ps1\r\n".into(),
        );
        f.answers
            .insert(("oh-my-posh.exe".into(), "version".into()), "31.9.1".into());
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
        assert_eq!(s.terminals.len(), 2);
        assert!(s.terminals.iter().all(|t| !t.font_set));

        let done = posh.perform(&Task::SetTerminalFonts);
        assert!(done.ok, "{:?}", done.log);
        assert!(done.status.terminals.iter().all(|t| t.font_set));
        let code = fake.text(r"C:\Users\ann\AppData\Roaming\Code\User\settings.json");
        assert_eq!(
            code,
            "{\n    \"terminal.integrated.fontFamily\": \"MesloLGM Nerd Font\",\n    \"a\": 1\n}\n"
        );

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
