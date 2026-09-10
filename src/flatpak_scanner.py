"""
Flatpak application scanner for BigScreen Launcher
"""

import os
import subprocess
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
    
    # Paths to check for Flatpak applications (in order of preference)
    FLATPAK_APPS_DIRS = [
        # User-installed Flatpaks
        Path.home() / '.local/share/flatpak/exports/share/applications',
        # System-wide Flatpaks (accessible from sandbox)
        Path('/run/user') / str(os.getuid()) / 'flatpak/exports/share/applications',
        # Fallback: use flatpak command to find apps
    ]
    
    def __init__(self):
        self.apps: List[FlatpakApp] = []
        self.scan()
    
    def scan(self):
        """Scan for installed Flatpak applications"""
        self.apps = []
        
        # Try each directory in order
        for apps_dir in self.FLATPAK_APPS_DIRS:
            if os.path.exists(apps_dir):
                print(f"Scanning Flatpak apps directory: {apps_dir}")
                self._scan_directory(apps_dir)
                if self.apps:  # If we found apps, stop
                    break
        
        # If no apps found in directories, try using flatpak command
        if not self.apps:
            print("No Flatpak directories found, attempting to use 'flatpak list' command")
            self._scan_via_flatpak_command()
        
        # Sort alphabetically by name
        self.apps.sort(key=lambda x: x.name.lower())
        print(f"Found {len(self.apps)} Flatpak applications")
    
    def _scan_directory(self, apps_dir: Path):
        """Scan a specific directory for .desktop files"""
        try:
            desktop_files = sorted(apps_dir.glob('*.desktop'))
            
            for desktop_file in desktop_files:
                app = self._parse_desktop_file(desktop_file)
                if app:
                    self.apps.append(app)
        except Exception as e:
            print(f"Error scanning directory {apps_dir}: {e}")
    
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
    
    def _scan_via_flatpak_command(self):
        """Fallback: use 'flatpak list' command to get installed apps"""
        try:
            # Get list of installed Flatpaks
            result = subprocess.run(
                ['flatpak', 'list', '--app', '--columns=application,name'],
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode != 0:
                print(f"flatpak list command failed: {result.stderr}")
                return
            
            # Parse output (skip header line)
            lines = result.stdout.strip().split('\n')[1:]
            for line in lines:
                parts = line.split('\t', 1)
                if len(parts) == 2:
                    app_id = parts[0].strip()
                    name = parts[1].strip()
                    
                    if app_id and name:
                        app = FlatpakApp(
                            app_id=app_id,
                            name=name,
                            icon='application-x-executable',  # Default icon
                            exec=f'flatpak run {app_id}',
                            desktop_file=''
                        )
                        self.apps.append(app)
        
        except subprocess.TimeoutExpired:
            print("flatpak list command timed out")
        except FileNotFoundError:
            print("flatpak command not found")
        except Exception as e:
            print(f"Error using flatpak command: {e}")
    
    def get_apps(self) -> List[FlatpakApp]:
        """Get all discovered applications"""
        return self.apps.copy()
    
    def get_app_by_id(self, app_id: str) -> Optional[FlatpakApp]:
        """Get an application by its ID"""
        for app in self.apps:
            if app.app_id == app_id:
                return app
        return None
