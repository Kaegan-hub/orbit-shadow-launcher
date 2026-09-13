'use strict';

const motionButton = document.getElementById('motion-toggle');
const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
let motionOverride = null;
try { motionOverride = localStorage.getItem('orbit-shadow-motion'); } catch (_) { /* Storage can be unavailable in local previews. */ }

function setMotion(enabled) {
  enabled = enabled && !motionPreference.matches;
  document.body.classList.toggle('motion-paused', !enabled);
  motionButton.disabled = motionPreference.matches;
  motionButton.setAttribute('aria-pressed', String(enabled));
  motionButton.querySelector('span').textContent = motionPreference.matches ? 'Animations réduites' : enabled ? 'Animations activées' : 'Animations désactivées';
}

setMotion(!motionPreference.matches && motionOverride !== 'off');
motionButton.addEventListener('click', () => {
  if (motionPreference.matches) return;
  const enabled = motionButton.getAttribute('aria-pressed') !== 'true';
  motionOverride = enabled ? 'on' : 'off';
  setMotion(enabled);
  try { localStorage.setItem('orbit-shadow-motion', motionOverride); } catch (_) { /* The toggle still works without persistence. */ }
});
motionPreference.addEventListener('change', event => {
  if (event.matches) setMotion(false);
  else setMotion(motionOverride !== 'off');
});

const playButton = document.getElementById('play-btn');
const previewDialog = document.getElementById('preview-dialog');
const statusMessage = document.getElementById('status-msg');
const invoke = window.__TAURI__?.core?.invoke;
if (typeof invoke === 'function') {
  document.querySelector('.preview-badge').textContent = 'LAUNCHER';
}

playButton.addEventListener('click', async () => {
  if (playButton.disabled) return;
  if (typeof invoke !== 'function') {
    previewDialog.showModal();
    return;
  }

  playButton.disabled = true;
  playButton.setAttribute('aria-busy', 'true');
  playButton.querySelector('strong').textContent = 'LANCEMENT…';
  statusMessage.hidden = false;
  statusMessage.dataset.state = 'loading';
  statusMessage.textContent = 'Lancement du jeu…';
  try {
    // Native command used by the installed Orbit Shadow launcher.
    await invoke('open_game');
    statusMessage.dataset.state = 'success';
    statusMessage.textContent = 'Le jeu est lancé dans une fenêtre séparée.';
  } catch (error) {
    statusMessage.dataset.state = 'error';
    const detail = typeof error === 'string' ? error : error?.message;
    statusMessage.textContent = detail ? `Impossible de lancer le jeu : ${detail}` : 'Impossible de lancer le jeu. Veuillez réessayer.';
  } finally {
    playButton.disabled = false;
    playButton.removeAttribute('aria-busy');
    playButton.querySelector('strong').textContent = 'JOUER';
  }
});
previewDialog.addEventListener('click', event => {
  if (event.target !== previewDialog) return;
  const bounds = previewDialog.getBoundingClientRect();
  if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) previewDialog.close();
});
