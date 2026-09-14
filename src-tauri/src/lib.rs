use std::process::Command;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

// 2026-09-14 pivot: the launcher's main window now loads the REAL public
// showcase site instead of a bundled local page (src/index.html, kept on
// disk but no longer loaded by anything -- see git history if it's ever
// needed again). This guarantees the launcher is always byte-for-byte the
// same login/register module, hero, footer, everything as the website,
// with zero duplicated HTML/CSS to keep in sync by hand -- any future
// edit to the site is automatically the launcher's content too.
// "?via=launcher" is read once by the showcase site (bootstrap.php) and
// remembered for that whole session (shared with play.orbit-shadow.cloud
// via the SSO work already in place there) so header.php's own "Jouer"
// link can point at orbitshadow://play instead of its normal
// browser-only map-revolution link -- see the deep-link handling below
// for why that's necessary. "launcher_version" rides along so the site's
// own footer can show which launcher build is actually running (built
// from the real Cargo/tauri.conf.json version at compile time, in
// setup() below, rather than hardcoded here where it would silently go
// stale on the next version bump).
const LAUNCHER_HOME_URL_BASE: &str = "https://orbit-shadow.cloud/?via=launcher";

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
//
// Still needed even now that the CMS/account pages open in the user's
// real browser: Flash only ever worked through this shell, never in a
// normal browser, so the actual "Jouer" action is routed back here via
// the orbitshadow:// deep link (handle_possible_deep_link below) instead
// of trying (and failing) to run Flash in whatever browser the CMS opened
// in.
//
// Real bug found 2026-09-14: this was the bare CMS root, not the actual
// game route -- header.php's OWN non-launcher "Jouer" link (the browser
// fallback, further down in this file's history) already points at
// map-revolution, which is the page that actually embeds the SWF client
// (files/external/map_revolution.php, via jquery.flashembed.js); the
// bare root just re-renders the ordinary CMS home/dashboard. With the
// session-handoff fix (header.php's ?uid=&tok=, adopted by config.php)
// landing Electron on an authenticated dashboard instead of a login
// page, this was the one remaining reason "Jouer" never actually showed
// the game itself. Same query-string handoff still applies here --
// this constant is the base GAME_URL launch_game appends it to.
const GAME_URL: &str = "http://play.orbit-shadow.cloud:8081/map-revolution";

// Shared by both the open_game Tauri command (kept for anything that still
// calls it directly) and the orbitshadow://play deep-link handler.
//
// `session_query`: the query string off the orbitshadow://play link
// (header.php), e.g. "uid=39&tok=<32-char token>" -- see that file's own
// comment for why. Electron is a completely separate browser engine/cookie
// jar from this launcher's own WebView2 main window, so it can never just
// "already be" logged in there no matter how correctly the WebView2 side's
// own session/cookie is shared/scoped; forwarding this account's own
// already-validated session token as a query string on Electron's start URL
// (game-shell/main.js already supports overriding its start URL via argv)
// lets config.php adopt it into that fresh $_SESSION on Electron's very
// first request, the same as a real login. None when launched with no deep
// link data at all (open_game / a cold start with no orbitshadow:// argv) --
// Electron just loads GAME_URL bare, exactly as before.
fn launch_game(app: &tauri::AppHandle, session_query: Option<&str>) -> Result<(), String> {
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

    let start_url = match session_query {
        Some(query) if !query.is_empty() => format!("{GAME_URL}?{query}"),
        _ => GAME_URL.to_string(),
    };

    Command::new(electron_exe)
        .arg(&shell_dir)
        .arg(start_url)
        .env("ELECTRON_USER_DATA_DIR", &profile_dir)
        .spawn()
        .map_err(|_| "Could not open the game. Please try reinstalling Orbit Shadow.".to_string())?;

    Ok(())
}

#[tauri::command]
fn open_game(app: tauri::AppHandle) -> Result<(), String> {
    launch_game(&app, None)
}

// "Remember my password" feature request 2026-09-14: NOT a silent
// auto-login (the user explicitly asked against that) -- this is a
// password-manager-style autofill: the login page's own JS (showcase's
// index.php, only wired up when window.__TAURI__ is present, i.e. only
// ever inside this launcher, never a real visitor's browser) shows a
// dropdown of usernames that have been logged in with before and, once
// one is picked, fills the password field from here -- the player still
// clicks Log in themselves every time.
//
// The actual password is stored in the OS credential store (Windows
// Credential Manager via the `keyring` crate), never in our own file or
// database -- the same trusted, per-Windows-account secure store a real
// browser's saved passwords already live in, not a bespoke "encryption"
// this app would have to get right and keep safe the key for. Only the
// (non-secret) list of usernames that have a saved password lives in a
// plain local file, purely so the dropdown has something to show before
// any one username is picked.
const CREDENTIAL_SERVICE: &str = "cloud.orbit-shadow.launcher";
const SAVED_USERNAMES_FILE: &str = "saved_usernames.json";
const MAX_SAVED_USERNAMES: usize = 5;

fn saved_usernames_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "Could not locate the application data folder.".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|_| "Could not create the application data folder.".to_string())?;
    Ok(dir.join(SAVED_USERNAMES_FILE))
}

fn read_saved_usernames(app: &tauri::AppHandle) -> Vec<String> {
    let Ok(path) = saved_usernames_path(app) else {
        return Vec::new();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn list_saved_usernames(app: tauri::AppHandle) -> Vec<String> {
    read_saved_usernames(&app)
}

#[tauri::command]
fn save_login(app: tauri::AppHandle, username: String, password: String) -> Result<(), String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("Username is empty.".to_string());
    }

    let entry = keyring::Entry::new(CREDENTIAL_SERVICE, username)
        .map_err(|_| "Could not reach the Windows credential store.".to_string())?;
    entry
        .set_password(&password)
        .map_err(|_| "Could not save the password.".to_string())?;

    let mut usernames = read_saved_usernames(&app);
    usernames.retain(|u| u != username);
    usernames.insert(0, username.to_string());
    usernames.truncate(MAX_SAVED_USERNAMES);

    let path = saved_usernames_path(&app)?;
    let json = serde_json::to_string(&usernames)
        .map_err(|_| "Could not remember this username.".to_string())?;
    std::fs::write(&path, json).map_err(|_| "Could not remember this username.".to_string())?;

    Ok(())
}

#[tauri::command]
fn get_saved_password(username: String) -> Result<Option<String>, String> {
    let username = username.trim();
    if username.is_empty() {
        return Ok(None);
    }

    let entry = keyring::Entry::new(CREDENTIAL_SERVICE, username)
        .map_err(|_| "Could not reach the Windows credential store.".to_string())?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not read the saved password.".to_string()),
    }
}

#[tauri::command]
fn forget_login(app: tauri::AppHandle, username: String) -> Result<(), String> {
    let username = username.trim();
    if !username.is_empty() {
        if let Ok(entry) = keyring::Entry::new(CREDENTIAL_SERVICE, username) {
            let _ = entry.delete_credential();
        }
    }

    let mut usernames = read_saved_usernames(&app);
    usernames.retain(|u| u != username);
    let path = saved_usernames_path(&app)?;
    let json = serde_json::to_string(&usernames)
        .map_err(|_| "Could not update the remembered usernames.".to_string())?;
    std::fs::write(&path, json).map_err(|_| "Could not update the remembered usernames.".to_string())?;

    Ok(())
}

// Windows/Linux have no in-process event for a custom-protocol click (that's
// a macOS/iOS-only capability of tauri-plugin-deep-link) -- instead the OS
// spawns a brand new instance of orbit-shadow-launcher.exe with the URL as
// a plain CLI argument. tauri-plugin-single-instance forwards that argv to
// the ALREADY-running (minimized, per the on_navigation handler below)
// instance instead of leaving a redundant second window around; this same
// function also covers the rarer cold-start case (launcher fully closed,
// not just minimized, when "Jouer" is clicked) via std::env::args() in
// setup() below -- both hand it the exact same argv shape.
fn handle_possible_deep_link(app: &tauri::AppHandle, args: &[String]) {
    let Some(link) = args.iter().find(|a| a.starts_with("orbitshadow://play")) else {
        return;
    };
    // header.php appends ?uid=&tok= (see that file's own comment) so the
    // game shell's own separate cookie jar can adopt this account's
    // already-validated session -- forward it through verbatim, still
    // just an opaque query string to this side. Plain split instead of
    // pulling in a URL parser for two fields already in exactly the
    // "key=value&key=value" shape Electron's start URL needs appended.
    let session_query = link.split_once('?').map(|(_, query)| query);
    if let Err(e) = launch_game(app, session_query) {
        eprintln!("orbitshadow://play deep link: game launch failed: {e}");
    }
}

// Checks the update manifest (see tauri.conf.json's plugins.updater.endpoints)
// on every launch and, if a newer signed build is published, downloads and
// installs it automatically, then restarts into the new version -- same
// flow as Elixia's own launcher (tauri-plugin-updater), just self-hosted:
// the manifest and installer are served as plain static files from the
// existing CMS webroot (sites/cms-next/launcher/), no separate update
// server needed.
//
// Real gap found via user feedback: this used to run silently behind
// whatever window was already showing, with update-status/update-progress
// events going nowhere once the main window started loading the live site
// directly (nothing there listens for Tauri events) -- from the user's
// side the app looked like it "opened and closed right away" whenever an
// update WAS being installed, with the NSIS installer's own window being
// the very first visible sign anything was happening. `window` here is
// the local splash screen (src/index.html + main.js, which DOES listen
// for both events) created first in setup() below, specifically so this
// has somewhere to show progress; once this resolves either way (up to
// date, or install failed and the user should still be able to play),
// it navigates that same window on to the real site. A successful
// install skips the navigate -- request_restart() is about to reload the
// whole app into the new version anyway, which repeats this same check
// fresh (finds itself already up to date, navigates normally).
async fn check_for_update(app: tauri::AppHandle, window: tauri::WebviewWindow, home_url: String) {
    let go_to_home = || {
        let _ = window.navigate(home_url.parse().expect("home_url is a valid, hardcoded URL"));
    };

    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("updater unavailable: {e}");
            go_to_home();
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            go_to_home(); // already up to date
            return;
        }
        Err(e) => {
            eprintln!("update check failed: {e}");
            go_to_home();
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
            let _ = app.emit("update-status", format!("Update failed: {e}"));
            // Give the failure message a moment on screen -- go_to_home()
            // navigates this same window away immediately otherwise,
            // which would make the error invisible in practice. Don't
            // strand the user here permanently either, though: they
            // should still be able to keep playing on the current
            // version once they've seen it.
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            go_to_home();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered before every other plugin (tauri-plugin-
        // single-instance's own documented requirement). Its callback is
        // how a warm-started orbitshadow://play click (launcher already
        // running, minimized) reaches us -- see handle_possible_deep_link.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            handle_possible_deep_link(app, &argv);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Best-effort self-registration of the orbitshadow:// scheme as
            // a repair path (idempotent, safe every launch) in case the
            // NSIS install step for it was ever skipped -- e.g. a portable
            // copy, or upgrading in-place from a version predating this.
            #[cfg(desktop)]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }

            // Cold start: the launcher wasn't running at all when
            // "Jouer" was clicked, so the OS starts a fresh instance with
            // the URL as a normal argv entry -- same shape single-instance
            // hands the warm-start case above, just read directly here.
            handle_possible_deep_link(&app.handle().clone(), &std::env::args().collect::<Vec<_>>());

            // Main window: the real live website (see LAUNCHER_HOME_URL's
            // own comment for why), not a bundled local page. Everything
            // -- login, register, and once logged in the whole CMS on
            // play.orbit-shadow.cloud -- stays embedded in this SAME
            // window (WebView2, itself Chromium-based) rather than
            // popping out to the OS's default browser: an earlier version
            // of this did open the CMS externally, but that's explicitly
            // NOT what's wanted here.
            //
            // The one link that can never just navigate normally is
            // orbitshadow://play (header.php's "Jouer" link once ?via=
            // launcher is set -- see config.php/header.php): WebView2 has
            // no built-in handler for a non-http(s) scheme and would just
            // fail the navigation silently, so that one case alone is
            // still handed to the OS explicitly via the opener plugin,
            // which resolves it through the same registered protocol
            // handler as clicking it from a real browser would (Windows
            // then re-invokes this exe, caught by tauri-plugin-single-
            // instance / handle_possible_deep_link, same as ever).
            let home_url = format!(
                "{LAUNCHER_HOME_URL_BASE}&launcher_version={}",
                app.package_info().version
            );
            let app_for_nav = app.handle().clone();
            // Starts on the LOCAL splash page (src/index.html), not the
            // real site directly -- see check_for_update's own comment
            // for why: that's the one thing left that can show real
            // progress for an update download/install, which otherwise
            // happens with zero visible feedback (a real, reported gap).
            // on_navigation is set up here at window-creation time but
            // stays active for every navigation this window makes for
            // its whole lifetime, including the .navigate() call below
            // that leaves this page for the real site once the update
            // check resolves -- so orbitshadow://play still gets caught
            // correctly no matter when it's clicked.
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Orbit Shadow")
                .inner_size(1280.0, 800.0)
                .resizable(true)
                .maximized(true)
                .on_navigation(move |url| {
                    if url.scheme() == "orbitshadow" {
                        let _ = app_for_nav.opener().open_url(url.as_str(), None::<&str>);
                        return false;
                    }
                    true
                })
                .build()?;

            tauri::async_runtime::spawn(check_for_update(app.handle().clone(), window, home_url));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_game,
            save_login,
            get_saved_password,
            list_saved_usernames,
            forget_login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
