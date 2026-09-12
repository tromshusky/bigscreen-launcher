use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4::{
    Adjustment, Align, ApplicationWindow, Box as GtkBox, Button, ComboBoxText, CssProvider,
    Image, Justification, Label, Orientation, PolicyType, ScrolledWindow, SpinButton, Stack,
    StackTransitionType,
};
use std::cell::{Cell, RefCell};
use std::process::{Command, Stdio};
use std::rc::Rc;

use crate::config::Config;
use crate::flatpak_scanner::{FlatpakApp, FlatpakScanner};
use crate::input_handler::{map_keyval, InputAction};

const STYLE_CSS: &str = "
window {
    background-color: #1e1f29;
}
label {
    color: #e6e6ec;
}

/* Header */
.clock-time {
    color: #f2f2f6;
    font-size: 120px;
    font-weight: 800;
    line-height: 1;
}
.clock-date {
    color: #9a9ab0;
    font-size: 40px;
    font-weight: 400;
}

/* App tiles and settings tile share the same interaction visuals */
button.bigscreen-item {
    background-color: transparent;
    border: 4px solid transparent;
    border-radius: 20px;
    padding: 8px;
    transition: border-color 120ms ease;
}
button.bigscreen-item.selected {
    border-color: #7aa2f7;
}
.app-name {
    color: #e6e6ec;
    font-size: 36px;
    margin-top: 12px;
}

/* Thin light backdrop card — ONLY used behind real (often-transparent)
   icons for contrast. Letter avatars do NOT use this; they supply their
   own single colored box instead, to avoid a double-box look. */
.icon-card {
/*    background-color: #f2f2f2; */
    border-radius: 16px;
    padding: 12px;
    min-width: 140px;
    min-height: 140px;
}
.icon-card.settings-card {
    background-color: #33344a;
    min-width: 0;
    min-height: 0;
}
.icon-card.settings-card image {
    color: #e6e6ec;
}

/* Letter-avatar fallback: single self-contained colored box */
.avatar-box {
    border-radius: 16px;
    min-width: 140px;
    min-height: 140px;
}
.avatar-label {
    color: #ffffff;
    font-size: 56px;
    font-weight: bold;
}
.avatar-color-0 { background-color: #e07a5f; }
.avatar-color-1 { background-color: #81b29a; }
.avatar-color-2 { background-color: #d9a441; }
.avatar-color-3 { background-color: #3d5a80; }
.avatar-color-4 { background-color: #9d6b8f; }
.avatar-color-5 { background-color: #457b9d; }
.avatar-color-6 { background-color: #6d6875; }
.avatar-color-7 { background-color: #ef8354; }

/* Kodi-style full-window settings page */
.settings-page {
    background-color: #1e1f29;
}
.settings-page label {
    color: #e6e6ec;
    font-size: 22px;
}
.settings-heading {
    font-size: 32px;
    font-weight: 800;
    margin-bottom: 20px;
}
.settings-page button {
    font-size: 20px;
    padding: 10px 18px;
}
";

const AVATAR_COLORS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    Grid,
    SettingsTile,
}

pub struct MainWindow {
    pub window: ApplicationWindow,
    stack: Stack,
    items_box: GtkBox,
    settings_button: Button,
    clock_time_label: Label,
    clock_date_label: Label,
    config: RefCell<Config>,
    scanner: RefCell<FlatpakScanner>,
    apps: RefCell<Vec<FlatpakApp>>,
    buttons: RefCell<Vec<Button>>,
    selected: Cell<usize>,
    focus: Cell<FocusTarget>,
}

impl MainWindow {
    pub fn new(app: &gtk4::Application, config: Config, scanner: FlatpakScanner) -> Rc<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BigScreen Launcher")
            .default_width(1920)
            .default_height(1080)
            .build();

        let provider = CssProvider::new();
        provider.load_from_data(STYLE_CSS);
        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            let icon_theme = gtk4::IconTheme::for_display(&display);
            for path in FlatpakScanner::icon_search_paths() {
                if path.exists() {
                    icon_theme.add_search_path(&path);
                }
            }
        }

        // --- Main page ---
        let main_page = GtkBox::new(Orientation::Vertical, 24);
        main_page.set_margin_start(32);
        main_page.set_margin_end(32);
        main_page.set_margin_top(24);
        main_page.set_margin_bottom(32);

        // Header: big clock (time + date subtitle) top-left, large settings tile top-right
        let header_box = GtkBox::new(Orientation::Horizontal, 20);
        header_box.set_hexpand(true);

        let clock_box = GtkBox::new(Orientation::Vertical, 2);
        clock_box.set_valign(Align::Start);
        let clock_time_label = Label::new(None);
        clock_time_label.add_css_class("clock-time");
        clock_time_label.set_halign(Align::Start);
        let clock_date_label = Label::new(None);
        clock_date_label.add_css_class("clock-date");
        clock_date_label.set_halign(Align::Start);
        clock_box.append(&clock_time_label);
        clock_box.append(&clock_date_label);
        header_box.append(&clock_box);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_box.append(&spacer);

        let settings_card = GtkBox::new(Orientation::Vertical, 0);
        settings_card.add_css_class("icon-card");
        settings_card.add_css_class("settings-card");
        settings_card.set_halign(Align::Center);
        settings_card.set_valign(Align::Center);
        settings_card.set_hexpand(true);
        settings_card.set_vexpand(true);

        let settings_icon = Image::from_icon_name("emblem-system-symbolic");
        settings_icon.set_pixel_size(32);
        settings_card.append(&settings_icon);

        let settings_button = Self::build_tile_button(&settings_card.upcast::<gtk4::Widget>(), None);
        header_box.append(&settings_button);

        main_page.append(&header_box);

        // Horizontally scrolling strip of apps
        let items_box = GtkBox::new(Orientation::Horizontal, 20);
        items_box.set_valign(Align::Center);

        let scroller = ScrolledWindow::new();
        scroller.set_policy(PolicyType::Automatic, PolicyType::Never);
        scroller.set_vexpand(true);
        scroller.set_child(Some(&items_box));

        main_page.append(&scroller);

        // --- Stack: main page <-> full-window settings page ---
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::SlideUpDown);
        stack.add_named(&main_page, Some("main"));
        window.set_child(Some(&stack));

        let win = Rc::new(MainWindow {
            window: window.clone(),
            stack: stack.clone(),
            items_box,
            settings_button: settings_button.clone(),
            clock_time_label,
            clock_date_label,
            config: RefCell::new(config),
            scanner: RefCell::new(scanner),
            apps: RefCell::new(Vec::new()),
            buttons: RefCell::new(Vec::new()),
            selected: Cell::new(0),
            focus: Cell::new(FocusTarget::Grid),
        });

        let settings_page = win.build_settings_page();
        stack.add_named(&settings_page, Some("settings"));

        win.setup_input_handlers();
        win.refresh_apps();
        win.update_clock();

        {
            let win_clone = win.clone();
            settings_button.connect_clicked(move |_| win_clone.open_settings());
        }

        {
            let win_clone = win.clone();
            glib::timeout_add_seconds_local(60, move || {
                win_clone.update_clock();
                glib::ControlFlow::Continue
            });
        }
        win.update_clock();

        window.present();
        window.fullscreen();

        // Size app tiles to the real monitor width once the window is up,
        // so exactly `grid_columns` tiles are visible instead of "as many
        // as fit at 200px".
        {
            let win_clone = win.clone();
            glib::idle_add_local_once(move || win_clone.apply_tile_sizing());
        }

        win
    }

    /// Builds a card-style tile button. `icon_widget` is placed as-is —
    /// callers decide whether it needs a light backdrop card (real icons)
    /// or is already a self-contained box (letter avatars, settings glyph).
    fn build_tile_button(icon_widget: &gtk4::Widget, label_text: Option<&str>) -> Button {
        let button = Button::new();
        button.add_css_class("bigscreen-item");

        let outer = GtkBox::new(Orientation::Vertical, 0);
        outer.set_halign(Align::Center);
        outer.set_valign(Align::Center);
        outer.append(icon_widget);

        if let Some(text) = label_text {
            let label = Label::new(Some(text));
            label.add_css_class("app-name");
            label.set_wrap(true);
            label.set_justify(Justification::Center);
            outer.append(&label);
        }

        button.set_child(Some(&outer));
        button
    }

    /// Sizes app tiles so exactly `grid_columns` are visible, and shrinks
    /// the settings tile to ~20% of that size (per UX feedback: it should
    /// read as a small corner control, not a peer-sized tile).
    fn apply_tile_sizing(self: &Rc<Self>) {
        let columns = self.config.borrow().grid_columns.max(1) as f64;
        let monitor_width = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
            .map(|m| m.geometry().width() as f64)
            .unwrap_or(1920.0);

        let outer_margins = 64.0;
        let spacing = 20.0;
        let usable = monitor_width - outer_margins;
        let tile_width = ((usable - spacing * (columns - 1.0)) / columns).max(120.0);

        for button in self.buttons.borrow().iter() {
            button.set_size_request(tile_width as i32, tile_width as i32 + 90);
        }

        let settings_size = (tile_width * 0.2).max(56.0);
        self.settings_button
            .set_size_request(settings_size as i32, settings_size as i32);
    }

    fn setup_input_handlers(self: &Rc<Self>) {
        let key_controller = gtk4::EventControllerKey::new();
        let win = self.clone();
        key_controller.connect_key_pressed(move |_, keyval, _keycode, _state| {
            if let Some(action) = map_keyval(keyval) {
                win.handle_action(action);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        self.window.add_controller(key_controller);
    }

    fn handle_action(self: &Rc<Self>, action: InputAction) {
        // While the settings page is showing, Escape/B returns to main;
        // other navigation is left to the settings widgets themselves.
        if self.stack.visible_child_name().as_deref() == Some("settings") {
            if action == InputAction::Back {
                self.close_settings();
            }
            return;
        }

        match action {
            InputAction::Left => self.navigate(-1),
            InputAction::Right => self.navigate(1),
            InputAction::Up => self.focus_settings_tile(),
            InputAction::Down => self.focus_grid(),
            InputAction::Activate => self.activate_focused(),
            InputAction::Back => self.on_back(),
            InputAction::Settings => self.open_settings(),
        }
    }

    fn refresh_apps(self: &Rc<Self>) {
        while let Some(child) = self.items_box.first_child() {
            self.items_box.remove(&child);
        }
        self.buttons.borrow_mut().clear();

        let apps = self.scanner.borrow().get_apps();
        for app in &apps {
            let button = self.create_app_button(app);
            self.items_box.append(&button);
            self.buttons.borrow_mut().push(button);
        }
        *self.apps.borrow_mut() = apps;

        self.selected.set(0);
        self.focus.set(FocusTarget::Grid);
        self.update_selection_visuals();
    }

    fn create_app_button(self: &Rc<Self>, app: &FlatpakApp) -> Button {
        let icon_widget = self.resolve_icon_widget(app);
        let button = Self::build_tile_button(&icon_widget, Some(&app.name));

        let app_id = app.app_id.clone();
        let win = self.clone();
        button.connect_clicked(move |_| win.launch(&app_id));

        button
    }

    /// Real icon (wrapped in a thin light card for contrast) if the icon
    /// theme can resolve it; a single self-contained letter-avatar box
    /// otherwise. Never both nested together.
    fn resolve_icon_widget(&self, app: &FlatpakApp) -> gtk4::Widget {
        if !app.icon.is_empty() {
            if let Some(display) = gdk::Display::default() {
                let theme = gtk4::IconTheme::for_display(&display);
                if theme.has_icon(&app.icon) {
                    let card = GtkBox::new(Orientation::Vertical, 0);
                    card.add_css_class("icon-card");
                    card.set_halign(Align::Center);
                    card.set_valign(Align::Center);

                    let image = Image::from_icon_name(&app.icon);
                    image.set_pixel_size(96);
                    card.append(&image);

                    return card.upcast();
                }
            }
        }
        Self::letter_avatar(&app.name)
    }

    fn letter_avatar(name: &str) -> gtk4::Widget {
        let letter = name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string());

        let mut hash: u32 = 0;
        for b in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u32);
        }
        let color_index = (hash as usize) % AVATAR_COLORS;

        let avatar = GtkBox::new(Orientation::Vertical, 0);
        avatar.add_css_class("avatar-box");
        avatar.add_css_class(&format!("avatar-color-{color_index}"));
        avatar.set_halign(Align::Center);
        avatar.set_valign(Align::Center);

        let label = Label::new(Some(&letter));
        label.add_css_class("avatar-label");
        avatar.append(&label);

        avatar.upcast()
    }

    fn navigate(self: &Rc<Self>, delta: i32) {
        if self.focus.get() != FocusTarget::Grid {
            return;
        }
        let len = self.buttons.borrow().len();
        if len == 0 {
            return;
        }
        let current = self.selected.get() as i32;
        let next = (current + delta).clamp(0, len as i32 - 1) as usize;
        self.selected.set(next);
        self.update_selection_visuals();
    }

    fn focus_settings_tile(self: &Rc<Self>) {
        self.focus.set(FocusTarget::SettingsTile);
        self.update_selection_visuals();
    }

    fn focus_grid(self: &Rc<Self>) {
        self.focus.set(FocusTarget::Grid);
        self.update_selection_visuals();
    }

    fn update_selection_visuals(self: &Rc<Self>) {
        let buttons = self.buttons.borrow();
        for (i, btn) in buttons.iter().enumerate() {
            let should_select = self.focus.get() == FocusTarget::Grid && i == self.selected.get();
            if should_select {
                btn.add_css_class("selected");
                btn.grab_focus();
            } else {
                btn.remove_css_class("selected");
            }
        }
        if self.focus.get() == FocusTarget::SettingsTile {
            self.settings_button.add_css_class("selected");
            self.settings_button.grab_focus();
        } else {
            self.settings_button.remove_css_class("selected");
        }
    }

    fn activate_focused(self: &Rc<Self>) {
        match self.focus.get() {
            FocusTarget::Grid => {
                let idx = self.selected.get();
                if let Some(app) = self.apps.borrow().get(idx) {
                    self.launch(&app.app_id);
                }
            }
            FocusTarget::SettingsTile => self.open_settings(),
        }
    }

    fn launch(self: &Rc<Self>, app_id: &str) {
        if let Some(app) = self.scanner.borrow().get_app_by_id(app_id) {
            match Command::new("flatpak-spawn")
                .args(["--host", "flatpak", "run", &app.app_id])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(_) => {}
                Err(e) => eprintln!("Error launching {}: {e}", app.name),
            }
        }
    }

    fn on_back(self: &Rc<Self>) {
        // Reserved for future overlay/back-navigation within the grid.
    }

    fn open_settings(self: &Rc<Self>) {
        self.stack.set_visible_child_name("settings");
    }

    fn close_settings(self: &Rc<Self>) {
        self.stack.set_visible_child_name("main");
        self.focus.set(FocusTarget::SettingsTile);
        self.update_selection_visuals();
    }

    /// Kodi-style settings: a full page that replaces the whole window
    /// content, rather than a small popover.
    fn build_settings_page(self: &Rc<Self>) -> GtkBox {
        let page = GtkBox::new(Orientation::Vertical, 20);
        page.add_css_class("settings-page");
        page.set_margin_start(48);
        page.set_margin_end(48);
        page.set_margin_top(40);
        page.set_margin_bottom(40);

        let heading = Label::new(Some("Settings"));
        heading.add_css_class("settings-heading");
        heading.set_halign(Align::Start);
        page.append(&heading);

        let cfg = self.config.borrow().clone();

        let columns_row = GtkBox::new(Orientation::Horizontal, 16);
        columns_row.append(&Label::new(Some("Visible columns")));
        let columns_adjustment = Adjustment::new(cfg.grid_columns as f64, 1.0, 8.0, 1.0, 1.0, 0.0);
        let columns_spin = SpinButton::new(Some(&columns_adjustment), 1.0, 0);
        columns_row.append(&columns_spin);
        page.append(&columns_row);

        let steam_row = GtkBox::new(Orientation::Horizontal, 16);
        steam_row.append(&Label::new(Some("Steam launch mode")));
        let steam_combo = ComboBoxText::new();
        steam_combo.append(Some("bigpicture"), "Big Picture");
        steam_combo.append(Some("pc"), "PC Mode");
        steam_combo.set_active_id(Some(&cfg.steam_launch_mode));
        steam_row.append(&steam_combo);
        page.append(&steam_row);

        let scroll_row = GtkBox::new(Orientation::Horizontal, 16);
        scroll_row.append(&Label::new(Some("Scrolling mode")));
        let scroll_combo = ComboBoxText::new();
        scroll_combo.append(Some("continuous"), "Continuous");
        scroll_combo.append(Some("pages"), "Pages");
        scroll_combo.set_active_id(Some(&cfg.scrolling_mode));
        scroll_row.append(&scroll_combo);
        page.append(&scroll_row);

        let spacer = GtkBox::new(Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        page.append(&spacer);

        let button_row = GtkBox::new(Orientation::Horizontal, 16);
        let save_button = Button::with_label("Save");
        let back_button = Button::with_label("Back");
        button_row.append(&save_button);
        button_row.append(&back_button);
        page.append(&button_row);

        let win = self.clone();
        save_button.connect_clicked(move |_| {
            {
                let mut cfg = win.config.borrow_mut();
                cfg.grid_columns = columns_spin.value() as u32;
                if let Some(id) = steam_combo.active_id() {
                    cfg.steam_launch_mode = id.to_string();
                }
                if let Some(id) = scroll_combo.active_id() {
                    cfg.scrolling_mode = id.to_string();
                }
                cfg.save();
            }
            win.apply_tile_sizing();
            win.close_settings();
        });

        let win_back = self.clone();
        back_button.connect_clicked(move |_| win_back.close_settings());

        page
    }

    fn update_clock(self: &Rc<Self>) {
        if let Ok(now) = glib::DateTime::now_local() {
            let time_str = now.format("%H:%M").unwrap_or_default();
            let date_str = now.format("%a, %d %b %Y").unwrap_or_default();
            self.clock_time_label.set_text(&time_str);
            self.clock_date_label.set_text(&date_str);
        }
    }
}