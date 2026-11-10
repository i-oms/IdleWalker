//! The terminal prompt window: Oh My Posh's state at a glance, and a button for each step
//! that would otherwise be a terminal command. Install or update it, fetch its themes,
//! pick one, tick the shells it goes into, install the Nerd Font, point the terminals
//! at it, and put them back the way they were. Every step runs on a worker thread and
//! reports back through the event loop; the window shows what it did in a log at the
//! bottom.
use super::dialogs::Outcome;
use super::gfx::{self, button, checkbox, Canvas, Font, Mouse};
use super::{Services, UserEvent};
use idlewalker_application::posh::{
    Cursor, Done, Look, Posh, Shell, Status, Task, DEFAULT_THEME, FONT_NAME, PALETTES, RELEASES,
    THEME_GALLERY,
};
use std::collections::HashMap;
use winit::event_loop::EventLoopProxy;

/// The window's size in device-independent pixels.
pub const SIZE: (f64, f64) = (860.0, 720.0);

pub struct PoshDialog {
    posh: Posh,
    proxy: EventLoopProxy<UserEvent>,
    status: Option<Status>,
    /// The newest release, from the last check.
    latest: Option<String>,
    /// What the worker is doing, while it is.
    busy: Option<&'static str>,
    log: Vec<String>,
    /// The theme picked in the list; `None` is Oh My Posh's built-in default.
    theme: Option<String>,
    /// The first status picks the theme already in use; after that the person does.
    picked: bool,
    wanted: HashMap<Shell, bool>,
    /// Typed into the window, to narrow the theme list.
    pub filter: String,
    scroll: i32,
    /// "Uninstall" was clicked once; the bottom row asks before doing it.
    confirm_uninstall: bool,
    /// The appearance panel is showing, and what is picked in each of its rows.
    show_look: bool,
    look_picks: [usize; 6],
}

impl PoshDialog {
    pub fn new(posh: Posh, proxy: EventLoopProxy<UserEvent>) -> Self {
        let mut d = PoshDialog {
            posh,
            proxy,
            status: None,
            latest: None,
            busy: None,
            log: Vec::new(),
            theme: None,
            picked: false,
            wanted: HashMap::new(),
            filter: String::new(),
            scroll: 0,
            confirm_uninstall: false,
            show_look: false,
            look_picks: [0; 6],
        };
        d.start(Task::Refresh, "Reading the terminal setup...");
        d
    }

    /// Runs a task on a worker; its answer arrives as `UserEvent::Posh`.
    fn start(&mut self, task: Task, what: &'static str) {
        if self.busy.is_some() {
            return;
        }
        self.busy = Some(what);
        let (posh, proxy) = (self.posh.clone(), self.proxy.clone());
        let _ = std::thread::Builder::new()
            .name("IdleWalker terminal prompt".into())
            .spawn(move || {
                let done = posh.perform(&task);
                let _ = proxy.send_event(UserEvent::Posh(Box::new(done)));
            });
    }

    pub fn finish(&mut self, done: Done) {
        self.busy = None;
        let status = done.status;
        if status.latest.is_some() {
            self.latest = status.latest.clone();
        }
        if !self.picked {
            self.picked = true;
            let in_use = status.shells.iter().find_map(|s| s.theme.clone());
            self.theme = in_use.filter(|t| t != DEFAULT_THEME);
        }
        for s in &status.shells {
            self.wanted.entry(s.shell).or_insert(s.blocker.is_none());
        }
        self.log.extend(done.log);
        self.status = Some(status);
    }

    pub fn draw(
        &mut self,
        c: &mut Canvas,
        f: &mut Font,
        m: &Mouse,
        s: f64,
        services: &Services,
    ) -> Outcome {
        if self.show_look {
            self.draw_look(c, f, m, s);
            return Outcome::None;
        }
        c.clear(gfx::BACK);
        let px = |v: f64| (v * s) as i32;
        let (title, text, small) = ((18.0 * s) as f32, (13.0 * s) as f32, (12.0 * s) as f32);
        let left = px(18.0);
        let width = c.w - 2 * left;
        let (bh, line) = (px(28.0), px(20.0));
        let idle = self.busy.is_none();
        // Buttons draw while a task runs, but only an idle window takes their clicks.
        let btn = |c: &mut Canvas, f: &mut Font, x: i32, y: i32, w: i32, label: &str| {
            button(c, f, m, x, y, w, bh, s, label) && idle
        };
        let mut next: Option<(Task, &'static str)> = None;

        c.text(
            f,
            left,
            px(14.0),
            "Terminal prompt: Oh My Posh",
            title,
            gfx::ACCENT,
        );
        let status_line = match (&self.busy, &self.status) {
            (Some(what), _) => (what.to_string(), gfx::ACCENT),
            (None, None) => ("Reading...".to_string(), gfx::DIM),
            (None, Some(st)) => match (&st.binary, &st.version) {
                (None, _) => (
                    format!(
                        "Not installed. It would be installed with {}.",
                        st.installer
                    ),
                    gfx::WHITE,
                ),
                (Some(_), v) => {
                    let v = v.as_deref().unwrap_or("unknown version");
                    match &self.latest {
                        Some(l) if st.update_available() => {
                            (format!("Installed: {v}. Newer release: {l}."), gfx::GREEN)
                        }
                        Some(_) => (format!("Installed: {v}, the newest release."), gfx::WHITE),
                        None => (format!("Installed: {v}."), gfx::WHITE),
                    }
                }
            },
        };
        c.text_fit(
            f,
            left,
            px(44.0),
            width,
            &status_line.0,
            text,
            status_line.1,
        );

        // The top row: install or update, the version check, a fresh look.
        let mut x = left;
        let y = px(68.0);
        if let Some(st) = &self.status {
            if st.binary.is_none() {
                if btn(c, f, x, y, px(170.0), "Install Oh My Posh") {
                    next = Some((Task::Install, "Installing Oh My Posh..."));
                }
                x += px(180.0);
            } else {
                let label = if st.update_available() {
                    "Update Oh My Posh"
                } else {
                    "Update anyway"
                };
                if btn(c, f, x, y, px(150.0), label) {
                    next = Some((Task::Update, "Updating Oh My Posh..."));
                }
                x += px(160.0);
                if btn(c, f, x, y, px(150.0), "Check for update") {
                    next = Some((Task::CheckLatest, "Asking GitHub for the newest release..."));
                }
                x += px(160.0);
                if btn(c, f, x, y, px(110.0), "Uninstall...") {
                    self.confirm_uninstall = true;
                }
                x += px(120.0);
            }
        }
        if btn(c, f, x, y, px(90.0), "Refresh") {
            next = Some((Task::Refresh, "Reading the terminal setup..."));
        }
        if btn(
            c,
            f,
            left + width - px(130.0),
            y,
            px(130.0),
            "Releases page",
        ) {
            services.desktop.open_url(RELEASES);
        }

        let top = px(110.0);
        let log_top = c.h - px(118.0);
        let buttons_y = log_top - px(40.0);
        let (col_w, gap) = (px(270.0), px(24.0));
        let right = left + col_w + gap;
        let right_w = left + width - right;

        // ---- The themes, down the left ----
        c.text(f, left, top, "Theme", (14.0 * s) as f32, gfx::WHITE);
        let hint = if self.filter.is_empty() {
            "Type to narrow the list".to_string()
        } else {
            format!("Showing: {}", self.filter)
        };
        c.text_right(f, left + col_w, top + px(2.0), &hint, small, gfx::DIM);
        let list_top = top + px(26.0);
        let list_bottom = buttons_y - px(40.0);
        let list_h = list_bottom - list_top;
        c.rect(left, list_top, col_w, list_h, gfx::PANEL);
        let mut names: Vec<Option<String>> = vec![None];
        if let Some(st) = &self.status {
            let filter = self.filter.to_lowercase();
            names.extend(
                st.themes
                    .iter()
                    .filter(|t| t.name.to_lowercase().contains(&filter))
                    .map(|t| Some(t.name.clone())),
            );
        }
        let row_h = line;
        let max_scroll = (names.len() as i32 * row_h - list_h).max(0);
        if m.inside(left, list_top, col_w, list_h) {
            self.scroll =
                (self.scroll - (m.wheel * row_h as f32 * 3.0) as i32).clamp(0, max_scroll);
        }
        self.scroll = self.scroll.min(max_scroll);
        for (n, name) in names.iter().enumerate() {
            let y = list_top + n as i32 * row_h - self.scroll;
            if y + row_h <= list_top || y >= list_bottom {
                continue;
            }
            let hit = m.inside(left, y.max(list_top), col_w, row_h.min(list_bottom - y));
            if m.clicked && hit {
                self.theme = name.clone();
            }
            let chosen = *name == self.theme;
            if chosen {
                c.rect(
                    left,
                    y.max(list_top),
                    col_w,
                    (y + row_h).min(list_bottom) - y.max(list_top),
                    0xFF2C4A7A,
                );
            } else if hit {
                c.rect(
                    left,
                    y.max(list_top),
                    col_w,
                    (y + row_h).min(list_bottom) - y.max(list_top),
                    gfx::RIM,
                );
            }
            if y < list_top || y + row_h > list_bottom {
                continue;
            }
            let label = name.as_deref().unwrap_or("Default (built in)");
            c.text_fit(
                f,
                left + px(8.0),
                y + px(2.0),
                col_w - px(16.0),
                label,
                text,
                gfx::WHITE,
            );
        }
        if max_scroll > 0 {
            let thumb = (list_h * list_h / (names.len() as i32 * row_h)).max(px(20.0));
            let ty = list_top + (list_h - thumb) * self.scroll / max_scroll;
            c.round_box(
                left + col_w - px(5.0),
                ty,
                px(3.0),
                thumb,
                1.5 * s,
                gfx::DIM,
                0,
                0.0,
            );
        }
        let y = list_bottom + px(8.0);
        match &self.status {
            Some(st) if st.binary.is_some() && st.themes.is_empty() => {
                if btn(c, f, left, y, px(150.0), "Download themes") {
                    next = Some((Task::DownloadThemes, "Downloading the themes..."));
                }
            }
            _ => {
                if btn(c, f, left, y, px(150.0), "Theme gallery online") {
                    services.desktop.open_url(THEME_GALLERY);
                }
            }
        }
        if let Some(st) = &self.status {
            if st.themes_dir.is_some() {
                c.text_right(
                    f,
                    left + col_w,
                    y + px(6.0),
                    &format!("{} themes", st.themes.len()),
                    small,
                    gfx::DIM,
                );
            }
        }

        // ---- The shells, the font and the terminals, down the right ----
        let mut y = top;
        c.text(
            f,
            right,
            y,
            "Shells it goes into",
            (14.0 * s) as f32,
            gfx::WHITE,
        );
        y += px(26.0);
        if let Some(st) = &self.status {
            if st.shells.is_empty() {
                c.text(f, right, y, "No shell found.", text, gfx::DIM);
                y += line;
            }
            let mut toggled = None;
            for sh in &st.shells {
                let on = self.wanted.get(&sh.shell).copied().unwrap_or(false);
                let enabled = sh.blocker.is_none() && idle;
                let label = match (&sh.blocker, sh.configured, &sh.theme) {
                    (Some(b), _, _) => format!("{} ({b})", sh.shell.name()),
                    (None, true, Some(t)) => format!("{} (set: {t})", sh.shell.name()),
                    (None, true, None) => format!("{} (set)", sh.shell.name()),
                    (None, false, _) => sh.shell.name().to_string(),
                };
                let label_w = px(250.0);
                if checkbox(c, f, m, right, y, label_w, px(22.0), s, &label, on, enabled) {
                    toggled = Some(sh.shell);
                }
                c.text_fit(
                    f,
                    right + label_w + px(8.0),
                    y + px(4.0),
                    right_w - label_w - px(8.0),
                    &sh.shown,
                    small,
                    gfx::DIM,
                );
                y += px(22.0);
            }
            if let Some(shell) = toggled {
                let e = self.wanted.entry(shell).or_insert(false);
                *e = !*e;
            }
            if st.shells.iter().any(|sh| sh.shell == Shell::Cmd) && st.clink.is_none() {
                y += px(4.0);
                if btn(c, f, right, y, px(170.0), "Install Clink for cmd") {
                    next = Some((Task::InstallClink, "Installing Clink..."));
                }
                c.text_fit(
                    f,
                    right + px(180.0),
                    y + px(6.0),
                    right_w - px(180.0),
                    "cmd has no prompt of its own; Clink adds one.",
                    small,
                    gfx::DIM,
                );
                y += bh + px(4.0);
            }

            // The font.
            y += px(10.0);
            c.text(f, right, y, "Font", (14.0 * s) as f32, gfx::WHITE);
            y += px(24.0);
            let (font_line, colour) = if st.font_installed {
                (format!("{FONT_NAME} is installed."), gfx::GREEN)
            } else {
                (
                    format!("{FONT_NAME} is not installed; the prompt's icons need it."),
                    gfx::WHITE,
                )
            };
            c.text_fit(
                f,
                right,
                y + px(6.0),
                right_w - px(180.0),
                &font_line,
                text,
                colour,
            );
            let font_label = if st.font_installed {
                "Reinstall font"
            } else {
                "Install Nerd Font"
            };
            if btn(c, f, right + right_w - px(170.0), y, px(170.0), font_label) {
                next = Some((Task::InstallFont, "Installing the Nerd Font..."));
            }
            y += bh + px(10.0);

            // The terminals.
            c.text(f, right, y, "Terminals", (14.0 * s) as f32, gfx::WHITE);
            y += px(24.0);
            if st.terminals.is_empty() {
                c.text_fit(
                    f,
                    right,
                    y,
                    right_w,
                    "No terminal settings found to change.",
                    text,
                    gfx::DIM,
                );
                y += line;
            }
            // A long list goes in two columns, so it stays above the buttons.
            let row = px(18.0);
            let cols: usize = if st.terminals.len() > 8 { 2 } else { 1 };
            let col_w = right_w / cols as i32;
            for (i, t) in st.terminals.iter().enumerate() {
                let by_hand = t.kind == idlewalker_application::posh::TerminalKind::ByHand;
                let (what, colour) = if by_hand {
                    ("set the font yourself", gfx::DIM)
                } else if t.font_set {
                    ("uses the Nerd Font", gfx::GREEN)
                } else {
                    ("not using the Nerd Font yet", gfx::WHITE)
                };
                let detail = if t.detail.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", t.detail)
                };
                let x = right + col_w * (i % cols) as i32;
                c.text_fit(
                    f,
                    x,
                    y,
                    col_w - px(8.0),
                    &format!("{}: {what}{detail}", t.name),
                    text,
                    colour,
                );
                if i % cols == cols - 1 || i + 1 == st.terminals.len() {
                    y += row;
                }
            }
            c.text_fit(f, right, y, right_w, st.terminal_note, small, gfx::DIM);
            y += line;
            let settable = st
                .terminals
                .iter()
                .any(|t| {
                    !matches!(
                        t.kind,
                        idlewalker_application::posh::TerminalKind::BuiltIn
                            | idlewalker_application::posh::TerminalKind::ByHand
                    )
                });
            if settable && btn(c, f, right, y, px(220.0), "Use the font in these terminals") {
                next = Some((Task::SetTerminalFonts, "Setting the terminal fonts..."));
            }
            if st.undo_available
                && btn(
                    c,
                    f,
                    right + px(230.0),
                    y,
                    px(190.0),
                    "Put the terminals back",
                )
            {
                next = Some((
                    Task::RestoreTerminalFonts,
                    "Putting the terminal settings back...",
                ));
            }
        }

        // ---- The bottom row ----
        let ticked: Vec<Shell> = self
            .status
            .iter()
            .flat_map(|st| st.shells.iter())
            .filter(|sh| {
                sh.blocker.is_none() && self.wanted.get(&sh.shell).copied().unwrap_or(false)
            })
            .map(|sh| sh.shell)
            .collect();
        // Decided once per paint: a "Yes" must not fall through to the button that takes
        // the question's place, since the click is still live for the rest of this paint.
        let confirming = self.confirm_uninstall;
        if confirming {
            // Asked here, in place of the apply row, as the brains dashboard does.
            let question =
                "Take the prompt out of every shell, put the terminals back and uninstall Oh My Posh?";
            c.text(f, left, buttons_y + px(6.0), question, text, gfx::ACCENT);
            let qx = left + f.width(question, text) + px(14.0);
            if btn(c, f, qx, buttons_y, px(120.0), "Yes, uninstall") {
                self.confirm_uninstall = false;
                next = Some((Task::Uninstall, "Uninstalling Oh My Posh..."));
            }
            if btn(c, f, qx + px(130.0), buttons_y, px(60.0), "No") {
                self.confirm_uninstall = false;
            }
        }
        let apply_label = match &self.theme {
            Some(t) => format!("Apply \"{t}\" to the ticked shells"),
            None => "Apply the default theme to the ticked shells".to_string(),
        };
        let apply_w = f.width(&apply_label, text) + px(28.0);
        if confirming {
            // The question stands where these buttons were.
        } else if btn(c, f, left, buttons_y, apply_w, &apply_label) {
            if ticked.is_empty() {
                self.log.push("Tick at least one shell first.".into());
            } else {
                next = Some((
                    Task::Apply {
                        theme: self.theme.clone(),
                        shells: ticked,
                    },
                    "Writing the shell profiles...",
                ));
            }
        }
        if !confirming
            && btn(
                c,
                f,
                left + apply_w + px(10.0),
                buttons_y,
                px(160.0),
                "Remove from all shells",
            )
        {
            next = Some((Task::Remove, "Taking the prompt out of the profiles..."));
        }
        if !confirming
            && btn(
                c,
                f,
                left + width - px(210.0),
                buttons_y,
                px(110.0),
                "Appearance...",
            )
        {
            self.show_look = true;
        }
        if btn(c, f, left + width - px(90.0), buttons_y, px(90.0), "Close") {
            return Outcome::Close;
        }

        // ---- The log ----
        let log_h = c.h - log_top - px(12.0);
        c.rect(left, log_top, width, log_h, gfx::PANEL);
        let log_line = px(17.0);
        let shown = ((log_h - px(8.0)) / log_line).max(1) as usize;
        let start = self.log.len().saturating_sub(shown);
        for (n, entry) in self.log[start..].iter().enumerate() {
            let colour = if entry.starts_with("    ") {
                gfx::DIM
            } else {
                gfx::WHITE
            };
            c.text_fit(
                f,
                left + px(8.0),
                log_top + px(4.0) + n as i32 * log_line,
                width - px(16.0),
                entry,
                small,
                colour,
            );
        }
        if self.log.is_empty() {
            c.text(
                f,
                left + px(8.0),
                log_top + px(4.0),
                "What each step does is reported here.",
                small,
                gfx::DIM,
            );
        }

        if let Some((task, what)) = next {
            self.start(task, what);
        }
        Outcome::None
    }

    /// The shells a default can be picked from, in the order of the last row's choices.
    fn look_shells(&self) -> Vec<Shell> {
        self.status
            .as_ref()
            .map(|st| {
                st.shells
                    .iter()
                    .filter(|s| s.blocker.is_none() && s.shell != Shell::Wsl)
                    .map(|s| s.shell)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The panel with font size, colours, cursor, opacity, ligatures and default shell.
    /// "Keep" leaves a setting as each terminal has it.
    fn draw_look(&mut self, c: &mut Canvas, f: &mut Font, m: &Mouse, s: f64) {
        const KEEP: &str = "Keep";
        const SIZES: [u32; 10] = [9, 10, 11, 12, 13, 14, 15, 16, 18, 20];
        const OPACITIES: [u32; 6] = [100, 95, 90, 85, 80, 70];
        c.clear(gfx::BACK);
        let px = |v: f64| (v * s) as i32;
        let (title, text, small) = ((18.0 * s) as f32, (13.0 * s) as f32, (12.0 * s) as f32);
        let left = px(18.0);
        let width = c.w - 2 * left;
        let bh = px(28.0);
        let idle = self.busy.is_none();
        let shells = self.look_shells();
        let with_keep = |rest: Vec<String>| {
            let mut v = vec![KEEP.to_string()];
            v.extend(rest);
            v
        };
        let rows: [(&str, Vec<String>); 6] = [
            ("Font size", with_keep(SIZES.iter().map(u32::to_string).collect())),
            ("Colours", with_keep(PALETTES.iter().map(|p| p.name.to_string()).collect())),
            ("Cursor", with_keep(vec!["Block".into(), "Bar".into(), "Underline".into()])),
            ("Opacity", with_keep(OPACITIES.iter().map(|o| format!("{o}%")).collect())),
            ("Ligatures", with_keep(vec!["On".into(), "Off".into()])),
            ("Default shell", with_keep(shells.iter().map(|s| s.name().to_string()).collect())),
        ];
        c.text(f, left, px(14.0), "Terminal appearance", title, gfx::ACCENT);
        let what = self.busy.unwrap_or("Applies to every terminal listed that has the setting; the log says what each one took.");
        c.text_fit(f, left, px(44.0), width, what, small, gfx::DIM);
        let mut y = px(76.0);
        let (arrow, value_w) = (px(32.0), px(260.0));
        for (i, (label, options)) in rows.iter().enumerate() {
            c.text(f, left, y + px(6.0), label, text, gfx::WHITE);
            let x = left + px(130.0);
            let n = options.len();
            let pick = self.look_picks[i] % n;
            if button(c, f, m, x, y, arrow, bh, s, "<") && idle {
                self.look_picks[i] = (pick + n - 1) % n;
            }
            let colour = if pick == 0 { gfx::DIM } else { gfx::WHITE };
            let shown = self.look_picks[i] % n;
            c.text_fit(f, x + arrow + px(12.0), y + px(6.0), value_w - px(24.0), &options[shown], text, colour);
            if button(c, f, m, x + arrow + value_w, y, arrow, bh, s, ">") && idle {
                self.look_picks[i] = (pick + 1) % n;
            }
            y += bh + px(8.0);
        }
        c.text_fit(
            f,
            left,
            y + px(4.0),
            width,
            "Open terminals pick most of this up when reopened. MesloLGM has no ligatures, so that row only matters with another font.",
            small,
            gfx::DIM,
        );
        y += px(30.0);
        let p = self.look_picks;
        let nth = |i: usize, len: usize| (p[i] % (len + 1)).checked_sub(1);
        let look = Look {
            font_size: nth(0, SIZES.len()).map(|i| SIZES[i]),
            colours: nth(1, PALETTES.len()),
            cursor: nth(2, 3).map(|i| [Cursor::Block, Cursor::Bar, Cursor::Underline][i]),
            opacity: nth(3, OPACITIES.len()).map(|i| OPACITIES[i]),
            ligatures: nth(4, 2).map(|i| i == 0),
            default_shell: nth(5, shells.len()).map(|i| shells[i]),
        };
        if button(c, f, m, left, y, px(170.0), bh, s, "Apply to the terminals") && idle {
            self.start(Task::SetLook(look), "Changing how the terminals look...");
        }
        if button(c, f, m, left + px(180.0), y, px(190.0), bh, s, "Put the terminals back") && idle {
            self.start(Task::RestoreTerminalFonts, "Putting the terminals back...");
        }
        if button(c, f, m, left + width - px(90.0), y, px(90.0), bh, s, "Back") {
            self.show_look = false;
        }
        y += bh + px(12.0);
        let log_h = c.h - y - px(12.0);
        c.rect(left, y, width, log_h, gfx::PANEL);
        let log_line = px(17.0);
        let shown = ((log_h - px(8.0)) / log_line).max(1) as usize;
        let start = self.log.len().saturating_sub(shown);
        for (n, entry) in self.log[start..].iter().enumerate() {
            c.text_fit(f, left + px(8.0), y + px(4.0) + n as i32 * log_line, width - px(16.0), entry, small, gfx::WHITE);
        }
    }
}
