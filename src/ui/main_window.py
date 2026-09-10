"""
Main window UI for BigScreen Launcher
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gdk, GLib
from datetime import datetime
from typing import List

from config import Config
from flatpak_scanner import FlatpakScanner, FlatpakApp
from input_handler import InputHandler


class MainWindow(Gtk.ApplicationWindow):
    """Main application window"""
    
    def __init__(self, app: Gtk.Application, config: Config, scanner: FlatpakScanner):
        super().__init__(application=app)
        self.config = config
        self.scanner = scanner
        self.input_handler = InputHandler()
        
        # Setup window properties
        self.set_title("BigScreen Launcher")
        self.set_default_size(1920, 1080)
        self.set_fullscreen(True)
        
        # Build UI
        self.setup_ui()
        self.setup_input_handlers()
        self.refresh_apps()
        
        # Start clock update
        GLib.timeout_add_seconds(1, self.update_clock)
    
    def setup_ui(self):
        """Setup the main UI layout"""
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        main_box.set_margin_start(20)
        main_box.set_margin_end(20)
        main_box.set_margin_top(20)
        main_box.set_margin_bottom(20)
        
        # Header with clock
        header_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        header_box.set_hexpand(True)
        
        title_label = Gtk.Label(label="BigScreen Launcher")
        title_label.set_markup("<span size='x-large' weight='bold'>BigScreen Launcher</span>")
        header_box.append(title_label)
        
        self.clock_label = Gtk.Label()
        self.clock_label.set_markup("<span size='large'></span>")
        header_box.set_end_widget(self.clock_label)
        
        main_box.append(header_box)
        
        # Scrollable grid for apps
        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.NEVER)
        scroll.set_vexpand(True)
        
        self.grid_view = Gtk.FlowBox()
        self.grid_view.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.grid_view.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self.grid_view.set_homogeneous(True)
        self.grid_view.set_max_children_per_line(self.config.get('grid_columns', 4))
        self.grid_view.set_activate_on_single_click(False)
        
        scroll.set_child(self.grid_view)
        main_box.append(scroll)
        
        self.set_child(main_box)
    
    def setup_input_handlers(self):
        """Setup keyboard and gamepad input handlers"""
        self.input_handler.on_left = self.navigate_left
        self.input_handler.on_right = self.navigate_right
        self.input_handler.on_activate = self.activate_app
        self.input_handler.on_back = self.on_back
        self.input_handler.on_settings = self.show_settings
        
        # Add key event controller
        key_controller = Gtk.EventControllerKey.new()
        key_controller.connect('key-pressed', self.on_key_pressed)
        self.add_controller(key_controller)
    
    def on_key_pressed(self, controller: Gtk.EventControllerKey, keyval: int, 
                       keycode: int, state: Gdk.ModifierType) -> bool:
        """Handle key press events"""
        return self.input_handler.handle_key_press(controller, keyval, keycode, state, self)
    
    def refresh_apps(self):
        """Refresh the app grid with current apps"""
        # Clear existing children
        child = self.grid_view.get_first_child()
        while child:
            next_child = child.get_next_sibling()
            self.grid_view.remove(child)
            child = next_child
        
        # Add app buttons
        apps = self.scanner.get_apps()
        for app in apps:
            button = self.create_app_button(app)
            self.grid_view.append(button)
        
        # Select first app
        if self.grid_view.get_first_child():
            self.grid_view.select_child(self.grid_view.get_first_child())
    
    def create_app_button(self, app: FlatpakApp) -> Gtk.Button:
        """Create a button for an app"""
        button = Gtk.Button()
        button.set_size_request(200, 200)
        
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.set_margin_start(10)
        box.set_margin_end(10)
        box.set_margin_top(10)
        box.set_margin_bottom(10)
        
        # App icon
        if app.icon:
            try:
                icon = Gtk.Image.new_from_icon_name(app.icon)
                icon.set_pixel_size(128)
                box.append(icon)
            except:
                icon_label = Gtk.Label(label="📦")
                icon_label.set_markup("<span size='50000'>📦</span>")
                box.append(icon_label)
        else:
            icon_label = Gtk.Label(label="📦")
            icon_label.set_markup("<span size='50000'>📦</span>")
            box.append(icon_label)
        
        # App name
        name_label = Gtk.Label(label=app.name)
        name_label.set_wrap(True)
        name_label.set_justify(Gtk.Justification.CENTER)
        box.append(name_label)
        
        button.set_child(box)
        button.set_data("app_id", app.app_id)
        button.connect('clicked', self.on_app_clicked)
        
        return button
    
    def on_app_clicked(self, button: Gtk.Button):
        """Handle app button click"""
        app_id = button.get_data("app_id")
        app = self.scanner.get_app_by_id(app_id)
        if app:
            self.launch_app(app)
    
    def launch_app(self, app: FlatpakApp):
        """Launch a Flatpak application"""
        import subprocess
        try:
            subprocess.Popen(['flatpak', 'run', app.app_id], 
                           stdout=subprocess.DEVNULL, 
                           stderr=subprocess.DEVNULL)
        except Exception as e:
            print(f"Error launching {app.name}: {e}")
    
    def navigate_left(self):
        """Navigate to previous app"""
        selected = self.grid_view.get_selected_children()
        if selected:
            current = selected[0]
            previous = current.get_prev_sibling()
            if previous:
                self.grid_view.select_child(previous)
                previous.grab_focus()
    
    def navigate_right(self):
        """Navigate to next app"""
        selected = self.grid_view.get_selected_children()
        if selected:
            current = selected[0]
            next_child = current.get_next_sibling()
            if next_child:
                self.grid_view.select_child(next_child)
                next_child.grab_focus()
    
    def activate_app(self):
        """Activate/launch the selected app"""
        selected = self.grid_view.get_selected_children()
        if selected:
            selected[0].activate()
    
    def on_back(self):
        """Handle back button (Escape)"""
        # Could minimize or show options
        pass
    
    def show_settings(self):
        """Show settings dialog"""
        # Placeholder for settings
        print("Settings requested")
    
    def update_clock(self) -> bool:
        """Update the clock display"""
        now = datetime.now()
        time_str = now.strftime("%H:%M:%S")
        self.clock_label.set_markup(f"<span size='large'>{time_str}</span>")
        return True
