"""
Input handling for keyboard and gamepad/controller support
"""

import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gdk
from typing import Callable, Optional


class InputHandler:
    """Handle keyboard and gamepad input"""
    
    def __init__(self):
        self.on_left: Optional[Callable] = None
        self.on_right: Optional[Callable] = None
        self.on_activate: Optional[Callable] = None
        self.on_back: Optional[Callable] = None
        self.on_settings: Optional[Callable] = None
    
    def handle_key_press(self, controller: Gtk.EventControllerKey, keyval: int, 
                        keycode: int, state: Gdk.ModifierType, widget: Gtk.Widget) -> bool:
        """Handle keyboard input"""
        
        # Arrow keys for navigation
        if keyval == Gdk.KEY_Left:
            if self.on_left:
                self.on_left()
            return True
        elif keyval == Gdk.KEY_Right:
            if self.on_right:
                self.on_right()
            return True
        
        # Enter to activate
        elif keyval == Gdk.KEY_Return:
            if self.on_activate:
                self.on_activate()
            return True
        
        # Escape to go back
        elif keyval == Gdk.KEY_Escape:
            if self.on_back:
                self.on_back()
            return True
        
        # Super key to show settings (or bring to foreground)
        elif keyval == Gdk.KEY_Super_L or keyval == Gdk.KEY_Super_R:
            if self.on_settings:
                self.on_settings()
            return True
        
        return False
    
    def handle_gamepad(self, gamepad, event_type: str, button: int, 
                      axis: int, value: float) -> bool:
        """
        Handle gamepad/controller input
        
        Note: GTK4 does not have native gamepad support.
        This is a stub for future implementation using external libraries
        (e.g., python-evdev, pygame, or libinput).
        """
        # TODO: Implement gamepad support using external library
        # For now, keyboard input is the primary control method
        return False
