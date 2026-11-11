# Developing IdleWalker

Everything you need to build, test, port-check and release IdleWalker. For what it is and how to install it, see the [README](../README.md).

## Build and run

```sh
cargo build --release
target/release/idlewalker          # idlewalker.exe on Windows
cargo test --release --test selftest
```

## Where things are kept

| OS      | Folder                                       | "Start at sign-in"                                  |
| ------- | -------------------------------------------- | --------------------------------------------------- |
| Windows | `%APPDATA%\IdleWalker`                     | the `IdleWalker` value of the HKCU `Run` key    |
| macOS   | `~/Library/Application Support/IdleWalker` | `~/Library/LaunchAgents/com.idlewalker.app.plist` |
| Linux   | `~/.config/idlewalker`                     | `~/.config/autostart/idlewalker.desktop`          |

The folder holds `settings.txt` (menu choices) and `brains.txt` (what the walkers learned).
The terminal prompt window keeps `terminal-undo.txt` there (every terminal setting it changed,
with the old value) and, after each font task, `terminal-debug.txt`: every program it ran with
what that printed, for working out why a terminal did not take the font.
`brains.txt` has the same format and, on Windows, the same location as the C# version, so
an existing cast keeps its memories; menu choices not yet in `settings.txt` are read from
the C# version's registry key. Memories of walkers no longer in the cast are ignored.

## Platform differences

The C# version leaned on Windows-only behaviour. Not all of it exists elsewhere:

| Feature                               | Windows      | macOS                                                             | Linux on X11                                                  | Linux on Wayland (via XWayland)                                                     |
| ------------------------------------- | ------------ | ----------------------------------------------------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Walkers, click-through, always on top | Yes          | Yes                                                               | Yes (needs a compositor)                                      | Yes, above X11 and most native windows                                              |
| Idle detection from the cursor        | Yes          | Yes                                                               | Yes                                                           | **Limited**: Wayland hides the cursor while it is over native Wayland windows |
| Hot corners                           | Yes          | Yes                                                               | Yes                                                           | **Limited**, for the same reason                                              |
| Click highlight                       | Yes          | Yes                                                               | Yes; a click shorter than the 30 ms poll can be missed        | **Limited**, for the same reason                                              |
| Find cursor (a key pressed twice)     | Yes          | Yes                                                               | Yes; a tap shorter than the 30 ms poll can be missed          | **Limited**: keys are only seen while an X11 window has the focus             |
| Corner actions                        | All          | Keys need the Accessibility permission                            | By desktop shortcuts: GNOME and KDE defaults, others may vary | Not reliable                                                                        |
| Full-screen detection                 | Yes          | Active app only                                                   | Yes (needs an EWMH window manager)                            | X11 windows only                                                                    |
| Brightness, built-in panel            | Yes          | Yes, through a private framework that a future macOS could remove | Yes (`brightnessctl` or a udev rule)                        | Same as X11                                                                         |
| Brightness, external monitors         | Yes (DDC/CI) | DDC/CI on Apple Silicon (private IOKit calls), otherwise software dimming | With `ddcutil`                                              | With `ddcutil`                                                                    |
| Hidden from screenshots               | Yes          | Only where macOS still honours it                                 | **No**                                                  | **No**                                                                        |
| Anti-idle nudge                       | Yes          | `caffeinate`                                                    | `systemd-inhibit` and the X screen-saver reset              | `systemd-inhibit`                                                                 |
| Terminal prompt: install and update   | winget, else the install script | Homebrew, else the install script into `~/.local/bin` | The install script into `~/.local/bin` (needs `curl` and `unzip`) | Same as X11 |
| Terminal prompt: uninstall            | winget, scoop or Chocolatey as installed, else its folder is deleted | Homebrew, else the binary is deleted | The binary is deleted; one under `/usr/local` needs `sudo`, which it says | Same as X11 |
| Terminal prompt: shells               | PowerShell 7, Windows PowerShell, Git Bash, cmd (Clink), nu | zsh, bash, fish, PowerShell, nu | bash, zsh, fish, PowerShell, nu | Same as X11 |
| Terminal prompt: terminal font        | VS Code family, Windows Terminal (every profile), the console host (`HKCU\Console` and its subkeys), ConEmu and Cmder (`ConEmu.xml` or `HKCU\Software\ConEmu`, every configuration), PuTTY (every session under `HKCU\Software\SimonTatham\PuTTY\Sessions`), mintty for Git Bash, MSYS2 and Cygwin (each home's `.minttyrc`) and wsltty, Alacritty, Tabby, Hyper, Rio, Warp, Wave, Zed | VS Code family, Terminal (AppleScript, every profile, may ask for Automation permission), iTerm2 (`defaults export`/`import`, every profile, iTerm2 must be closed), kitty, Alacritty, Tabby, Hyper, Rio, Warp, Wave, Zed | VS Code family, GNOME Terminal, Tilix and MATE Terminal (`gsettings`, every profile), Ptyxis, GNOME Console, Guake and elementary Terminal (`gsettings`, one font each), Konsole (its profile files), Xfce Terminal, Terminator (every profile), LXTerminal, QTerminal, Deepin Terminal, Sakura, foot, PuTTY (`~/.putty/sessions`), xterm and urxvt (`~/.Xresources`, loaded with `xrdb`), kitty, Alacritty, Tabby, Hyper, Rio, Warp, Wave, Zed | Same as X11 |
| Terminal prompt: put back             | Every change above is recorded in `terminal-undo.txt` with the old value and restored from it; so are the font files the window installed | Same | Same | Same |

Differences on every OS, compared with the C# version:

- **The settings menu is drawn by the program.** Native tray menus close after every click, so
  this one replaces them: it stays open while you change settings, and a click outside or Esc
  closes it. Groups open in place instead of as fly-outs. On Linux the desktop's own right-click
  menu has a **Settings...** entry that opens it.
- **Dialogs are drawn by the program**, in one dark style with a bundled font, rather than
  with native Windows controls. "Forget everything..." asks inside the dashboard.
- **The colour picker is three sliders**, in place of the Windows colour dialog.
- **A start-up setting failure is logged**, not shown as a balloon tip.

## How it is built

The code follows Clean Architecture: one Cargo crate per layer, and each crate may depend only
on the layers inside it. Cargo enforces this, so a dependency that points outward does not compile.

```
crate         depends on
domain        nothing
rendering     domain
application   domain
platform      domain, rendering, application   (implements their ports)
ui            domain, rendering, application   (never platform)
src/main.rs   all of them: the composition root
```

```
crates/
├── domain/        entities: the cast, rt.rs (the runtime under the translated code: reference
│                  semantics, .NET numerics and Random) and settings.rs. No dependencies at all.
│                  Its ports: rt::IWorldApi (the monitors) and rt::LocalClock (local time).
├── rendering/     presenter: the software painters and the paint threads. Depends on domain.
│                  Its port: SpriteSink, the window a finished frame is shown in.
├── application/   use cases: the per-frame session (idle, hot corners, keep-awake, full-screen,
│                  brain saves, menu choices), preferences, brains, update checks and the
│                  terminal prompt (posh.rs: Oh My Posh, its themes, the shell profiles and the
│                  terminal fonts; jsonc.rs edits VS Code's and Windows Terminal's settings
│                  without disturbing their comments). Depends on domain. Its ports (ports.rs):
│                  Desktop, Displays, PreferenceStore, BrainStore, ReleaseFeed, Journal,
│                  Workstation.
├── platform/      adapters: one OS backend each in windows.rs, macos.rs and linux.rs (the same
│                  functions in each), the port implementations (adapters.rs), and settings.txt
│                  and brains.txt (files.rs). Depends on domain, rendering and application.
└── ui/            presentation: the winit loop, frame pump, sprite views, tray menu, cards and
                   dialogs. Depends on domain, rendering and application, but never on platform.
src/main.rs        the composition root: picks the adapters, plugs them into the ports, runs the UI
tools/
├── cs2rs/         the translator used for the migration
├── compare/       checks this port against the C# original
└── hash.rs, sim_trace.rs
```

**Adding an OS capability.** Declare it on the port that needs it (usually `Desktop` in
`crates/application/src/ports.rs`), add the function to all three backends, and forward to it in
`crates/platform/src/adapters.rs`. The UI and the use cases only ever call the port.

**The terminal prompt.** `posh.rs` decides everything (which installer, which profile file, what
line goes in it) and the `Workstation` port only runs programs and reads and writes files, so the
whole plan for every OS is tested in memory on any OS. The port reports the OS; the use case never
asks the compiler. A profile gets one block between `# >>> IdleWalker: Oh My Posh >>>` and
`# <<< IdleWalker: Oh My Posh <<<` (Lua comments for Clink), which is what "Remove" takes out
again. Each terminal has an "apply" that records every setting's old value in
`terminal-undo.txt` (one line per setting: target, key, old value) before changing it, and a
"restore" that reads those lines back; a new terminal needs both. A terminal whose font is one
or two keys in a settings file of a common shape (INI, `Key=Value`, TOML or JSON) needs only a
`Recipe` entry and a line that finds its file; one whose font is a gsettings key needs only a
`GsRecipe`. `IDLEWALKER_TEST_MENU=posh` opens the window at start-up.

**Translated code.** `crates/domain/` (except `lib.rs`), `crates/rendering/` (except `lib.rs`,
`render_thread.rs` and `sprite_sink.rs`) and `settings.rs` were
produced from the C# by `tools/cs2rs/cs2rs.py`, which keeps statement order so floating-point
results stay identical. These files are the source now: edit them directly. The translator is
kept as a record of how they were made. Rerunning it overwrites them, and its output uses the old
single-crate layout.

**The object model.** The cast is an object graph, with walkers holding each other and calling
each other re-entrantly. So a C# class became a `Copy` handle to heap data (see the top of
`crates/domain/src/rt.rs`), and methods take the handle by value. This uses `unsafe` internally and is not
idiomatic Rust. It is safe in practice because the cast is only touched from the UI thread, and
each painter from its own thread, but the compiler does not enforce that. Rewriting the
simulation around indices would be the next step if it needs to become idiomatic.

**Test switches.** `IDLEWALKER_TEST=1` leaves the sign-in entry and the legacy registry alone,
skips the single-instance check, and on Windows keeps sprites capturable.
`IDLEWALKER_TEST_MENU=about,brains` fires menu items at start-up.

## Checking the port

`tools/compare/compare.sh /path/to/the/C#/checkout` (Windows, Git Bash) builds both versions and
compares them. This port's cast is smaller than the C# original's, so **the comparison only passes
against a checkout with the same cast, and fails against the unmodified original.**

- **Painters:** an FNV hash of the painted canvas for 630 poses across both figures, which
  must be byte-identical.
- **Behaviour:** a seeded 10-minute simulation of all 20 walkers, printing every walker's activity
  and position every 10 seconds, then what each has learned. Those must match. The line's running
  hash of raw doubles may differ in its last bits, because `exp` and `pow` come from a different
  maths library; that noise has not been seen to change a decision.

## Releasing

`.github/workflows/build.yml` builds and self-tests on Windows, Linux and macOS (Apple Silicon and
Intel) on every pull request and every `v*` tag; a tag then publishes a GitHub release. Pushing to
another branch builds nothing. You never edit the version by hand: the tag is the version.

```sh
git checkout main && git pull
git push                  # what you release must already be on the default branch
git tag v1.2.3
git push origin v1.2.3
```

| Tag                 | Published as | Accepted                          |
| ------------------- | ------------ | --------------------------------- |
| `v1.2.3`          | release      | yes                               |
| `v1.2.3-beta.1`   | pre-release  | yes                               |
| `1.2.3`, `v1.2` | -            | no: the run fails before building |

A release has `idlewalker-<version>-windows-x64.zip`, `-linux-x64.zip`, `-macos.zip` (a universal
app), and a `SHA256SUMS` file. Every zip holds the app, `README.md`, `LICENSE`,
`THIRD_PARTY_NOTICES.md` and the `LICENSES/` folder. The run fails, and nothing is published, if
the tag is malformed, the tagged commit is not on the default branch, a self-test fails on any OS,
a binary does not report the tagged version (`idlewalker --version`), the Linux binary does not
run cleanly on glibc 2.28, or the packaged macOS app is not universal and validly signed.

To release without pushing a tag, or to rebuild one, use **Run workflow** on the Actions tab and
enter the tag; it is created on the selected commit if it does not exist.

To fix a bad release, delete it on GitHub and delete the tag (`git tag -d v1.2.3` and
`git push origin :refs/tags/v1.2.3`), or simply release the next patch version.

Binaries are unsigned: macOS Gatekeeper and Windows SmartScreen will warn until they are signed.

