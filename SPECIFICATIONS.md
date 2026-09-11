# BigScreen Launcher - Application Specifications

## Overview
A GTK4 application designed for HTPC (Home Theater PC) environments that displays installed Flatpak applications in a horizontal scrollable grid with full controller and keyboard support.

## Implementations

This repo ships two implementations of the same specification, both built
by the same Flatpak manifest:

- **Python / PyGObject** (default, `src/`) — the original reference
  implementation.
- **Rust / gtk4-rs** (`rust/`) — a from-scratch implementation of the same
  UI and behavior. Launch it with:

```bash
  flatpak run com.github.tromshusky.bigscreenLauncher -- --rust
```

  The installed `bigscreen-launcher` command is a thin dispatcher: it
  detects `--rust` and `exec`s `/app/bin/bigscreen-launcher-rs` instead of
  starting the Python app.

Both implementations read the same `~/.config/bigscreen-launcher/settings.json`
config format and scan the same Flatpak `.desktop` directories, so switching
between them doesn't lose settings.

### Known gaps (both implementations)
- Gamepad/controller input is stubbed — only keyboard input (arrows,
  Enter, Escape, Super) is wired up. Controller support needs an
  external crate/library (`python-evdev`/`pygame` for Python, `gilrs`
  for Rust) polled on a timer.
- The Settings menu is a placeholder; grid size, Steam launch mode, and
  scrolling mode are read from config but not yet editable in-app.
- Multi-row grids (`grid_rows` > 1) are accepted in config but not yet
  rendered — both UIs currently render a single scrolling row.
## Core Features

### Application Display
- **Source**: Scans `/var/lib/flatpak/exports/share/applications/*.desktop` for installed Flatpaks
- **Layout**: Horizontal scrolling grid
- **Default Display**: 1 row × 4 columns (4 items visible at once)
- **Sorting**: Alphabetically by application name
- **Item Format**: Icon above application name
- **Selection Indicator**: Rounded corner frame around selected item

### Navigation & Controls
#### Keyboard
- **Arrow Keys**: Navigate left/right through items
- **Enter**: Launch selected application
- **Esc**: Close application or exit settings

#### Controller Support
- **D-Pad/Analog Sticks**: Navigate left/right through items
- **A Button** (PlayStation Cross, Xbox A): Launch selected application
- **B Button** (PlayStation Circle, Xbox B): Close application or exit settings
- **Standard Gamepad Mappings**: Full support for common game controllers

#### Input Method
- Fully functional with keyboard OR controller only
- No mouse/touchscreen required

### User Interface

#### Top Left
- Digital clock (HH:MM format)
- Real-time updates

#### Top Right
- Settings wheel icon (clickable)

#### Main Content Area
- Vertically centered horizontal scrolling list
- Visible items constrained to display bounds
- Smooth continuous or page-based scrolling

#### Selected Item Visual
- Rounded corner frame/border around focused item
- Clear visual distinction from other items

### Settings Menu

#### Options
1. **Steam Launch Mode**
   - BigPicture Mode
   - PC Mode
   - Default: BigPicture Mode

2. **Grid Display**
   - Rows: Configurable (default: 1)
   - Columns: Configurable (default: 4)

3. **Scrolling Mode**
   - Continuous: Smooth scrolling to any item
   - Pages: Scroll by page (full viewport width)
   - Default: Continuous

#### Settings Persistence
- Settings saved to user configuration directory
- Loaded on application startup

### Application Launching
- Click or press Enter on selected app
- Launch using `flatpak run <app-id>`
- Proper handling of application lifecycle
- Return focus to launcher after app closes (if applicable)
- Capture Super key (keyboard) or Home (controller) to bring launcher to the foreground within launched applications

### Technical Requirements

#### Framework
- GTK 4.0 or higher
- Python 3.8+ with PyGObject (`src/`), **or**
- Rust 1.75+ with gtk4-rs 0.9 (`rust/`)

#### Desktop File Parsing
- Parse `.desktop` files from `/var/lib/flatpak/exports/share/applications/`
- Extract: Name, Icon, Exec fields (multiple if applicable)

#### Distribution
- Packaged as Flatpak
- Org ID: `com.github.tromshusky.bigscreenLauncher`
- Permissions: File system access to Flatpak directories, D-Bus for launching apps
- Fallback to single letter icons in different colors 

## User Experience Goals
- Simple, distraction-free interface optimized for distance viewing
- Fast, responsive navigation
- Minimal text, maximum icons
- Game controller as primary input method
- TV/projector friendly design
