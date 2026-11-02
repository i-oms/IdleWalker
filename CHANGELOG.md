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

### Changed

- Linux downloads are now a single executable that runs on glibc 2.28 and later, instead of a
  tar archive that needed glibc 2.34.
- macOS downloads are now one disk image with a universal `IdleWalker.app`, instead of a tar
  archive per CPU type.
- On macOS, a click on the status bar icon now opens the brightness panel.

### Fixed

- On macOS with displays of different scaling, tray panels no longer open in the wrong place
  or close when clicked.
- On macOS, each brightness slider now uses the same display number as the card on that monitor.
