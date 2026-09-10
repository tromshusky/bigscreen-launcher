# BigScreen Launcher - Application Specifications

## Overview
A GTK4 application designed for HTPC (Home Theater PC) environments that displays installed Flatpak applications in a horizontal scrollable grid with full controller and keyboard support.

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

### Technical Requirements

#### Framework
- GTK 4.0 or higher
- Python 3.8+ with PyGObject

#### Desktop File Parsing
- Parse `.desktop` files from `/var/lib/flatpak/exports/share/applications/`
- Extract: Name, Icon, Exec fields
- Support for both local icons and icon theme lookups

#### Distribution
- Packaged as Flatpak
- Org ID: `com.github.tromshusky.bigscreenLauncher`
- Permissions: File system access to Flatpak directories, D-Bus for launching apps
- Fallback to bundled or system icons

## User Experience Goals
- Simple, distraction-free interface optimized for distance viewing
- Fast, responsive navigation
- Minimal text, maximum icons
- Game controller as primary input method
- TV/projector friendly design
