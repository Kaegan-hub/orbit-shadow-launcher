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

## Mise à jour automatique

Le launcher vérifie `https://play.orbit-shadow.cloud/launcher/latest.json` à chaque
démarrage (`tauri-plugin-updater`) et installe silencieusement toute version plus
récente et correctement signée, puis relance l'app.

**Limitation connue : Windows SmartScreen.** L'installeur téléchargé n'a pas de
signature Authenticode (seulement la signature minisign du plugin, qui protège
l'intégrité de la mise à jour mais n'est pas reconnue par Windows). SmartScreen
peut donc afficher "Windows a protégé votre PC" et bloquer l'installation tant
que l'utilisateur n'a pas cliqué "Informations complémentaires → Exécuter quand
même". Décision prise (2026-09-13) : ne rien faire pour l'instant -- la
réputation SmartScreen du fichier s'améliore avec le nombre d'exécutions sans
incident. Un certificat de signature de code (Authenticode, payant) supprimerait
l'avertissement dès la première installation si ça devient un problème réel.

### Publier une nouvelle version

1. Bumper la version dans `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` et
   `package.json` (les trois doivent correspondre).
2. Compiler en signant avec la clé privée (jamais commitée, voir
   `src-tauri/signing.key` -- gitignored) :
   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content src-tauri\signing.key -Raw
   $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<mot de passe de la clé>"
   npm run tauri build
   ```
3. Copier `src-tauri/target/release/bundle/nsis/Orbit Shadow_<version>_x64-setup.exe`
   vers `C:\OrbitShadow\sites\cms-next\launcher\` sur le VPS (remplace l'ancien).
4. Mettre à jour `C:\OrbitShadow\sites\cms-next\launcher\latest.json` : nouvelle
   `version`, nouvelle `url`, et le contenu du fichier `.sig` généré à côté de
   l'installeur (dans le champ `signature`).
