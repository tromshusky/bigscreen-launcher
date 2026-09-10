#!/usr/bin/env python3
"""
BigScreen Launcher - HTPC application for launching Flatpak apps
"""

import sys
import os
import gi
import subprocess
import threading
import time
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional
import json

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gdk, GLib

from config import Config
from flatpak_scanner import FlatpakScanner
from ui.main_window import MainWindow


class BigScreenLauncher:
    def __init__(self):
        self.app = Gtk.Application(application_id='com.github.tromshusky.bigscreenLauncher')
        self.app.connect('activate', self.on_activate)
        self.config = Config()
        self.scanner = FlatpakScanner()
        self.window = None

    def on_activate(self, app):
        """Handle application activation"""
        if self.window is None:
            self.window = MainWindow(self.app, self.config, self.scanner)
        self.window.present()

    def run(self, argv):
        """Run the application"""
        return self.app.run(argv)


def main():
    launcher = BigScreenLauncher()
    sys.exit(launcher.run(sys.argv))


if __name__ == '__main__':
    main()
