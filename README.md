# Cloud Clipboard

Copies every new file in a folder to the clipboard, then archives it. The folder can be on any
cloud storage [rclone](https://rclone.org) supports (Google Drive, OneDrive, Dropbox, …) or on
this computer.

Made for phone scanner apps that save to the cloud, such as
[FairScan](https://github.com/pynicolas/FairScan): scan on the phone, press Ctrl+V on the computer.

- Images are pasted as images, a PDF as an image of its first page, and text files as text.
  Other files are pasted as the file.
- When several files arrive between two checks, only the newest goes to the clipboard.
- Copied files move to an `Archive` subfolder, where they're deleted (to the cloud's trash)
  after 24 hours.
- A tray icon pauses and resumes watching and opens the settings. While paused, it makes no
  requests at all.

Works on Wayland desktops. On GNOME, the tray icon needs the
[AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/).

## Install

Download from the [latest release](https://github.com/vofy/cloud-clipboard/releases/latest):

- **Flatpak:** `flatpak install --user Cloud_Clipboard-x86_64.flatpak`, then start Cloud
  Clipboard from the app menu.
- **AppImage:** `chmod +x Cloud_Clipboard-x86_64.AppImage` and run it.

Both include rclone, wl-copy and pdftoppm. They use your existing rclone config
(`~/.config/rclone/rclone.conf`). To connect a cloud first, run `rclone config` or
[rclone-cloud-storage.sh](https://github.com/vofy/scripts/blob/main/doc/rclone-cloud-storage.md),
and name the remote in the folder setting, for example `gdrive:10_Clipboard/laptop`.

The app starts itself at login; turn that off in its settings.

## Settings

Open them from the tray menu, or by starting the app again while it runs. They're saved to
`~/.config/cloud-clipboard/config.toml` (Flatpak:
`~/.var/app/io.github.vofy.CloudClipboard/config/cloud-clipboard/config.toml`).

| Setting | Default | |
|---|---|---|
| Folder | `gdrive:10_Clipboard/<hostname>` | rclone path (`remote:path`) or local folder (`~/Scans`) |
| Archive subfolder | `Archive` | |
| Keep archived files | 24 hours | |
| Check every | 10 seconds | Shorter is faster, longer saves battery |
| Start at login | on | |

Files waiting in the folder when the app starts are archived without being copied, so starting
doesn't replace your clipboard.

## Build

```sh
cargo build --release                 # needs gtk4-devel and libadwaita-devel
flatpak-builder --user --install --force-clean build-dir build-aux/io.github.vofy.CloudClipboard.yml
build-aux/appimage.sh                 # needs wl-clipboard and poppler-utils
```

Pushing a `v*` tag builds both packages and attaches them to a GitHub release.

## Debug

Run it from a terminal to see what it does: `cloud-clipboard`, `flatpak run
io.github.vofy.CloudClipboard` or the AppImage. The tray menu also shows the last error.

| Problem | Fix |
|---|---|
| No tray icon on GNOME | Enable the AppIndicator extension. |
| Tray shows `didn't find section in config file` | The remote name in the folder doesn't exist in `rclone config`. |
| Tray shows an auth error | Sign in again: `rclone config reconnect gdrive:` |
| Nothing pasted | Is the desktop Wayland? `echo $WAYLAND_DISPLAY` must not be empty. |
