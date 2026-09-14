// Orbit Shadow game shell: a small Electron wrapper whose only job is to
// display the real game (Flash, via the CMS website) in clean, chromeless
// windows -- no address bar, no browser tabs -- so it looks like a native
// app instead of a visible browser. Electron windows have no browser UI at
// all by default (unlike a raw Chromium window, which needs an explicit
// --app= flag and still hands off pop-ups to a full browser window); the
// one thing that still needs handling by hand is intercepting pop-ups
// (e.g. the CMS's "Play" link opens Map Revolution via target="_blank")
// so they also become clean Electron windows instead of default OS pop-ups.
//
// Same underlying technique (Electron + an injected PepperFlash build) as
// other open-source DarkOrbit clients use, written fresh for this project.
const { app, BrowserWindow } = require("electron");
const path = require("path");

// The launcher passes a per-user, writable profile directory (never the
// read-only installed resources folder) via this env var.
if (process.env.ELECTRON_USER_DATA_DIR) {
  app.setPath("userData", process.env.ELECTRON_USER_DATA_DIR);
}

const START_URL =
  process.argv.find((a) => a.startsWith("http://") || a.startsWith("https://")) ||
  "http://play.orbit-shadow.cloud:8081/";

// Same PepperFlash build already bundled with the launcher's Chromium copy
// (pre-2021, predates every Flash kill-switch/EOL check).
const PEPFLASH_PATH = path.join(__dirname, "flash", "pepflashplayer.dll");
app.commandLine.appendSwitch("ppapi-flash-path", PEPFLASH_PATH);
app.commandLine.appendSwitch("ignore-certificate-errors");

const WINDOW_DEFAULTS = {
  width: 1280,
  height: 900,
  autoHideMenuBar: true,
  webPreferences: {
    plugins: true,
    contextIsolation: false,
    nodeIntegration: false,
  },
};

function createWindow(url) {
  const win = new BrowserWindow(WINDOW_DEFAULTS);
  win.setMenuBarVisibility(false);

  // Anything the page tries to open in a new tab/window (target="_blank",
  // window.open) becomes another clean shell window instead of a bare
  // browser pop-up.
  win.webContents.on("new-window", (event, targetUrl) => {
    event.preventDefault();
    createWindow(targetUrl);
  });

  win.loadURL(url);
  return win;
}

app.whenReady().then(() => {
  createWindow(START_URL);
});

app.on("window-all-closed", () => {
  app.quit();
});
