use std::process::Command;
use tauri::Manager;

// The real game (all mechanics, real protocol, real server) is a Flash
// (.swf) client. Modern browsers/WebView2 dropped Flash entirely, and
// Ruffle (the open-source Flash emulator) can't run this exact obfuscated
// build past its loading screen (verified this session, two different
// Ruffle backends, same result). The actual fix: bundle a real, unmodified,
// pre-2021 Chromium build (49.0.2623.112) with its official PepperFlash
// plugin still present -- this predates every Flash kill-switch/EOL block
// entirely, so it just runs the real client with zero reimplementation.
// This reuses 100% of the existing game (quests, crafting, Skylab, Company
// Hierarchy, real UI, everything) instead of rebuilding it in a web client.
const GAME_URL: &str = "http://play.orbit-shadow.cloud:8081/";
const PEPPERFLASH_VERSION: &str = "21.0.0.213";

#[tauri::command]
fn open_game(app: tauri::AppHandle) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?;
    let chromium_dir = resource_dir.join("chromium");
    let chrome_exe = chromium_dir.join("chrome.exe");
    let pepflash_dll = chromium_dir
        .join("49.0.2623.112")
        .join("PepperFlash")
        .join("pepflashplayer.dll");

    if !chrome_exe.exists() {
        return Err(format!("chrome.exe introuvable: {}", chrome_exe.display()));
    }

    // A fresh, per-user, writable profile -- never reuse a profile shipped
    // inside the (read-only) installed resources directory.
    let profile_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("app_local_data_dir: {e}"))?
        .join("chromium-profile");
    std::fs::create_dir_all(&profile_dir).map_err(|e| format!("create profile dir: {e}"))?;

    Command::new(chrome_exe)
        .current_dir(&chromium_dir)
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-sync")
        .arg("--disable-translate")
        .arg("--disable-component-update")
        .arg("--allow-outdated-plugins")
        .arg("--no-proxy-server")
        .arg(format!("--ppapi-flash-path={}", pepflash_dll.display()))
        .arg(format!("--ppapi-flash-version={PEPPERFLASH_VERSION}"))
        .arg(GAME_URL)
        .spawn()
        .map_err(|e| format!("spawn chrome.exe: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
