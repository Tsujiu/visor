<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**Il visualizzatore di immagini ultraveloce, moderno e al 100% privato per Windows 10 e 11 in Rust nativo.**

<br>

### 🌐 Seleziona lingua / Select Language / Selecione o Idioma / 选择语言:

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

<details>
<summary><b>🌍 Traduzione automatica della pagina con 1 clic (Google Translate)</b></summary>

| Lingua | Pulsante di traduzione rapida |
| :--- | :--- |
| 🇮🇹 **Italiano** | [⚡ Traduci automaticamente il repository in italiano](https://translate.google.com/translate?sl=auto&tl=it&u=https://github.com/Tsujiu/visor) |
| 🇺🇸 **English** | [⚡ Translate Repository to English](https://translate.google.com/translate?sl=auto&tl=en&u=https://github.com/Tsujiu/visor) |
| 🇧🇷 **Português (BR)** | [⚡ Traduzir Repositório para Português](https://translate.google.com/translate?sl=auto&tl=pt&u=https://github.com/Tsujiu/visor) |
| 🇪🇸 **Español** | [⚡ Traducir Repositorio al Español](https://translate.google.com/translate?sl=auto&tl=es&u=https://github.com/Tsujiu/visor) |
| 🇨🇳 **简体中文** | [⚡ 在浏览器中将仓库自动翻译为简体中文](https://translate.google.com/translate?sl=auto&tl=zh-CN&u=https://github.com/Tsujiu/visor) |
| 🇷🇺 **Русский** | [⚡ Открыть репозиторий с переводом на русский](https://translate.google.com/translate?sl=auto&tl=ru&u=https://github.com/Tsujiu/visor) |
| 🇩🇪 **Deutsch** | [⚡ Repository automatisch auf Deutsch übersetzen](https://translate.google.com/translate?sl=auto&tl=de&u=https://github.com/Tsujiu/visor) |
| 🇯🇵 **日本語** | [⚡ リポジトリ全体を日本語に自動翻訳](https://translate.google.com/translate?sl=auto&tl=ja&u=https://github.com/Tsujiu/visor) |
| 🇵🇱 **Polski** | [⚡ Przetłumacz całe repozytorium na język polski](https://translate.google.com/translate?sl=auto&tl=pl&u=https://github.com/Tsujiu/visor) |
| 🇫🇷 **Français** | [⚡ Traduire automatiquement le dépôt en français](https://translate.google.com/translate?sl=auto&tl=fr&u=https://github.com/Tsujiu/visor) |

</details>

<br>

<p align="center">
  <a href="https://github.com/Tsujiu/visor/releases/latest"><img src="https://img.shields.io/badge/Release-v1.0.0-0078D4?style=for-the-badge&logo=windows11&logoColor=white" alt="Release v1.0.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="100% Rust"></a>
  <img src="https://img.shields.io/badge/Formati-75+-2ea44f?style=for-the-badge" alt="75+ Formati">
  <img src="https://img.shields.io/badge/Privacy-100%25_Offline-blueviolet?style=for-the-badge" alt="100% Offline">
  <img src="https://img.shields.io/badge/Telemetria-Zero-success?style=for-the-badge" alt="Zero Telemetria">
</p>

<p align="center">
  <a href="#-download--installazione"><strong>Scarica VISOR v1.0.0</strong></a>
  &nbsp;•&nbsp;
  <a href="#-perch%C3%A9-visor">Funzionalità</a>
  &nbsp;•&nbsp;
  <a href="#-formati-supportati-75">Formati</a>
  &nbsp;•&nbsp;
  <a href="#-scorciatoie-da-tastiera">Scorciatoie</a>
  &nbsp;•&nbsp;
  <a href="#-sostieni-lo-sviluppo--buy-me-a-coffee"><strong>Sostieni il Progetto ☕</strong></a>
</p>

</div>

---

## 🖼️ Screenshot

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="Interfaccia principale di VISOR su Windows 11">
</p>

<p align="center">
  <em>Interfaccia fluida in stile Fluent Design, barra degli strumenti dinamica e pellicola di anteprime asincrona.</em>
</p>

---

## ⚡ Perché VISOR?

I visualizzatori di foto predefiniti di Windows sono diventati lenti, pesanti e carichi di telemetria invasiva. **VISOR** è stato scritto da zero in **Rust puro** con accelerazione hardware GPU:

* 🚀 **Avvio Immediato:** Apre immagini molto pesanti in pochi millisecondi senza bloccarsi.
* 🛡️ **Privacy Assoluta:** Funziona **al 100% offline**. Nessuna foto, anteprima o metadato lascia mai il tuo PC.
* 🍃 **Zero Spreco di Risorse:** Da 20 a 40 MB di memoria RAM e 0% di uso CPU quando inattivo.
* 🎯 **Zoom 100% (1:1) Predefinito:** Visualizza ogni pixel alla sua risoluzione nativa, con zoom fluido dall'1% al 6400% ancorato al cursore.
* 🔍 **Modalità Pixel-Perfect:** Campionamento Nearest Neighbor nitido per pixel art e texture di videogiochi.
* 🎞️ **Pellicola di Anteprime (Filmstrip):** Navigazione rapida nella cartella con precaricamento asincrono.
* 📊 **Pannello Tecnico EXIF e ICC:** Lettura precisa di fotocamera, obiettivo, apertura, ISO, tempi, profilo colore ICC e GPS.
* 🔄 **Convertitore in Batch Integrato:** Conversione di intere cartelle mantenendo fino a 16 bit di profondità.
* 🌐 **Interfaccia in 9 Lingue:** Italiano, Inglese, Spagnolo, Portoghese, Francese, Tedesco, Russo, Cinese Semplificato e Giapponese.

---

## 📸 Dimostrazione Visiva

| Pellicola di Anteprime (Filmstrip) | Pannello Tecnico EXIF e Menu |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="Navigazione Filmstrip"> | <img src="assets/screenshots/info_panel.png" width="460" alt="Pannello Tecnico EXIF e Menu di Configurazione"> |
| *Navigazione fluida con precaricamento asincrono* | *Ispezione approfondita dei metadati EXIF, interpolazione e controlli* |

---

## 📦 Formati Supportati (75+)

* **Fotografia e Raster:** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld, JPEG 2000 (JP2, J2K), Netpbm, SGI, Sun RAS, WBMP, XBM.
* **Vettoriale, CAD & Grafica:** CorelDRAW (.cdr v3 fino a 2024+ con estrazione anteprima), SVG e SVGZ, AutoCAD DXF, Windows EMF e WMF (GDI+), oltre a PDF, EPS, PS, AI tramite ImageMagick nel PATH.
* **Animazioni:** GIF, APNG, cursori animati Windows (ANI), MNG, animazioni vettoriali Lottie (.lottie, .json) e sticker Telegram (.tgs).
* **Negativi Digitali RAW:** Canon (CR2, CRW, CR3), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ, CAP), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Adobe DNG.
* **Texture di Giochi:** DirectDraw Surface (DDS con BC1-BC7 e Mipmaps), Khronos KTX 1 e 2, Valve VTF, Blizzard BLP, Half-Life Sprites (.spr), RenderWare TXD, ASTC, Basis Universal, PowerVR PVR, Klei KTEX.

---

## ⌨️ Scorciatoie da Tastiera

| Tasto | Azione |
| :--- | :--- |
| `Ctrl` + `O` | Apri file immagine |
| `←` / `→` | Immagine precedente / successiva nella cartella |
| `Home` / `End` | Prima / ultima immagine nella cartella |
| `1` | Zoom 100% (Dimensioni reali 1:1) |
| `F` | Alterna tra Adatta alla Finestra e 1:1 |
| `P` | Alterna modalità di campionamento (Pixel Perfect / Smussato / Auto) |
| `R` / `Shift` + `R` | Ruota di 90° in senso orario / antiorario |
| `+` / `−` | Ingrandisci / Riduci |
| `Rotellina Mouse`| Zoom centrato sul puntatore (o navigazione) |
| `T` | Mostra / nascondi pellicola delle miniature |
| `I` | Mostra / nascondi pannello metadati EXIF |
| `B` | Alterna sfondo (Scacchiera scura/chiara, nero, bianco, personalizzato) |
| `Spazio` | Riproduci / metti in pausa animazioni |
| `Ctrl` + `C` | Copia immagine negli appunti con trasparenza |
| `Canc` | Sposta in sicurezza nel Cestino di Windows |
| `Ctrl` + `,` | Apri Impostazioni |
| `F11` | Schermo Intero |

---

## 📥 Download & Installazione

### Opzione 1: Installatore Ufficiale (.exe) o Versione Portatile (.zip)
Scarica l'ultima versione dalla pagina [Releases](https://github.com/Tsujiu/visor/releases/latest).

### Opzione 2: Compilazione dai sorgenti
```bash
git clone https://github.com/Tsujiu/visor.git
cd visor
cargo build --release
# Il file eseguibile si trova in: target/release/visor.exe
```

---

## ☕ Sostieni lo Sviluppo / Buy Me a Coffee

**VISOR** è un software libero e indipendente, gratuito e privo di annunci pubblicitari o tracciamento. Se VISOR ti è utile, puoi fare una donazione volontaria per sostenere lo sviluppo continuo:

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_Global-Importo_libero_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Dona con Stripe"></a>
  &nbsp;&nbsp;
  <a href="https://linknabio.gg/mutsujii"><img src="https://img.shields.io/badge/PIX_&_Carta-Offri_un_caffè_☕-00C9A7?style=for-the-badge&logo=pix&logoColor=white" alt="InfinitePay Brasile"></a>
</p>

* **🌍 Donazioni Internazionali (Stripe - Importo a scelta):** [Link donazioni Stripe (\$1, \$5, \$10, \$20, \$50 o personalizzato)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)

---

## 📄 Licenza

Distribuito con licenza open source libera per uso personale e professionale. Vedi [LICENSE](LICENSE).

---

<p align="center">
  Creato con passione da <strong>Tsujiu</strong> • Sviluppato con ❤️ in Rust
</p>
