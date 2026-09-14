use std::process::Command;
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

// The real game (all mechanics, real protocol, real server) is a Flash
// (.swf) client. Modern browsers/WebView2 dropped Flash entirely, and
// Ruffle (the open-source Flash emulator) can't run this exact obfuscated
// build past its loading screen (verified this session, two different
// Ruffle backends, same result). The fix: a small bundled Electron shell
// (game-shell/) with a real, unmodified, pre-2021 PepperFlash build
// injected -- this predates every Flash kill-switch/EOL block entirely, so
// it just runs the real client with zero reimplementation. This reuses
// 100% of the existing game (quests, crafting, Skylab, Company Hierarchy,
// real UI, everything) instead of rebuilding it in a web client.
//
// Electron instead of raw Chromium specifically because Electron windows
// have no browser UI at all by default (no address bar, no tabs) -- a raw
// Chromium window needs an explicit --app= flag for that, and pop-ups from
// an --app= window (e.g. the CMS's "Play" link opening Map Revolution) are
// not guaranteed to inherit the chromeless look. game-shell/main.js
// intercepts every pop-up itself and opens it as another clean shell
// window, matching how other open-source DarkOrbit clients solve this.
const GAME_URL: &str = "http://play.orbit-shadow.cloud:8081/";

#[tauri::command]
fn open_game(app: tauri::AppHandle) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| "Could not locate the launcher resources.".to_string())?;
    let shell_dir = resource_dir.join("game-shell");
    let electron_exe = shell_dir.join("node_modules").join("electron").join("dist").join("electron.exe");

    if !electron_exe.exists() {
        return Err(format!("Game shell not found: {}", electron_exe.display()));
    }

    // A fresh, per-user, writable profile -- never reuse a profile shipped
    // inside the (read-only) installed resources directory.
    let profile_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Could not locate the application data folder.".to_string())?
        .join("game-shell-profile");
    std::fs::create_dir_all(&profile_dir)
        .map_err(|_| "Could not create the game profile folder. Check the folder permissions.".to_string())?;

    Command::new(electron_exe)
        .arg(&shell_dir)
        .arg(GAME_URL)
        .env("ELECTRON_USER_DATA_DIR", &profile_dir)
        .spawn()
        .map_err(|_| "Could not open the game. Please try reinstalling Orbit Shadow.".to_string())?;

    Ok(())
}

// Checks the update manifest (see tauri.conf.json's plugins.updater.endpoints)
// on every launch and, if a newer signed build is published, downloads and
// installs it automatically, then restarts into the new version -- same
// flow as Elixia's own launcher (tauri-plugin-updater), just self-hosted:
// the manifest and installer are served as plain static files from the
// existing CMS webroot (sites/cms-next/launcher/), no separate update
// server needed.
async fn check_for_update(app: tauri::AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("updater unavailable: {e}");
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => return, // already up to date
        Err(e) => {
            eprintln!("update check failed: {e}");
            return;
        }
    };

    let _ = app.emit(
        "update-status",
        format!("Downloading update {}...", update.version),
    );

    let downloaded = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let downloaded_for_progress = downloaded.clone();
    let app_for_progress = app.clone();

    let result = update
        .download_and_install(
            move |chunk_len, total_len| {
                let total = downloaded_for_progress
                    .fetch_add(chunk_len, std::sync::atomic::Ordering::Relaxed)
                    + chunk_len;
                if let Some(total_len) = total_len {
                    let total_len = (total_len as usize).max(1);
                    let _ = app_for_progress.emit("update-progress", (total * 100) / total_len);
                }
            },
            || {
                let _ = app.emit("update-status", "Installing update...".to_string());
            },
        )
        .await;

    match result {
        Ok(()) => app.request_restart(),
        Err(e) => {
            eprintln!("update install failed: {e}");
            let _ = app.emit(
                "update-status",
                format!("Update failed: {e}"),
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(check_for_update(handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![open_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
