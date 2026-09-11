use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4::{
    Adjustment, Align, ApplicationWindow, Box as GtkBox, Button, ComboBoxText, CssProvider,
    Image, Justification, Label, Orientation, Popover, PolicyType, ScrolledWindow, SpinButton,
};
use std::cell::{Cell, RefCell};
use std::process::{Command, Stdio};
use std::rc::Rc;

use crate::config::Config;
use crate::flatpak_scanner::{FlatpakApp, FlatpakScanner};
use crate::input_handler::{map_keyval, InputAction};

// Eye-friendly dark palette instead of the GTK default white background.
const STYLE_CSS: &str = "
window {
    background-color: #1e1f29;
}
label {
    color: #e6e6ec;
}
button.bigscreen-item {
    background-color: #2a2b3a;
    border: none;
    border-radius: 16px;
    padding: 0;
    transition: border-color 120ms ease, background-color 120ms ease;
}
button.bigscreen-item:hover {
    background-color: #34364a;
}
button.bigscreen-item.selected {
    border: 4px solid #7aa2f7;
}
.app-avatar {
    border-radius: 20px;
    min-width: 128px;
    min-height: 128px;
}
.app-avatar label {
    color: #ffffff;
    font-size: 42px;
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
.settings-popover-box label {
    color: #1e1f29;
}
";

const AVATAR_COLORS: usize = 8;

pub struct MainWindow {
    pub window: ApplicationWindow,
    items_box: GtkBox,
    clock_label: Label,
    config: RefCell<Config>,
    scanner: RefCell<FlatpakScanner>,
    apps: RefCell<Vec<FlatpakApp>>,
    buttons: RefCell<Vec<Button>>,
    selected: Cell<usize>,
    settings_popover: RefCell<Option<Popover>>,
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

            // Other apps' exported icons aren't in the sandbox's default
            // icon search path — add them explicitly.
            let icon_theme = gtk4::IconTheme::for_display(&display);
            for path in FlatpakScanner::icon_search_paths() {
                if path.exists() {
                    icon_theme.add_search_path(&path);
                }
            }
        }

        let main_box = GtkBox::new(Orientation::Vertical, 20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);

        // Header: clock top-left, settings wheel top-right (per spec —
        // no app title bar in the design).
        let header_box = GtkBox::new(Orientation::Horizontal, 20);
        header_box.set_hexpand(true);

        let clock_label = Label::new(None);
        clock_label.set_halign(Align::Start);
        clock_label.set_markup("<span size='large'></span>");
        header_box.append(&clock_label);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_box.append(&spacer);

        let settings_button = Button::from_icon_name("emblem-system-symbolic");
        settings_button.add_css_class("flat");
        settings_button.set_tooltip_text(Some("Settings"));
        header_box.append(&settings_button);

        main_box.append(&header_box);

        // Horizontally scrolling strip of apps.
        let items_box = GtkBox::new(Orientation::Horizontal, 20);
        items_box.set_valign(Align::Center);

        let scroller = ScrolledWindow::new();
        scroller.set_policy(PolicyType::Automatic, PolicyType::Never);
        scroller.set_vexpand(true);
        scroller.set_child(Some(&items_box));

        main_box.append(&scroller);
        window.set_child(Some(&main_box));

        let win = Rc::new(MainWindow {
            window: window.clone(),
            items_box,
            clock_label,
            config: RefCell::new(config),
            scanner: RefCell::new(scanner),
            apps: RefCell::new(Vec::new()),
            buttons: RefCell::new(Vec::new()),
            selected: Cell::new(0),
            settings_popover: RefCell::new(None),
        });

        win.setup_input_handlers();
        win.refresh_apps();
        win.update_clock();

        let popover = win.build_settings_popover(&settings_button);
        *win.settings_popover.borrow_mut() = Some(popover);

        {
            let win_clone = win.clone();
            settings_button.connect_clicked(move |_| win_clone.show_settings());
        }

        {
            let win_clone = win.clone();
            glib::timeout_add_seconds_local(1, move || {
                win_clone.update_clock();
                glib::ControlFlow::Continue
            });
        }

        window.present();
        window.fullscreen();

        win
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
        match action {
            InputAction::Left => self.navigate(-1),
            InputAction::Right => self.navigate(1),
            InputAction::Activate => self.activate_selected(),
            InputAction::Back => self.on_back(),
            InputAction::Settings => self.show_settings(),
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
        self.update_selection(None);
    }

    fn create_app_button(self: &Rc<Self>, app: &FlatpakApp) -> Button {
        let button = Button::new();
        button.set_size_request(200, 200);
        button.add_css_class("bigscreen-item");

        let box_ = GtkBox::new(Orientation::Vertical, 10);
        box_.set_margin_start(10);
        box_.set_margin_end(10);
        box_.set_margin_top(10);
        box_.set_margin_bottom(10);
        box_.set_valign(Align::Center);
        box_.set_halign(Align::Center);

        box_.append(&self.create_app_icon(app));

        let name_label = Label::new(Some(&app.name));
        name_label.set_wrap(true);
        name_label.set_justify(Justification::Center);
        box_.append(&name_label);

        button.set_child(Some(&box_));

        let app_id = app.app_id.clone();
        let win = self.clone();
        button.connect_clicked(move |_| win.launch(&app_id));

        button
    }

    /// Uses the real icon if the icon theme can resolve it; otherwise
    /// falls back to a colored letter avatar, per spec.
    fn create_app_icon(&self, app: &FlatpakApp) -> gtk4::Widget {
        if !app.icon.is_empty() {
            if let Some(display) = gdk::Display::default() {
                let theme = gtk4::IconTheme::for_display(&display);
                if theme.has_icon(&app.icon) {
                    let image = Image::from_icon_name(&app.icon);
                    image.set_pixel_size(128);
                    return image.upcast();
                }
            }
        }
        Self::create_letter_avatar(&app.name)
    }

    fn create_letter_avatar(name: &str) -> gtk4::Widget {
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
        avatar.set_size_request(128, 128);
        avatar.set_halign(Align::Center);
        avatar.set_valign(Align::Center);
        avatar.add_css_class("app-avatar");
        avatar.add_css_class(&format!("avatar-color-{color_index}"));

        let label = Label::new(Some(&letter));
        label.set_halign(Align::Center);
        label.set_valign(Align::Center);
        avatar.append(&label);

        avatar.upcast()
    }

    fn navigate(self: &Rc<Self>, delta: i32) {
        let len = self.buttons.borrow().len();
        if len == 0 {
            return;
        }
        let current = self.selected.get() as i32;
        let next = (current + delta).clamp(0, len as i32 - 1) as usize;
        if next != self.selected.get() {
            let previous = self.selected.get();
            self.selected.set(next);
            self.update_selection(Some(previous));
        }
    }

    fn update_selection(self: &Rc<Self>, previous: Option<usize>) {
        let buttons = self.buttons.borrow();
        if let Some(prev) = previous {
            if let Some(btn) = buttons.get(prev) {
                btn.remove_css_class("selected");
            }
        }
        if let Some(btn) = buttons.get(self.selected.get()) {
            btn.add_css_class("selected");
            btn.grab_focus();
        }
    }

    fn activate_selected(self: &Rc<Self>) {
        let idx = self.selected.get();
        if let Some(app) = self.apps.borrow().get(idx) {
            self.launch(&app.app_id);
        }
    }

    /// Launches via flatpak-spawn --host, since the sandboxed app has
    /// no `flatpak` binary of its own to call directly.
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
        // Reserved for closing an overlay / exiting settings.
        if let Some(p) = self.settings_popover.borrow().as_ref() {
            p.popdown();
        }
    }

    fn show_settings(self: &Rc<Self>) {
        if let Some(p) = self.settings_popover.borrow().as_ref() {
            p.popup();
        }
    }

    /// Builds the (now functional) settings popover: grid columns,
    /// Steam launch mode, and scrolling mode, persisted to config.json.
    fn build_settings_popover(self: &Rc<Self>, parent: &Button) -> Popover {
        let popover = Popover::new();
        popover.set_parent(parent);

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.add_css_class("settings-popover-box");
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let heading = Label::new(None);
        heading.set_markup("<b>Settings</b>");
        heading.set_halign(Align::Start);
        content.append(&heading);

        let cfg = self.config.borrow().clone();

        let columns_row = GtkBox::new(Orientation::Horizontal, 8);
        columns_row.append(&Label::new(Some("Visible columns")));
        let columns_adjustment = Adjustment::new(cfg.grid_columns as f64, 1.0, 8.0, 1.0, 1.0, 0.0);
        let columns_spin = SpinButton::new(Some(&columns_adjustment), 1.0, 0);
        columns_row.append(&columns_spin);
        content.append(&columns_row);

        let steam_row = GtkBox::new(Orientation::Horizontal, 8);
        steam_row.append(&Label::new(Some("Steam launch mode")));
        let steam_combo = ComboBoxText::new();
        steam_combo.append(Some("bigpicture"), "Big Picture");
        steam_combo.append(Some("pc"), "PC Mode");
        steam_combo.set_active_id(Some(&cfg.steam_launch_mode));
        steam_row.append(&steam_combo);
        content.append(&steam_row);

        let scroll_row = GtkBox::new(Orientation::Horizontal, 8);
        scroll_row.append(&Label::new(Some("Scrolling mode")));
        let scroll_combo = ComboBoxText::new();
        scroll_combo.append(Some("continuous"), "Continuous");
        scroll_combo.append(Some("pages"), "Pages");
        scroll_combo.set_active_id(Some(&cfg.scrolling_mode));
        scroll_row.append(&scroll_combo);
        content.append(&scroll_row);

        let save_button = Button::with_label("Save");
        content.append(&save_button);

        popover.set_child(Some(&content));

        let win = self.clone();
        let popover_for_save = popover.clone();
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
            popover_for_save.popdown();
        });

        popover
    }

    fn update_clock(self: &Rc<Self>) {
        if let Ok(now) = glib::DateTime::now_local() {
            let time_str = now.format("%H:%M").unwrap_or_default();
            self.clock_label
                .set_markup(&format!("<span size='large'>{}</span>", time_str));
        }
    }
}