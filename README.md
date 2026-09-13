# Orbit Shadow Launcher

Launcher Windows Tauri 2 pour ouvrir Orbit Shadow dans le navigateur de jeu fourni.

## Interface

Le launcher affiche une planète en éclipse, un symbole orbital vectoriel et un bouton Jouer. Le fond est animé avec discrétion ; le bouton « Animations » permet de le figer et la préférence système de réduction des animations est respectée. L’interface tient dans la fenêtre native de 640 × 480 pixels.

Le frontend se trouve dans `src/` et fonctionne sans CDN ni police distante. Ouvrir `src/index.html` dans un navigateur donne un aperçu visuel. Dans Tauri, Jouer utilise la commande native existante `open_game`. Les demandes simultanées sont bloquées et le bouton redevient disponible après un succès ou une erreur.

## Développement et compilation

Prérequis Windows : Node.js, Rust stable MSVC, Microsoft C++ Build Tools avec Windows SDK et WebView2. Voir les [prérequis officiels Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm test
npm run tauri -- dev
npm run tauri -- build -- --locked
```

L’exécutable se trouve dans `src-tauri/target/release/orbit-shadow-launcher.exe` et l’installateur NSIS dans `src-tauri/target/release/bundle/nsis/`. Le dossier `src-tauri/resources/chromium` est inclus par la configuration de distribution existante.

Le workflow `Build Orbit Shadow for Windows` compile cette branche de refonte et conserve l’exécutable et l’installateur dans les artefacts GitHub Actions. Il peut aussi être lancé manuellement une fois présent sur la branche par défaut. Il ne publie pas de release.

## Vérifications

`npm test` vérifie le mode aperçu, le contrat `open_game`, l’absence de doubles lancements pendant une demande, la remise en état après erreur et les préférences d’animation. Ces tests simulent l’API native ; ils ne se connectent pas au jeu.

Le fond original `src/assets/orbit-eclipse.png` a été créé pour cette interface avec ImageGen. Son prompt est conservé dans `design/IMAGE-PROMPT.txt`.
