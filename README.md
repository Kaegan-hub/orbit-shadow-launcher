# Orbit Shadow Launcher

A Windows launcher built with Tauri 2 that opens Orbit Shadow in the bundled game browser.

## Interface

The launcher features an eclipsed planet, an orbital mark and a Play button. The Discord button in the top right opens https://discord.gg/orbit-shadow in the default browser. Subtle background animations can be paused, and the system’s reduced motion preference is respected. The interface fits the native 640 × 480 window. All launcher text, accessibility labels, launch messages and installer screens are in English.

The frontend is in `src/` and runs without a CDN or remote fonts. Open `src/index.html` in a browser to preview it. In Tauri, Play calls the existing native `open_game` command. Concurrent requests are blocked, and the button becomes available again after success or failure.

## Development and builds

Windows prerequisites: Node.js, stable Rust MSVC, Microsoft C++ Build Tools with the Windows SDK, and WebView2. See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm test
npm run tauri -- dev
npm run tauri -- build -- --locked
```

The executable is generated at `src-tauri/target/release/orbit-shadow-launcher.exe`, and the NSIS installer is in `src-tauri/target/release/bundle/nsis/`. The existing bundle configuration includes `src-tauri/resources/chromium`. The installer is configured to use English only.

The `Build Orbit Shadow for Windows` workflow builds this redesign branch and saves the executable and installer as GitHub Actions artifacts. It can also be run manually once present on the default branch. It does not publish a release.

## Validation

`npm test` checks preview mode, the `open_game` contract, protection against concurrent launches, recovery after errors, English status text and animation preferences. These tests simulate the native API and do not connect to the game.

The original background, `src/assets/orbit-eclipse.png`, was created for this interface with ImageGen. Its prompt is saved in `design/IMAGE-PROMPT.txt`.

## Automatic updates

The launcher checks `https://play.orbit-shadow.cloud/launcher/latest.json` on
every startup (`tauri-plugin-updater`) and silently installs any newer,
correctly signed version, then restarts.

**Known limitation: Windows SmartScreen.** The downloaded installer has no
Authenticode signature (only the plugin's minisign signature, which protects
the update's integrity but isn't recognized by Windows). SmartScreen may show
"Windows protected your PC" and block the install until the user clicks "More
info → Run anyway". Decision made (2026-09-13): leave it as-is for now -- the
file's SmartScreen reputation improves as more people run it without incident.
A code-signing certificate (Authenticode, paid) would remove the warning from
the very first install if this becomes a real problem.

### Publishing a new version

1. Bump the version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and
   `package.json` (all three must match).
2. Build while signing with the private key (never committed, see
   `src-tauri/signing.key` -- gitignored):
   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\signing.key -Raw
   $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<key password>"
   npm run tauri build
   ```
3. Copy `src-tauri/target/release/bundle/nsis/Orbit Shadow_<version>_x64-setup.exe`
   to `C:\OrbitShadow\sites\cms-next\launcher\` on the VPS (replacing the old one).
4. Update `C:\OrbitShadow\sites\cms-next\launcher\latest.json`: new `version`,
   new `url`, and the contents of the `.sig` file generated next to the
   installer (in the `signature` field).
