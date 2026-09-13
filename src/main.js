const { invoke } = window.__TAURI__.core;

const playBtn = document.getElementById("play-btn");
const statusMsg = document.getElementById("status-msg");

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
