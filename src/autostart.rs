use crate::APP_ID;
use gtk::glib;

/// Starts the app at login through the desktop's autostart folder. Inside Flatpak this needs
/// --filesystem=xdg-config/autostart:create.
pub fn apply(enabled: bool) {
    let file = glib::home_dir().join(".config/autostart").join(format!("{APP_ID}.desktop"));
    if !enabled {
        let _ = std::fs::remove_file(&file);
        return;
    }
    let exec = if let Ok(id) = std::env::var("FLATPAK_ID") {
        format!("flatpak run {id}")
    } else if let Ok(appimage) = std::env::var("APPIMAGE") {
        format!("\"{appimage}\"")
    } else {
        let exe = std::env::current_exe().unwrap_or_else(|_| "cloud-clipboard".into());
        format!("\"{}\"", exe.display())
    };
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Cloud Clipboard\nExec={exec}\nIcon={APP_ID}\nNoDisplay=true\n"
    );
    let result = std::fs::create_dir_all(file.parent().unwrap()).and_then(|()| std::fs::write(&file, entry));
    if let Err(e) = result {
        eprintln!("Can't write {}: {e}", file.display());
    }
}
