<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**La visionneuse d'images ultra-rapide, moderne et 100% respectueuse de votre vie privée pour Windows 10 & 11 en Rust natif.**

<br>

### 🌐 Choisissez votre langue / Select Language / Selecione o Idioma / 选择语言:

[![Português](https://img.shields.io/badge/Português_(BR)-009c3b?style=for-the-badge&logo=flag&logoColor=white)](README.md)
[![English](https://img.shields.io/badge/English-00247d?style=for-the-badge&logo=flag&logoColor=white)](README.en.md)
[![Español](https://img.shields.io/badge/Español-aa151b?style=for-the-badge&logo=flag&logoColor=white)](README.es.md)
[![简体中文](https://img.shields.io/badge/简体中文-de2910?style=for-the-badge&logo=flag&logoColor=white)](README.zh-CN.md)
[![Русский](https://img.shields.io/badge/Русский-1c3578?style=for-the-badge&logo=flag&logoColor=white)](README.ru.md)
[![Deutsch](https://img.shields.io/badge/Deutsch-000000?style=for-the-badge&logo=flag&logoColor=white)](README.de.md)
[![日本語](https://img.shields.io/badge/日本語-bc002d?style=for-the-badge&logo=flag&logoColor=white)](README.ja.md)
[![Polski](https://img.shields.io/badge/Polski-dc143c?style=for-the-badge&logo=flag&logoColor=white)](README.pl.md)
[![Français](https://img.shields.io/badge/Français-0055a5?style=for-the-badge&logo=flag&logoColor=white)](README.fr.md)
[![Italiano](https://img.shields.io/badge/Italiano-009246?style=for-the-badge&logo=flag&logoColor=white)](README.it.md)

<br>

<p align="center">
  <a href="https://github.com/Tsujiu/visor/releases/latest"><img src="https://img.shields.io/github/v/release/Tsujiu/visor?style=for-the-badge&logo=github&logoColor=white&color=0078D4&label=Version" alt="Dernière Version"></a>
  <a href="https://github.com/Tsujiu/visor/actions/workflows/release-distribution.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tsujiu/visor/release-distribution.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=Build" alt="Statut Build CI"></a>
  <img src="https://img.shields.io/badge/Windows-10_%7C_11_(x64)-0078D4?style=for-the-badge" alt="Windows 10 & 11 x64">
  <a href="LICENSE"><img src="https://img.shields.io/badge/Licence-PolyForm_Noncommercial-F59E0B?style=for-the-badge" alt="PolyForm Noncommercial 1.0.0"></a>
  <br>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="100% Rust"></a>
  <img src="https://img.shields.io/badge/Formats-75+-2ea44f?style=for-the-badge" alt="75+ Formats">
  <img src="https://img.shields.io/badge/Confidentialit%C3%A9-100%25_Hors_ligne-blueviolet?style=for-the-badge" alt="100% Hors ligne">
  <img src="https://img.shields.io/badge/T%C3%A9l%C3%A9m%C3%A9trie-Z%C3%A9ro-success?style=for-the-badge" alt="Zéro télémétrie">
</p>

<p align="center">
  <a href="#-t%C3%A9l%C3%A9chargement--installation"><strong>Télécharger VISOR v1.0.2</strong></a>
  &nbsp;•&nbsp;
  <a href="#-pourquoi-visor">Fonctionnalités</a>
  &nbsp;•&nbsp;
  <a href="#-formats-pris-en-charge-75">Formats</a>
  &nbsp;•&nbsp;
  <a href="#-raccourcis-clavier">Raccourcis</a>
  &nbsp;•&nbsp;
  <a href="#-soutenir-le-projet--buy-me-a-coffee"><strong>Faire un don ☕</strong></a>
</p>

</div>

---

## 🖼️ Captures d'écran

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="Interface principale de VISOR sous Windows 11">
</p>

<p align="center">
  <em>Interface fluide au style Fluent Design, barre d'outils adaptative et pellicule de vignettes asynchrone en bas d'écran.</em>
</p>

---

## ⚡ Pourquoi VISOR ?

Les visionneuses intégrées de Windows sont devenues lentes, lourdes et truffées de télémétrie. **VISOR** a été conçu à partir de zéro en **Rust natif** avec accélération matérielle GPU :

* 🚀 **Démarrage instantané :** Ouvre des fichiers lourds en quelques millisecondes sans aucun freeze.
* 🛡️ **Confidentialité totale :** Fonctionne **100% hors ligne**. Aucune photo, vignette ou métadonnée ne quitte jamais votre ordinateur.
* 🍃 **Empreinte mémoire minime :** Seulement 20 à 40 Mo de RAM et 0% de CPU au repos.
* 🎯 **Zoom 100% (1:1) par défaut :** Affiche chaque pixel à sa résolution d'origine immédiatement, avec un zoom fluide de 1% à 6400% centré sur le curseur.
* 🔍 **Mode Pixel-Perfect :** Échantillonnage Nearest Neighbor ultra-net sans flou pour le pixel art et les textures de jeux vidéo.
* 🎞️ **Pellicule de vignettes (Filmstrip) :** Navigation rapide avec pré-rendu en arrière-plan décorrélé.
* 📊 **Panneau technique EXIF et ICC :** Analyse poussée des données de prise de vue (Boîtier, objectif, ouverture, ISO, vitesse, profil ICC et GPS).
* 🔄 **Convertisseur par lots intégré :** Conversion de dossiers entiers avec préservation jusqu'à 16 bits de profondeur.
* 🌐 **Interface en 9 langues :** Français, Anglais, Espagnol, Portugais, Allemand, Italien, Russe, Chinois simplifié et Japonais.

---

## 📸 Démonstration Visuelle

| Bandeau de Miniatures (Filmstrip) | Panneau Technique EXIF & Menus |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="Navigation par bandeau de miniatures"> | <img src="assets/screenshots/info_panel.png" width="460" alt="Panneau technique EXIF et menus d'options"> |
| *Navigation fluide et asynchrone en arrière-plan* | *Inspection complète des métadonnées EXIF, modes d'interpolation et réglages* |

---

## 📦 Formats pris en charge (75+)

* **Photographie et Raster :** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld, JPEG 2000 (JP2, J2K), Netpbm, SGI, Sun RAS, WBMP, XBM.
* **Vectoriel, CAO & Graphisme :** CorelDRAW (.cdr v3 à 2024+ avec extraction d'aperçu haute résolution), SVG et SVGZ, AutoCAD DXF, Windows EMF et WMF (GDI+), ainsi que PDF, EPS, PS, AI via ImageMagick dans le PATH.
* **Animations :** GIF, APNG, curseurs animés Windows (ANI), MNG, animations vectorielles Lottie (.lottie, .json) et stickers Telegram (.tgs).
* **Négatifs numériques RAW :** Canon (CR2, CRW, CR3), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ, CAP), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Adobe DNG.
* **Textures de jeux :** DirectDraw Surface (DDS BC1 à BC7 et Mipmaps), Khronos KTX 1 et 2, Valve VTF, Blizzard BLP, Half-Life Sprites (.spr), RenderWare TXD, ASTC, Basis Universal, PowerVR PVR, Klei KTEX.

---

## ⌨️ Raccourcis clavier

| Raccourci | Action |
| :--- | :--- |
| `Ctrl` + `O` | Ouvrir un fichier image |
| `←` / `→` | Image précédente / suivante |
| `Début` / `Fin` | Première / dernière image du dossier |
| `1` | Zoom 100% (Taille réelle 1:1) |
| `F` | Alterner entre Adapter à la fenêtre et Taille réelle |
| `P` | Alterner le mode d'interpolation (Pixel-Perfect / Lisse / Auto) |
| `R` / `Maj` + `R` | Rotation de 90° sens horaire / anti-horaire |
| `+` / `−` | Zoom avant / Zoom arrière |
| `Molette souris`| Zoom centré sur le curseur (ou navigation) |
| `T` | Afficher / masquer la pellicule inférieure |
| `I` | Afficher / masquer le panneau EXIF |
| `B` | Changer le fond du canevas (Damier sombre/clair, noir, blanc, couleur personnalisée) |
| `Espace` | Lecture / pause des animations |
| `Ctrl` + `C` | Copier dans le presse-papiers avec transparence |
| `Suppr` | Déplacer en sécurité dans la Corbeille Windows |
| `Ctrl` + `,` | Ouvrir les Paramètres |
| `F11` | Plein écran |

---

## 📥 Téléchargement & Installation

### Option 1 : Installateur officiel (.exe) ou Archive portable (.zip)
Téléchargez la dernière mouture sur la page [Releases](https://github.com/Tsujiu/visor/releases/latest).

### Option 2 : Compiler depuis les sources
```bash
git clone https://github.com/Tsujiu/visor.git
cd visor
cargo build --release
# Le binaire se trouve dans : target/release/visor.exe
```

---

## ☕ Soutenir le projet / Buy Me a Coffee

**VISOR** est un logiciel libre, gratuit et indépendant, garanti sans publicité ni suivi de données. Si VISOR vous est utile au quotidien, vous pouvez faire un don pour soutenir son développement continu :

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_Global-Montant_libre_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Faire un don via Stripe"></a>
  &nbsp;&nbsp;
  <a href="https://linknabio.gg/mutsujii"><img src="https://img.shields.io/badge/PIX_&_Carte-Offrir_un_café_☕-00C9A7?style=for-the-badge&logo=pix&logoColor=white" alt="InfinitePay Brésil"></a>
</p>

* **🌍 Donateurs internationaux (Stripe - Montant au choix) :** [Lien de don Stripe (Choisissez \$1, \$5, \$10, \$20, \$50 ou votre montant)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)

---

## 📄 Licence

Distribué sous la licence **PolyForm Noncommercial License 1.0.0** (Usage personnel et non commercial). Consultez le fichier [LICENSE](LICENSE) pour plus de détails.

---

<p align="center">
  Conçu avec passion par <strong>Tsujiu</strong> • Développé avec ❤️ en Rust
</p>
