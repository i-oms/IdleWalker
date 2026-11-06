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
