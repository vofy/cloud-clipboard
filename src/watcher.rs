use crate::clipboard;
use crate::config::Config;
use crate::tools::rclone;
use gtk::glib;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

const CLEANUP_EVERY: Duration = Duration::from_secs(3600);

pub enum Status {
    /// Watching works; nothing new.
    Fine,
    Copied(String),
    Failed(String),
}

/// "gdrive:" + "a" is "gdrive:a", "gdrive:x" + "a" is "gdrive:x/a".
fn join(folder: &str, name: &str) -> String {
    if folder.ends_with(':') || folder.ends_with('/') {
        format!("{folder}{name}")
    } else {
        format!("{folder}/{name}")
    }
}

fn expand_home(folder: &str) -> String {
    match folder.strip_prefix("~/") {
        Some(rest) => glib::home_dir().join(rest).to_string_lossy().into_owned(),
        None => folder.to_owned(),
    }
}

/// Downloads a file into an empty cache folder and puts it on the clipboard.
/// The outer error means the download failed (try again); the inner one that the file was
/// downloaded but couldn't be copied (don't try again).
fn fetch(remote: &str, name: &str, cache: &Path) -> Result<Result<(), String>, String> {
    let _ = std::fs::remove_dir_all(cache);
    let file = cache.join(name);
    rclone(&["copyto", remote, &file.to_string_lossy()])?;
    Ok(clipboard::copy(&file))
}

/// Checks the folder every few seconds. Every new file goes to the clipboard (the newest one,
/// when several arrive between two checks) and then moves to the archive subfolder, which is
/// cleared of files older than the configured age once an hour. While paused it does nothing.
pub fn spawn(config: Arc<Mutex<Config>>, report: impl Fn(Status) + Send + 'static) {
    std::thread::spawn(move || {
        let cache = glib::user_cache_dir().join("cloud-clipboard");
        // The folder the last check listed. Files waiting when watching starts, or after the
        // folder changes, are only archived, so starting doesn't replace the clipboard.
        let mut watched: Option<String> = None;
        let mut next_cleanup = Instant::now();
        loop {
            let cfg = config.lock().unwrap().clone();
            if !cfg.enabled {
                sleep(Duration::from_secs(1));
                continue;
            }
            let folder = expand_home(cfg.folder.trim());
            let archive = join(&folder, cfg.archive.trim());
            let first = watched.as_deref() != Some(folder.as_str());
            if first || Instant::now() >= next_cleanup {
                let _ = rclone(&["mkdir", &archive]);
                let age = format!("{}h", cfg.archive_hours);
                if let Err(e) = rclone(&["delete", &archive, "--min-age", &age]) {
                    report(Status::Failed(e));
                }
                next_cleanup = Instant::now() + CLEANUP_EVERY;
            }

            match rclone(&["lsf", &folder, "--files-only", "--format", "tp", "--separator", ";"]) {
                Err(e) => report(Status::Failed(e)),
                Ok(list) => {
                    let mut files: Vec<(&str, &str)> = list.lines().filter_map(|l| l.split_once(';')).collect();
                    files.sort();
                    let mut archive_all = true;
                    if let (false, Some((_, newest))) = (first, files.last()) {
                        match fetch(&join(&folder, newest), newest, &cache) {
                            Ok(Ok(())) => report(Status::Copied(newest.to_string())),
                            Ok(Err(e)) => report(Status::Failed(e)),
                            Err(e) => {
                                report(Status::Failed(e));
                                archive_all = false;
                            }
                        }
                    } else {
                        report(Status::Fine);
                    }
                    if archive_all {
                        for (_, name) in &files {
                            if let Err(e) = rclone(&["moveto", &join(&folder, name), &join(&archive, name)]) {
                                report(Status::Failed(e));
                            }
                        }
                    }
                    watched = Some(folder.clone());
                }
            }

            // Wait for the next check, but react to pausing or a new folder within a second.
            for _ in 0..cfg.interval.max(1) {
                sleep(Duration::from_secs(1));
                let now = config.lock().unwrap();
                if !now.enabled || now.folder != cfg.folder {
                    break;
                }
            }
        }
    });
}
