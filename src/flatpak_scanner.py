"""
Flatpak application scanner for BigScreen Launcher
"""

import os
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass
import configparser


@dataclass
class FlatpakApp:
    """Represents a Flatpak application"""
    app_id: str
    name: str
    icon: str
    exec: str
    desktop_file: str


class FlatpakScanner:
    """Scan and parse Flatpak applications"""
    
    FLATPAK_APPS_DIR = '/var/lib/flatpak/exports/share/applications'
    
    def __init__(self):
        self.apps: List[FlatpakApp] = []
        self.scan()
    
    def scan(self):
        """Scan for installed Flatpak applications"""
        self.apps = []
        
        if not os.path.exists(self.FLATPAK_APPS_DIR):
            print(f"Warning: Flatpak apps directory not found: {self.FLATPAK_APPS_DIR}")
            return
        
        desktop_files = sorted(Path(self.FLATPAK_APPS_DIR).glob('*.desktop'))
        
        for desktop_file in desktop_files:
            app = self._parse_desktop_file(desktop_file)
            if app:
                self.apps.append(app)
        
        # Sort alphabetically by name
        self.apps.sort(key=lambda x: x.name.lower())
    
    def _parse_desktop_file(self, desktop_file: Path) -> Optional[FlatpakApp]:
        """Parse a .desktop file and extract app information"""
        try:
            config = configparser.ConfigParser()
            config.read(desktop_file)
            
            if 'Desktop Entry' not in config:
                return None
            
            entry = config['Desktop Entry']
            
            # Extract required fields
            name = entry.get('Name', '')
            if not name:
                return None
            
            # Extract app ID from filename (remove .desktop)
            app_id = desktop_file.stem
            
            # Get icon
            icon = entry.get('Icon', '')
            
            # Get Exec field - may have multiple entries
            exec_cmd = entry.get('Exec', '')
            if not exec_cmd:
                return None
            
            return FlatpakApp(
                app_id=app_id,
                name=name,
                icon=icon,
                exec=exec_cmd,
                desktop_file=str(desktop_file)
            )
        
        except Exception as e:
            print(f"Warning: Failed to parse {desktop_file}: {e}")
            return None
    
    def get_apps(self) -> List[FlatpakApp]:
        """Get all discovered applications"""
        return self.apps.copy()
    
    def get_app_by_id(self, app_id: str) -> Optional[FlatpakApp]:
        """Get an application by its ID"""
        for app in self.apps:
            if app.app_id == app_id:
                return app
        return None
