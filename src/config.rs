use gtk::glib;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Settings, stored in config.toml in the app's config folder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// What to watch: an rclone path such as "gdrive:10_Clipboard/laptop", or a local folder.
    pub folder: String,
    /// Subfolder that copied files move to.
    pub archive: String,
    /// Hours a file stays in the archive before it's deleted.
    pub archive_hours: u32,
    /// Seconds between checks for new files.
    pub interval: u32,
    /// Watching is on (the tray toggle).
    pub enabled: bool,
    /// Start at login.
    pub autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        let host = glib::host_name();
        let host = host.split('.').next().unwrap_or("computer");
        Self {
            folder: format!("gdrive:10_Clipboard/{host}"),
            archive: "Archive".into(),
            archive_hours: 24,
            interval: 10,
            enabled: true,
            autostart: true,
        }
    }
}

pub fn path() -> PathBuf {
    glib::user_config_dir().join("cloud-clipboard").join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(path()) else {
            return Self::default();
        };
        toml::from_str(&text).unwrap_or_else(|e| {
            eprintln!("Ignoring {}: {e}", path().display());
            Self::default()
        })
    }

    pub fn save(&self) {
        let path = path();
        let text = toml::to_string(self).expect("config serializes");
        let result = std::fs::create_dir_all(path.parent().unwrap()).and_then(|()| std::fs::write(&path, text));
        if let Err(e) = result {
            eprintln!("Can't save {}: {e}", path.display());
        }
    }
}
