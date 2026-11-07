# Changelog

Notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## Unreleased

### Fixed

- Walkers on macOS no longer wobble or stretch while walking: each sprite window keeps
  its size and sits on whole points, and the picture moves inside it in one transaction.
- Smoother motion on macOS: frames follow the display's refresh through a display link
  instead of a 16 ms timer, and battery mode is an exact half of the refresh rate.
- Regular hitches on macOS: the battery check no longer launches `pmset` on the UI
  thread, the windows are no longer re-raised every second, and the display layout is
  read twice a second instead of several times a frame.
- The painter on macOS no longer drops to the efficiency cores on Apple Silicon.
- Painting runs on two threads where there are cores to spare, and the paint budget no
  longer flips the frame rate back and forth when the demand hovers near it.
- On macOS with displays of different scaling, tray panels no longer open in the wrong place
  or close when clicked.
- On macOS, each brightness slider now uses the same display number as the card on that monitor.
- On macOS, the terminal prompt window can now set the Nerd Font in Terminal: the app
  bundle declares that it sends Apple events, so macOS asks once to let IdleWalker control
  Terminal instead of refusing silently. When the permission is refused, the window now says
  where to allow it.
- The terminal prompt's icons and colours now come out right in every terminal profile, not
  only the default one, following the Oh My Posh documentation for each terminal:
  - Windows Terminal: the font goes into every profile that had a font of its own (those
    overrode the default), the older `fontFace` key is replaced, and the colour adjustment
    that breaks Powerline separators is turned off.
  - VS Code: the contrast rule that recolours the prompt's segments is turned off along
    with setting the font.
  - Console windows: every program or window title with its own font under `HKCU\Console`
    is set, not only the default; those kept Consolas before.
  - macOS Terminal, GNOME Terminal and Konsole: every profile, not only the default.
  - macOS Terminal and iTerm2 now get the font's real PostScript name, `MesloLGMNFM-Regular`:
    Nerd Fonts have shortened it since version 3, and Terminal answers "done" to a name it
    does not know while keeping its old font. The name macOS itself resolves the font to
    is used first, the profiles are read back afterwards, the font's other names are tried
    in turn, and the window says which profiles still have another font if none took,
    instead of reporting the font as set. The window could not read Terminal's answer
    before, since `tab` inside a `tell application "Terminal"` is its window-tab class.
    The status line asks the running Terminal too, instead of its settings file, which
    Terminal writes only when it quits.
  - iTerm2: the font for non-ASCII text as well, where a profile has one.
  - Bash: a `.bash_profile` that hid `.bashrc` is made to load it on Windows (Git Bash) and
    Linux too, not only on macOS.
  - Nushell: the window says when Nushell is too old (0.104 is needed), and "Remove" and
    "Uninstall" delete the script `oh-my-posh init nu` leaves in Nushell's autoload folder.

### Added

- Rust port of the C# Windows original, running on Windows, macOS and Linux.
- Settings menu and dialogs drawn by the program.
- Automated builds and releases for all supported platforms.
- MIT license, third-party notices and contributor documentation.
- External monitor brightness on Apple Silicon Macs, through DDC/CI, including monitors that
  accept a new level but will not report their current one.
- Software dimming on macOS for external monitors that DDC/CI cannot reach.
- "Check for updates" in the settings menu, and a check in the background every few hours.
  When a new version is found, a dialog offers the download and explains that the running
  copy must exit first.
- "Terminal prompt..." in the settings menu: a window that installs or updates Oh My Posh,
  downloads its themes, puts the chosen theme into every shell found on the machine
  (PowerShell 7, Windows PowerShell, bash, zsh, fish, Command Prompt through Clink, nu),
  installs the Meslo Nerd Font, sets it as the terminal font in VS Code, Windows Terminal,
  console windows, macOS Terminal, iTerm2, GNOME Terminal and Konsole, and can take the prompt
  out of every shell again or uninstall Oh My Posh
  altogether, through the package manager that installed it. On Windows it also lets
  PowerShell load its profile when the execution policy forbids it.
- The terminal prompt window also knows Tilix, Xfce Terminal, kitty, Alacritty and Tabby,
  Windows Terminal Canary and Code - OSS, and reports WezTerm and Ghostty as needing nothing, since
  they carry the icons themselves. It then grew to cover the rest of the popular terminals:
  on Windows ConEmu and Cmder (every configuration, in `ConEmu.xml` or the registry), PuTTY
  (every saved session), and mintty as Git Bash, MSYS2, Cygwin and wsltty ship it; on Linux
  MATE Terminal (every profile), Ptyxis, GNOME Console, Guake, elementary Terminal, Terminator
  (every profile), LXTerminal, QTerminal, Deepin Terminal, Sakura, foot, PuTTY, and xterm and
  urxvt through `~/.Xresources`; and everywhere Hyper, Rio, Warp, Wave and the Zed editor's
  terminal. Every terminal setting it changes is recorded with the
  value that was there, in `terminal-undo.txt` in IdleWalker's folder; "Put the terminals
  back" restores them all, and "Uninstall" restores them and removes the font files the
  window installed, leaving a font that was already there alone. After each font task it
  writes `terminal-debug.txt` beside it: every program it ran and what that printed, for
  working out why a terminal did not take the font.

### Changed

- Linux downloads are now a single executable that runs on glibc 2.28 and later, instead of a
  tar archive that needed glibc 2.34.
- macOS downloads are now one zip holding a universal `IdleWalker.app`, instead of a tar
  archive per CPU type.
- On macOS, a click on the status bar icon now opens the brightness panel.
- The Angry Birds look and move more like birds: soft feathered shading instead of a plastic
  shine, bounding hops, quick held glances, faster breathing and an occasional fluff-and-shake.

### Removed

- Mr. Bean and the Pink Panther, along with their menu entries and the antics only they used.
  Saved memories of the other walkers are kept.
