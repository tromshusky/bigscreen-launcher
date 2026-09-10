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
    
    def handle_key_press(self, controller: Gtk.EventControllerKey, keyval: int, keycode: int, 
                        state: Gdk.ModifierType, widget: Gtk.Widget) -> bool:
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
    
    def handle_gamepad(self, gamepad: Gtk.Gamepad, event_type: str, button: int, 
                      axis: int, value: float) -> bool:
        """Handle gamepad/controller input"""
        
        # D-Pad left navigation
        if event_type == 'button_press':
            # A Button (cross on PS, A on Xbox) - Activate
            if button == 0:  # A/Cross
                if self.on_activate:
                    self.on_activate()
                return True
            
            # B Button (circle on PS, B on Xbox) - Back
            elif button == 1:  # B/Circle
                if self.on_back:
                    self.on_back()
                return True
            
            # Home button to show settings
            elif button == 10:  # Home/Start
                if self.on_settings:
                    self.on_settings()
                return True
        
        # D-Pad and analog stick navigation
        elif event_type == 'axis_motion':
            # Left stick X axis (0) or D-Pad X (6)
            if axis in [0, 6]:
                if value < -0.5:  # Left
                    if self.on_left:
                        self.on_left()
                    return True
                elif value > 0.5:  # Right
                    if self.on_right:
                        self.on_right()
                    return True
        
        return False
