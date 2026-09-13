const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const playBtn = document.getElementById("play-btn");
const statusMsg = document.getElementById("status-msg");

// The Rust side checks for an update on every startup and, if one exists,
// downloads + installs it automatically, then restarts -- these events are
// just so the user sees it happening instead of the window looking frozen.
listen("update-status", (event) => {
  statusMsg.textContent = event.payload;
});
listen("update-progress", (event) => {
  statusMsg.textContent = `Téléchargement de la mise à jour... ${event.payload}%`;
});

playBtn.addEventListener("click", async () => {
  playBtn.disabled = true;
  statusMsg.textContent = "Lancement du jeu...";
  try {
    await invoke("open_game");
    statusMsg.textContent = "Le jeu est lancé dans une fenêtre séparée.";
  } catch (err) {
    statusMsg.textContent = "Erreur : " + err;
  } finally {
    playBtn.disabled = false;
  }
});
