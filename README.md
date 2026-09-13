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
