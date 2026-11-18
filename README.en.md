<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**The ultra-fast, modern, and privacy-first image viewer engineered for Windows 10 & 11 in native Rust.**

<br>

### 🌐 Select Language / Selecione o Idioma / 选择语言 / Выберите язык:

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
  <a href="https://github.com/Tsujiu/visor/releases/latest"><img src="https://img.shields.io/badge/Release-v1.0.0-0078D4?style=for-the-badge&logo=windows11&logoColor=white" alt="Release v1.0.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 100%"></a>
  <img src="https://img.shields.io/badge/Formats-75+-2ea44f?style=for-the-badge" alt="75+ Formats">
  <img src="https://img.shields.io/badge/Privacy-100%25_Offline-blueviolet?style=for-the-badge" alt="100% Offline">
  <img src="https://img.shields.io/badge/Telemetry-Zero-success?style=for-the-badge" alt="Zero Telemetry">
</p>

<p align="center">
  <a href="#-download--installation"><strong>Download VISOR v1.0.0</strong></a>
  &nbsp;•&nbsp;
  <a href="#-why-visor">Features</a>
  &nbsp;•&nbsp;
  <a href="#-supported-formats-75">Formats</a>
  &nbsp;•&nbsp;
  <a href="#-keyboard-shortcuts">Shortcuts</a>
  &nbsp;•&nbsp;
  <a href="#-support-development--buy-me-a-coffee"><strong>Support Development ☕</strong></a>
</p>

</div>

---

## 🖼️ Screenshots

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="VISOR Main Interface on Windows 11">
</p>

<p align="center">
  <em>Fluid Fluent Design interface, responsive adaptive toolbar, and high-fidelity asynchronous bottom thumbnail filmstrip.</em>
</p>

---

## ⚡ Why VISOR?

Default Windows image viewers have grown sluggish, bloated, and cluttered with telemetry and unwanted features. **VISOR** was built from scratch in **pure native Rust** with GPU hardware acceleration to deliver:

* 🚀 **Instant Startup:** Opens massive files in milliseconds without freezing.
* 🛡️ **Absolute Privacy:** Operates **100% offline**. No photos, thumbnails, metadata, or browsing history ever leaves your machine.
* 🍃 **Near-Zero Idle Consumption:** 20 to 40 MB RAM footprint and 0% CPU usage when idle.
* 🎯 **Default 100% Zoom (1:1):** Displays every pixel at native resolution immediately, with smooth continuous zoom from 1% to 6400% anchored to the cursor.
* 🔍 **Dynamic Pixel-Perfect Mode:** Crisp Nearest Neighbor sampling with zero blur for pixel art, game texture inspection, and vector assets.
* 🎞️ **Filmstrip Navigation:** Rapid bottom strip navigation with decoupled asynchronous background pre-rendering.
* 📊 **EXIF & ICC Technical Panel:** Deep inspection of photographic parameters (Camera model, Lens, f-stop, ISO, Shutter speed, ICC color profile, and GPS coordinates).
* 🔄 **Integrated Batch Converter:** Batch convert entire folders into multiple target formats while preserving up to 16 bits of bit depth.
* 🌐 **Multilingual Interface:** English, Portuguese (BR), Spanish, French, German, Italian, Russian, Simplified Chinese, and Japanese.

---

## 📸 Visual Demonstration

| Thumbnail Filmstrip | Technical EXIF Panel & Menus |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="Filmstrip Navigation"> | <img src="assets/screenshots/info_panel.png" width="460" alt="Technical EXIF Panel and Context Menus"> |
| *Smooth decoupled navigation with background preloading* | *Complete photographic metadata inspection, interpolation modes & controls* |

---

## 📊 Comparison

| Feature | VISOR | Windows Photos | IrfanView | ImageGlass |
| :--- | :---: | :---: | :---: | :---: |
| **Core Language** | **Native Rust** | C# / WinUI | C++ | C# / .NET |
| **Opening Latency** | **< 30 ms** | ~1.5 s - 3 s | ~100 ms | ~800 ms |
| **Telemetry & Tracking** | **Zero (100% Offline)** | Constant | None | Partial |
| **Modern Formats (AVIF, JXL, WebP)** | **Yes (Native)** | Limited | Via Plugins | Yes |
| **Professional RAW (23+ brands)** | **Yes (Native)** | Requires Extension | Via C Plugin | Partial |
| **Gaming Textures (DDS, KTX2, VTF)** | **Yes (Native)** | No | No | No |
| **Windows 11 Fluent Aesthetic** | **Native & Modern** | Native | 90s Style | Modern |

---

## 📦 Supported Formats (75+)

VISOR features one of the most comprehensive image decoding engines available:

* **High-Precision Photography & Raster:** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB flattened compositions), GIMP (XCF visible layers), OpenRaster (ORA), Farbfeld (16-bit), JPEG 2000 (JP2, J2K), Netpbm (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP, XBM.
* **Vector, CAD & Illustration:** CorelDRAW (.cdr v3 to 2024+ with embedded high-res preview extraction), SVG & SVGZ safe native rendering, AutoCAD DXF, Windows EMF & WMF (via GDI+), plus PDF, EPS, PS, Adobe Illustrator (AI) via ImageMagick on PATH.
* **Native Animations:** GIF, APNG, Windows Animated Cursor (ANI), MNG, Lottie Vector Animations (.lottie, .json), and Telegram Animated Stickers (.tgs).
* **Professional Camera RAW Negatives:** Canon (CR2, CRW, CR3), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ, CAP), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), and Adobe DNG.
* **Game & Engine Textures:** DirectDraw Surface (DDS with BC1 to BC7 & Mipmaps), Khronos KTX 1 & KTX 2, Valve Texture Format (VTF), Blizzard BLP1/BLP2, GoldSrc / Half-Life Sprites (.spr), RenderWare TXD (GTA III / VC / SA), ASTC, Basis Universal, PowerVR PVR, and Klei KTEX.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl` + `O` | Open image file |
| `←` / `→` | Previous / next image in directory |
| `Home` / `End` | First / last image in directory |
| `1` | 100% Zoom (Actual Size 1:1) |
| `F` | Toggle Fit to Window / Actual Size |
| `P` | Toggle interpolation mode (Pixel Perfect / Smooth / Auto) |
| `R` / `Shift` + `R` | Rotate 90° clockwise / counter-clockwise |
| `+` / `−` | Zoom in / Zoom out |
| `Mouse Wheel` | Zoom anchored to cursor (or navigate, configurable in Settings) |
| `T` | Toggle bottom thumbnail filmstrip |
| `I` | Toggle EXIF technical metadata panel |
| `B` | Cycle canvas background (Dark/Light checker, Black, White, Custom) |
| `Space` | Play / pause animations (GIF, APNG, Lottie) |
| `Ctrl` + `C` | Copy image to Windows Clipboard with alpha transparency |
| `Delete` | Safely send image to Windows Recycle Bin |
| `Ctrl` + `,` | Open Settings dialog |
| `F11` | Toggle Fullscreen mode |

---

## 📥 Download & Installation

### Option 1: Official Installer (.exe) or Portable Archive (.zip) for Windows 10 & 11
Download the latest build from [Releases](https://github.com/Tsujiu/visor/releases/latest) (conventional Inno Setup executable installer or standalone zero-install portable ZIP).

### Option 2: Build from Source
Ensure you have the **Rust** toolchain installed:
```bash
# Clone repository
git clone https://github.com/Tsujiu/visor.git
cd visor

# Build optimized release binary
cargo build --release

# The executable will be located at:
target/release/visor.exe
```

---

## ☕ Support Development / Buy Me a Coffee

**VISOR** is an independent, 100% free, and privacy-focused open-source project without ads or data collection. If VISOR is helpful for your daily workflow, you can voluntarily support its ongoing development:

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_Global-Choose_Any_Amount_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Support with custom USD amount via Stripe"></a>
  &nbsp;&nbsp;
  <a href="https://linknabio.gg/mutsujii"><img src="https://img.shields.io/badge/PIX_&_Card_(Brazil)-Buy_a_Coffee_☕-00C9A7?style=for-the-badge&logo=pix&logoColor=white" alt="Buy a Coffee via InfinitePay"></a>
  &nbsp;&nbsp;
  <a href="https://donate.stripe.com/test_5kQ4gydHc5tO6Cf5Ko3VC05"><img src="https://img.shields.io/badge/Stripe_Brazil-Choose_Amount_(BRL)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Support with BRL via Stripe"></a>
</p>

* **🌍 International Donors (Stripe - Choose any amount):** [Donate via Stripe (Choose \$1, \$5, \$10, \$20, \$50, \$100 or custom)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)
* **🇧🇷 Brazilian Supporters (PIX / InfinitePay):** [https://linknabio.gg/mutsujii](https://linknabio.gg/mutsujii)
* **🇧🇷 Card Donations (Stripe Brazil - Choose amount):** [Donate via Stripe (Choose R\$ 5, R\$ 15, R\$ 50 or custom)](https://donate.stripe.com/test_5kQ4gydHc5tO6Cf5Ko3VC05)

Your support directly funds decoder expansions, continuous GPU pipeline optimizations, and keeps VISOR 100% telemetry-free.

---

## 📄 License

Distributed under the **PolyForm Noncommercial License 1.0.0** (Personal & Noncommercial Use). See the [LICENSE](LICENSE) file for full details.

---

<p align="center">
  Crafted with dedication by <strong>Tsujiu</strong> • Built with ❤️ in Rust
</p>
