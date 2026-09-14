'use strict';

// This page is only the local startup splash (see index.html's own
// comment) -- it exists purely to show update-check/download/install
// progress before the Rust side (setup() in src-tauri/src/lib.rs)
// navigates this same window to the real orbit-shadow.cloud site. Nothing
// here needs to survive past that point, and nothing on the real site
// depends on this script running.

const statusMessage = document.getElementById('status-msg');
const listen = window.__TAURI__?.event?.listen;

if (typeof listen === 'function') {
  listen('update-status', (event) => {
    statusMessage.textContent = event.payload;
  });
  listen('update-progress', (event) => {
    statusMessage.textContent = `Downloading update… ${event.payload}%`;
  });
}
