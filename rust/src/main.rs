mod config;
mod flatpak_scanner;
mod input_handler;
mod ui;

use gtk4::glib;
use gtk4::prelude::*;

use config::Config;
use flatpak_scanner::FlatpakScanner;
use ui::MainWindow;

const APP_ID: &str = "com.github.tromshusky.bigscreenLauncher";

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let config = Config::load();
        let scanner = FlatpakScanner::new();
        let _window = MainWindow::new(app, config, scanner);
    });

    app.run()
}
