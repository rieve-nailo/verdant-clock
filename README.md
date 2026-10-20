# Verdant Clock

A customizable analog desktop clock written in Rust. It starts with a dark green
palette and lets you adjust the clock face, hands, hour markers, numerals, and
second hand independently. Pick a local image as the clock-face background;
it scales proportionally to fill the clock area, cropping edges if needed.
Adjust the face transparency to reveal it, or keep the window above other
windows. Your color, transparency, always-on-top, and background-image path
settings are saved automatically when the app closes and restored the next
time it starts. Selected images are copied into the app's local data directory,
so the original file can be moved or deleted without losing the clock
background.

The settings panel auto-hides when the pointer leaves it and reappears when the
pointer moves to the left edge. Separate opacity controls adjust the clock
face, its elements, and the window background (0% is transparent; 100% is
opaque). Auto-hide can be switched off from the settings panel; the panel also
stays open while a popup such as a color picker is active. A separate setting
can hide the native title bar. Drag the clock surface to move the window, with
or without the title bar. On Windows, hidden-title-bar mode maps pointer input
from the native cursor position to the window's content area to avoid pointer
offsets.

## Run

Install the stable Rust toolchain, then run:

```sh
cargo run --release
```

The app targets Windows, macOS, and Debian-based Linux. On Debian, install the
native windowing and graphics development packages required by eframe before
building (including the X11/Wayland and OpenGL development libraries for your
desktop environment). The native file picker uses the platform's standard
dialog.

## Build

```sh
cargo build --release
```

## Create installers

The packaging workflow builds native install files for all three platforms:

- Windows: `Verdant-Clock-Setup.exe` installer
- macOS: `.dmg` disk image containing the app bundle
- Debian Linux: `.deb` package

Run **Package installers** from the repository's Actions tab to build and
download the artifacts. Pushing a version tag such as `v0.1.0` also builds all
three installers and attaches them to the GitHub Release. Debian builds require
the native development packages listed in `.github/workflows/package.yml`.
