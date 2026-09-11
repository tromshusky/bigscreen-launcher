use gtk4::gdk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Left,
    Right,
    Up,
    Down,
    Activate,
    Back,
    Settings,
}

pub fn map_keyval(keyval: gdk::Key) -> Option<InputAction> {
    match keyval {
        gdk::Key::Left => Some(InputAction::Left),
        gdk::Key::Right => Some(InputAction::Right),
        gdk::Key::Up => Some(InputAction::Up),
        gdk::Key::Down => Some(InputAction::Down),
        gdk::Key::Return | gdk::Key::KP_Enter => Some(InputAction::Activate),
        gdk::Key::Escape => Some(InputAction::Back),
        gdk::Key::Super_L | gdk::Key::Super_R => Some(InputAction::Settings),
        _ => None,
    }
}