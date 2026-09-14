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

// Security audit finding (2026-09-14), fixed: only ever navigate this shell
// -- main window OR any pop-up it opens -- to the real game's own domain
// family. Without this, ANY page the shell ever loads (including a
// pop-up a compromised/MITM'd page asks for) could send it to an
// attacker-controlled URL, which would then run with the Flash plugin
// enabled. Applies to both the initial load and every subsequent
// navigation/pop-up, not just the pop-up path, since a same-window
// navigation is exactly as dangerous as a pop-up here.
const ALLOWED_HOST_SUFFIX = ".orbit-shadow.cloud";
const ALLOWED_HOST_EXACT = "orbit-shadow.cloud";
function isAllowedUrl(targetUrl) {
  try {
    const parsed = new URL(targetUrl);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return false;
    return parsed.hostname === ALLOWED_HOST_EXACT || parsed.hostname.endsWith(ALLOWED_HOST_SUFFIX);
  } catch {
    return false;
  }
}

// Security audit finding (2026-09-14), fixed: this used to blanket-disable
// TLS certificate validation for the whole embedded Chromium
// (--ignore-certificate-errors), which is a critical MITM vulnerability --
// anyone on the network path (public wifi, a compromised router, DNS
// hijacking) could intercept the connection, present any certificate at
// all, and this would accept it silently. That includes the login page the
// real player types their real password into. There is no good reason to
// ever disable this globally; if a specific legacy-root-trust problem
// shows up in practice, the correct fix is pinning the one known server
// certificate (session.setCertificateVerifyProc, scoped to this app), not
// disabling validation for every connection this process ever makes.
//
// Same PepperFlash build already bundled with the launcher's Chromium copy
// (pre-2021, predates every Flash kill-switch/EOL check). Flash itself has
// known, permanently-unpatched vulnerabilities (EOL since Dec 2020) -- an
// accepted, structural risk of reusing the real Flash client while the
// separate web-client-port project isn't a full replacement yet. Removing
// the cert-bypass and restricting navigation above closes off the REMOTE
// delivery paths that would let an untrusted party get their own content
// into this Flash-enabled context in the first place, which is the part
// actually fixable without rebuilding the client.
const PEPFLASH_PATH = path.join(__dirname, "flash", "pepflashplayer.dll");
app.commandLine.appendSwitch("ppapi-flash-path", PEPFLASH_PATH);

const WINDOW_DEFAULTS = {
  width: 1280,
  height: 900,
  fullscreen: true,
  autoHideMenuBar: true,
  webPreferences: {
    plugins: true,
    // Security audit finding (2026-09-14), fixed: contextIsolation:false is
    // a known Electron anti-pattern (Electron itself has defaulted this to
    // true since v12, specifically because of this). No preload script
    // attaches anything privileged here, so there's nothing that depends
    // on the old value -- flipping it is a pure hardening with no
    // functional change.
    contextIsolation: true,
    nodeIntegration: false,
  },
};

function createWindow(url) {
  if (!isAllowedUrl(url)) {
    console.error("Refusing to open a window for an untrusted URL:", url);
    return null;
  }

  const win = new BrowserWindow(WINDOW_DEFAULTS);
  win.setMenuBarVisibility(false);

  // Anything the page tries to open in a new tab/window (target="_blank",
  // window.open) becomes another clean shell window instead of a bare
  // browser pop-up -- but only within the trusted domain family (see
  // isAllowedUrl above).
  win.webContents.on("new-window", (event, targetUrl) => {
    event.preventDefault();
    createWindow(targetUrl);
  });

  // Same restriction for a same-window navigation (the page redirecting
  // itself), not just pop-ups -- both are equally capable of steering this
  // Flash-enabled shell to attacker content.
  win.webContents.on("will-navigate", (event, targetUrl) => {
    if (!isAllowedUrl(targetUrl)) {
      console.error("Refusing to navigate to an untrusted URL:", targetUrl);
      event.preventDefault();
    }
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
