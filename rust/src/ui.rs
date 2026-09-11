use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4::{
    Align, ApplicationWindow, Box as GtkBox, Button, CssProvider, Image, Justification, Label,
    Orientation, PolicyType, ScrolledWindow,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config::Config;
use crate::flatpak_scanner::{FlatpakApp, FlatpakScanner};
use crate::input_handler::{map_keyval, InputAction};

const SELECTED_CSS: &str = "
button.bigscreen-item { border-radius: 16px; }
button.bigscreen-item.selected {
    border: 4px solid @accent_bg_color;
    border-radius: 16px;
}
";

pub struct MainWindow {
    pub window: ApplicationWindow,
    items_box: GtkBox,
    clock_label: Label,
    config: Config,
    scanner: RefCell<FlatpakScanner>,
    apps: RefCell<Vec<FlatpakApp>>,
    buttons: RefCell<Vec<Button>>,
    selected: Cell<usize>,
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
        provider.load_from_data(SELECTED_CSS); // older gtk4-rs: provider.load_from_data(SELECTED_CSS.as_bytes())
        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let main_box = GtkBox::new(Orientation::Vertical, 20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);

        // Header: title, clock, settings
        let header_box = GtkBox::new(Orientation::Horizontal, 20);
        header_box.set_hexpand(true);

        let title_label = Label::new(None);
        title_label.set_markup("<span size='x-large' weight='bold'>BigScreen Launcher</span>");
        title_label.set_halign(Align::Start);
        header_box.append(&title_label);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header_box.append(&spacer);

        let clock_label = Label::new(None);
        clock_label.set_halign(Align::End);
        header_box.append(&clock_label);

        let settings_button = Button::from_icon_name("emblem-system-symbolic");
        settings_button.set_tooltip_text(Some("Settings"));
        header_box.append(&settings_button);

        main_box.append(&header_box);

        // Horizontally scrolling row of apps — a real horizontal strip,
        // not a wrapping grid, matching "1 row x 4 columns, scroll for more".
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
            config,
            scanner: RefCell::new(scanner),
            apps: RefCell::new(Vec::new()),
            buttons: RefCell::new(Vec::new()),
            selected: Cell::new(0),
        });

        win.setup_input_handlers();
        win.refresh_apps();
        win.update_clock();

        {
            let win_clone = win.clone();
            glib::timeout_add_seconds_local(1, move || {
                win_clone.update_clock();
                glib::ControlFlow::Continue
            });
        }

        {
            let win_clone = win.clone();
            settings_button.connect_clicked(move |_| win_clone.show_settings());
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

        if !app.icon.is_empty() {
            let icon = Image::from_icon_name(&app.icon);
            icon.set_pixel_size(128);
            box_.append(&icon);
        } else {
            let icon_label = Label::new(None);
            icon_label.set_markup("<span size='50000'>\u{1F4E6}</span>");
            box_.append(&icon_label);
        }

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

    fn launch(self: &Rc<Self>, app_id: &str) {
        if let Some(app) = self.scanner.borrow().get_app_by_id(app_id) {
            match std::process::Command::new("flatpak")
                .args(["run", &app.app_id])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => {}
                Err(e) => eprintln!("Error launching {}: {e}", app.name),
            }
        }
    }

    fn on_back(self: &Rc<Self>) {
        // Reserved for closing an overlay / exiting settings.
    }

    fn show_settings(self: &Rc<Self>) {
        // Placeholder settings dialog (steam_launch_mode, grid_columns,
        // scrolling_mode) — same stub status as the Python version.
        eprintln!("Settings requested (grid_columns={})", self.config.grid_columns);
    }

    fn update_clock(self: &Rc<Self>) {
        if let Ok(now) = glib::DateTime::now_local() {
            let time_str = now.format("%H:%M:%S").unwrap_or_default();
            self.clock_label
                .set_markup(&format!("<span size='large'>{}</span>", time_str));
        }
    }
}