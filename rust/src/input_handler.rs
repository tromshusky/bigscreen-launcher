use gtk4::gdk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Left,
    Right,
    Activate,
    Back,
    Settings,
}

pub fn map_keyval(keyval: gdk::Key) -> Option<InputAction> {
    match keyval {
        gdk::Key::Left => Some(InputAction::Left),
        gdk::Key::Right => Some(InputAction::Right),
        gdk::Key::Return | gdk::Key::KP_Enter => Some(InputAction::Activate),
        gdk::Key::Escape => Some(InputAction::Back),
        gdk::Key::Super_L | gdk::Key::Super_R => Some(InputAction::Settings),
        _ => None,
    }
}

// GTK4 has no native gamepad API, same as the Python version's stub in
// input_handler.py. Real controller support (D-Pad, A/B buttons) would
// mean polling a crate like `gilrs` on a glib timeout and translating
// button/axis events into the InputAction values above.