//! Cloud Clipboard: copies every new file in a watched folder (any rclone remote, or a local
//! folder) to the clipboard, then archives it. Lives in the tray.

mod autostart;
mod clipboard;
mod config;
mod settings;
mod tools;
mod tray;
mod watcher;

use adw::prelude::*;
use gtk::{gio, glib};
use ksni::blocking::TrayMethods;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use watcher::Status;

pub const APP_ID: &str = "io.github.vofy.CloudClipboard";

static CONFIG: OnceLock<Arc<Mutex<config::Config>>> = OnceLock::new();

/// Runs `f` on the GTK main thread (the tray menu runs on its own thread).
pub fn on_main(f: impl FnOnce(&adw::Application) + Send + 'static) {
    glib::MainContext::default().invoke(move || {
        if let Some(app) = gio::Application::default().and_downcast::<adw::Application>() {
            f(&app);
        }
    });
}

pub fn open_settings(_app: &adw::Application) {
    settings::open(CONFIG.get().unwrap().clone());
}

/// A desktop notification that replaces the previous one instead of piling up.
fn notify(summary: &str, body: &str) {
    static LAST_ID: AtomicU32 = AtomicU32::new(0);
    let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
        return;
    };
    let hints: HashMap<String, glib::Variant> = HashMap::new();
    let args = (
        "Cloud Clipboard",
        LAST_ID.load(Ordering::Relaxed),
        "edit-paste-symbolic",
        summary,
        body,
        Vec::<String>::new(),
        hints,
        5000i32,
    );
    let reply = bus.call_sync(
        Some("org.freedesktop.Notifications"),
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        Some(&args.to_variant()),
        Some(glib::VariantTy::new("(u)").unwrap()),
        gio::DBusCallFlags::NONE,
        -1,
        gio::Cancellable::NONE,
    );
    if let Some((id,)) = reply.ok().and_then(|r| r.get::<(u32,)>()) {
        LAST_ID.store(id, Ordering::Relaxed);
    }
}

fn start(app: &adw::Application) {
    // Keep running with no window open; the tray icon is the interface.
    std::mem::forget(app.hold());
    let config = CONFIG.get().unwrap().clone();
    if !config::path().exists() {
        config.lock().unwrap().save();
    }
    autostart::apply(config.lock().unwrap().autostart);

    // Flatpak only lets the app own D-Bus names under its own ID, so register the tray under
    // the connection's unique name there.
    let in_flatpak = std::env::var_os("FLATPAK_ID").is_some();
    let tray = match tray::ClipTray::new(config.clone()).disable_dbus_name(in_flatpak).spawn() {
        Ok(handle) => Some(handle),
        Err(e) => {
            eprintln!("No tray icon ({e}). GNOME needs the AppIndicator extension.");
            None
        }
    };
    watcher::spawn(config, move |status| {
        if let Status::Copied(name) = &status {
            println!("Copied {name}");
            notify("Copied to the clipboard", name);
        } else if let Status::Failed(e) = &status {
            eprintln!("{e}");
        }
        if let Some(tray) = &tray {
            tray.update(|t| t.set(&status));
        }
    });
}

fn main() -> glib::ExitCode {
    CONFIG.set(Arc::new(Mutex::new(config::Config::load()))).unwrap();
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(start);
    // The first launch only starts the tray; launching the app again opens the settings.
    let launched = std::cell::Cell::new(false);
    app.connect_activate(move |app| {
        if launched.replace(true) {
            open_settings(app);
        }
    });
    app.run()
}
