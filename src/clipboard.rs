use crate::tools;
use gtk::glib;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

/// Puts a downloaded file on the clipboard: images as images, a PDF as an image of its first
/// page (when pdftoppm is available), text as text, and anything else as the file itself.
pub fn copy(file: &Path) -> Result<(), String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let read = |f: &Path| std::fs::read(f).map_err(|e| format!("Can't read {}: {e}", f.display()));
    match ext.as_str() {
        "png" | "gif" | "webp" | "bmp" => wl_copy(&format!("image/{ext}"), &read(file)?),
        "jpg" | "jpeg" => wl_copy("image/jpeg", &read(file)?),
        "svg" => wl_copy("image/svg+xml", &read(file)?),
        "pdf" if tools::available("pdftoppm") => {
            let page = file.with_extension("");
            let ok = tools::command("pdftoppm")
                .args(["-png", "-r", "150", "-f", "1", "-l", "1", "-singlefile"])
                .arg(file)
                .arg(&page)
                .status()
                .is_ok_and(|s| s.success());
            if !ok {
                return Err(format!("Can't read {}", file.display()));
            }
            wl_copy("image/png", &read(&page.with_extension("png"))?)
        }
        "txt" | "md" | "csv" | "log" | "json" | "xml" | "yaml" | "yml" | "html" => {
            wl_copy("text/plain;charset=utf-8", &read(file)?)
        }
        _ => {
            let uri = glib::filename_to_uri(file, None).map_err(|e| e.to_string())?;
            wl_copy("text/uri-list", format!("{uri}\r\n").as_bytes())
        }
    }
}

fn wl_copy(mime: &str, data: &[u8]) -> Result<(), String> {
    let mut child = tools::command("wl-copy")
        .args(["--type", mime])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Can't run wl-copy: {e}"))?;
    // wl-copy keeps serving the clipboard in the background once its input is closed.
    child.stdin.take().unwrap().write_all(data).map_err(|e| e.to_string())?;
    match child.wait() {
        Ok(s) if s.success() => Ok(()),
        _ => Err("wl-copy failed".into()),
    }
}
