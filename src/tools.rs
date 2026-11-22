use gtk::glib;
use std::path::PathBuf;
use std::process::Command;

/// A helper program bundled next to our binary (AppImage, Flatpak), else the one on PATH.
fn find(name: &str) -> Option<PathBuf> {
    let bundled = std::env::current_exe().ok().and_then(|exe| Some(exe.parent()?.join(name)));
    let on_path = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|dir| dir.join(name));
    bundled.into_iter().chain(on_path).find(|p| p.is_file())
}

pub fn available(name: &str) -> bool {
    find(name).is_some()
}

pub fn command(name: &str) -> Command {
    Command::new(find(name).unwrap_or_else(|| name.into()))
}

/// Runs rclone and returns its output, or the last line of its error output.
pub fn rclone(args: &[&str]) -> Result<String, String> {
    let mut cmd = command("rclone");
    // Use the desktop's rclone config (and its sign-ins) also inside Flatpak, where the
    // default config folder is the app's own.
    let conf = glib::home_dir().join(".config/rclone/rclone.conf");
    if conf.is_file() {
        cmd.arg("--config").arg(conf);
    }
    let out = cmd.args(args).output().map_err(|e| format!("Can't run rclone: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(err.lines().last().unwrap_or("rclone failed").trim().to_owned())
    }
}
