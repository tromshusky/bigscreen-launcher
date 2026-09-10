"""
Configuration management for BigScreen Launcher
"""

import json
import os
from pathlib import Path
from typing import Dict, Any


class Config:
    """Handle application configuration and persistence"""
    
    DEFAULT_CONFIG = {
        'steam_launch_mode': 'bigpicture',  # 'bigpicture' or 'pc'
        'grid_rows': 1,
        'grid_columns': 4,
        'scrolling_mode': 'continuous',  # 'continuous' or 'pages'
    }
    
    def __init__(self):
        self.config_dir = Path.home() / '.config' / 'bigscreen-launcher'
        self.config_file = self.config_dir / 'settings.json'
        self.config = self.DEFAULT_CONFIG.copy()
        self.load()
    
    def load(self):
        """Load configuration from file"""
        if self.config_file.exists():
            try:
                with open(self.config_file, 'r') as f:
                    saved_config = json.load(f)
                    self.config.update(saved_config)
            except (json.JSONDecodeError, IOError) as e:
                print(f"Warning: Could not load config file: {e}")
                self.config = self.DEFAULT_CONFIG.copy()
        else:
            self.save()
    
    def save(self):
        """Save configuration to file"""
        self.config_dir.mkdir(parents=True, exist_ok=True)
        try:
            with open(self.config_file, 'w') as f:
                json.dump(self.config, f, indent=2)
        except IOError as e:
            print(f"Warning: Could not save config file: {e}")
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value"""
        return self.config.get(key, default)
    
    def set(self, key: str, value: Any):
        """Set a configuration value and save"""
        self.config[key] = value
        self.save()
    
    def get_all(self) -> Dict[str, Any]:
        """Get all configuration as a dictionary"""
        return self.config.copy()
