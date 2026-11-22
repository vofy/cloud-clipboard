use crate::config::Config;
use crate::watcher::Status;
use ksni::menu::{CheckmarkItem, StandardItem};
use gtk::prelude::*;
use ksni::{MenuItem, ToolTip, Tray};
use std::sync::{Arc, Mutex};

pub struct ClipTray {
    pub config: Arc<Mutex<Config>>,
    last: Option<String>,
    error: Option<String>,
}

impl ClipTray {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self { config, last: None, error: None }
    }

    pub fn set(&mut self, status: &Status) {
        match status {
            Status::Fine => self.error = None,
            Status::Copied(name) => {
                self.last = Some(name.clone());
                self.error = None;
            }
            Status::Failed(e) => self.error = Some(e.clone()),
        }
    }

    fn toggle(&mut self) {
        let mut cfg = self.config.lock().unwrap();
        cfg.enabled = !cfg.enabled;
        cfg.save();
        self.error = None;
    }

    fn info(&self) -> String {
        match (&self.error, &self.last) {
            (Some(e), _) => format!("Error: {}", shorten(e)),
            (None, Some(last)) => format!("Last copied: {}", shorten(last)),
            (None, None) => "Nothing copied yet".into(),
        }
    }
}

fn shorten(text: &str) -> String {
    match text.char_indices().nth(70) {
        Some((i, _)) => format!("{}…", &text[..i]),
        None => text.to_owned(),
    }
}

fn label(text: String) -> MenuItem<ClipTray> {
    StandardItem { label: text, enabled: false, ..Default::default() }.into()
}

impl Tray for ClipTray {
    fn id(&self) -> String {
        "cloud-clipboard".into()
    }

    fn title(&self) -> String {
        "Cloud Clipboard".into()
    }

    fn icon_name(&self) -> String {
        let enabled = self.config.lock().unwrap().enabled;
        match (enabled, &self.error) {
            (false, _) => "media-playback-pause-symbolic",
            (true, Some(_)) => "dialog-warning-symbolic",
            (true, None) => "edit-paste-symbolic",
        }
        .into()
    }

    fn tool_tip(&self) -> ToolTip {
        let cfg = self.config.lock().unwrap();
        let state = if cfg.enabled { "Watching" } else { "Paused" };
        ToolTip {
            title: "Cloud Clipboard".into(),
            description: format!("{state} {}\n{}", cfg.folder, self.info()),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let cfg = self.config.lock().unwrap().clone();
        vec![
            CheckmarkItem {
                label: "Watching".into(),
                checked: cfg.enabled,
                activate: Box::new(Self::toggle),
                ..Default::default()
            }
            .into(),
            label(format!("Folder: {}", shorten(&cfg.folder))),
            label(self.info()),
            MenuItem::Separator,
            StandardItem {
                label: "Settings…".into(),
                icon_name: "preferences-system-symbolic".into(),
                activate: Box::new(|_| crate::on_main(crate::open_settings)),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|_| crate::on_main(|app| app.quit())),
                ..Default::default()
            }
            .into(),
        ]
    }
}
