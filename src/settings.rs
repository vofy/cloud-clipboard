use crate::autostart;
use crate::config::Config;
use adw::prelude::*;
use std::sync::{Arc, Mutex};

/// The settings window. Every change is saved right away; the watcher picks it up at its
/// next check.
pub fn open(config: Arc<Mutex<Config>>) {
    let cfg = config.lock().unwrap().clone();
    let update = move |change: &dyn Fn(&mut Config)| {
        let mut cfg = config.lock().unwrap();
        change(&mut cfg);
        cfg.save();
    };
    let update = std::rc::Rc::new(update);

    let folder = adw::EntryRow::builder()
        .title("Folder")
        .text(&cfg.folder)
        .show_apply_button(true)
        .build();
    let archive = adw::EntryRow::builder()
        .title("Archive subfolder")
        .text(&cfg.archive)
        .show_apply_button(true)
        .build();
    let watched = adw::PreferencesGroup::builder()
        .title("Watched folder")
        .description(
            "New files here are copied to the clipboard, then moved to the archive subfolder. \
             Use an rclone path such as gdrive:10_Clipboard/laptop, or a local folder.",
        )
        .build();
    watched.add(&folder);
    watched.add(&archive);

    let hours = adw::SpinRow::with_range(1.0, 720.0, 1.0);
    hours.set_title("Keep archived files (hours)");
    hours.set_value(cfg.archive_hours.into());
    let interval = adw::SpinRow::with_range(2.0, 3600.0, 1.0);
    interval.set_title("Check every (seconds)");
    interval.set_subtitle("Longer saves battery");
    interval.set_value(cfg.interval.into());
    let timing = adw::PreferencesGroup::builder().title("Timing").build();
    timing.add(&hours);
    timing.add(&interval);

    let login = adw::SwitchRow::builder().title("Start at login").active(cfg.autostart).build();
    let startup = adw::PreferencesGroup::new();
    startup.add(&login);

    let page = adw::PreferencesPage::new();
    page.add(&watched);
    page.add(&timing);
    page.add(&startup);
    let dialog = adw::PreferencesDialog::builder().title("Cloud Clipboard").build();
    dialog.add(&page);

    let u = update.clone();
    folder.connect_apply(move |row| u(&|c| c.folder = row.text().trim().into()));
    let u = update.clone();
    archive.connect_apply(move |row| u(&|c| c.archive = row.text().trim().into()));
    let u = update.clone();
    hours.connect_value_notify(move |row| u(&|c| c.archive_hours = row.value() as u32));
    let u = update.clone();
    interval.connect_value_notify(move |row| u(&|c| c.interval = row.value() as u32));
    login.connect_active_notify(move |row| {
        update(&|c| c.autostart = row.is_active());
        autostart::apply(row.is_active());
    });

    dialog.present(None::<&gtk::Widget>);
}
