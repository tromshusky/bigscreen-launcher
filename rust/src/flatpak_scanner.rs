use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct FlatpakApp {
    pub app_id: String,
    pub name: String,
    pub icon: String,
    pub exec: String,
    pub desktop_file: String,
}

pub struct FlatpakScanner {
    apps: Vec<FlatpakApp>,
}

impl FlatpakScanner {
    pub fn new() -> Self {
        let mut scanner = FlatpakScanner { apps: Vec::new() };
        scanner.scan();
        scanner
    }

    /// Directories to check for exported .desktop files, in order.
    /// NOTE: inside the Flatpak sandbox, $HOME does NOT point at the
    /// real user home unless --filesystem=home is granted. We instead
    /// rely on $XDG_DATA_HOME, which Flatpak correctly remaps when the
    /// app is granted `--filesystem=xdg-data/flatpak:ro` — that grant
    /// bind-mounts the *real* ~/.local/share/flatpak at the sandboxed
    /// $XDG_DATA_HOME/flatpak path, so no broad home access is needed.
    fn candidate_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(data_home).join("flatpak/exports/share/applications"));
        }
        // System-wide installs (visible via --filesystem=/var/lib/flatpak/exports/share:ro)
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
        dirs
    }

    pub fn icon_search_paths() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(data_home).join("flatpak/exports/share/icons"));
        }
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/icons"));
        dirs
    }

    pub fn scan(&mut self) {
        self.apps.clear();

        for dir in Self::candidate_dirs() {
            if dir.exists() {
                eprintln!("Scanning Flatpak apps directory: {}", dir.display());
                self.scan_directory(&dir);
            }
        }

        if self.apps.is_empty() {
            eprintln!("No apps found in export directories, falling back to 'flatpak list' via flatpak-spawn");
            self.scan_via_flatpak_command();
        }

        self.apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        eprintln!("Found {} Flatpak applications", self.apps.len());
    }

    fn scan_directory(&mut self, dir: &Path) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error scanning directory {}: {e}", dir.display());
                return;
            }
        };

        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|ext| ext == "desktop").unwrap_or(false))
            .collect();
        files.sort();

        for file in files {
            if let Some(app) = Self::parse_desktop_file(&file) {
                // Avoid duplicates if the same app shows up in more than one dir
                if !self.apps.iter().any(|a| a.app_id == app.app_id) {
                    self.apps.push(app);
                }
            }
        }
    }

    fn parse_desktop_file(path: &Path) -> Option<FlatpakApp> {
        let contents = fs::read_to_string(path).ok()?;

        let mut in_desktop_entry = false;
        let mut name = None;
        let mut icon = String::new();
        let mut exec = None;
        let mut no_display = false;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_desktop_entry = trimmed == "[Desktop Entry]";
                continue;
            }
            if !in_desktop_entry || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "Name" if name.is_none() => name = Some(value.to_string()),
                    "Icon" => icon = value.to_string(),
                    "Exec" if exec.is_none() => exec = Some(value.to_string()),
                    "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }

        if no_display {
            return None;
        }

        let name = name?;
        if name.is_empty() {
            return None;
        }
        let exec = exec?;
        if exec.is_empty() {
            return None;
        }

        let app_id = path.file_stem()?.to_string_lossy().to_string();

        Some(FlatpakApp {
            app_id,
            name,
            icon,
            exec,
            desktop_file: path.to_string_lossy().to_string(),
        })
    }

    /// Falls back to querying the host's flatpak installation.
    /// A sandboxed app has no `flatpak` binary of its own — it must ask
    /// the host to run one via `flatpak-spawn --host`, which is granted
    /// by the `--talk-name=org.freedesktop.Flatpak` permission.
    fn scan_via_flatpak_command(&mut self) {
        let output = Command::new("flatpak-spawn")
            .args(["--host", "flatpak", "list", "--app", "--columns=application,name"])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Error using flatpak-spawn: {e}");
                return;
            }
        };

        if !output.status.success() {
            eprintln!(
                "flatpak list command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines().peekable();
        // Some flatpak versions print a header even with --columns; skip
        // it if the first line looks like one rather than data.
        if let Some(first) = lines.peek() {
            if first.to_lowercase().contains("application") {
                lines.next();
            }
        }

        for line in lines {
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() == 2 {
                let app_id = parts[0].trim();
                let name = parts[1].trim();
                if !app_id.is_empty() && !name.is_empty() {
                    self.apps.push(FlatpakApp {
                        app_id: app_id.to_string(),
                        name: name.to_string(),
                        icon: String::new(), // unknown -> letter-avatar fallback
                        exec: format!("flatpak run {app_id}"),
                        desktop_file: String::new(),
                    });
                }
            }
        }
    }

    pub fn get_apps(&self) -> Vec<FlatpakApp> {
        self.apps.clone()
    }

    pub fn get_app_by_id(&self, app_id: &str) -> Option<FlatpakApp> {
        self.apps.iter().find(|a| a.app_id == app_id).cloned()
    }
}