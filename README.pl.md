<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**Ultraszybka, nowoczesna i w 100% prywatna przeglądarka obrazów dla systemu Windows 10 i 11 w natywnym Rust.**

<br>

### 🌐 Wybierz język / Select Language / Selecione o Idioma / 选择语言:

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
  <a href="https://github.com/Tsujiu/visor/releases/latest"><img src="https://img.shields.io/github/v/release/Tsujiu/visor?style=for-the-badge&logo=github&logoColor=white&color=0078D4&label=Wydanie" alt="Najnowsze Wydanie"></a>
  <a href="https://github.com/Tsujiu/visor/actions/workflows/release-distribution.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tsujiu/visor/release-distribution.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=Build" alt="Status CI"></a>
  <img src="https://img.shields.io/badge/Windows-10_%7C_11_(x64)-0078D4?style=for-the-badge" alt="Windows 10 & 11 x64">
  <a href="LICENSE"><img src="https://img.shields.io/badge/Licencja-PolyForm_Noncommercial-F59E0B?style=for-the-badge" alt="PolyForm Noncommercial 1.0.0"></a>
  <br>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="100% Rust"></a>
  <img src="https://img.shields.io/badge/Formaty-75+-2ea44f?style=for-the-badge" alt="75+ Formatów">
  <img src="https://img.shields.io/badge/Prywatno%C5%9B%C4%87-100%25_Offline-blueviolet?style=for-the-badge" alt="100% Offline">
  <img src="https://img.shields.io/badge/Telemetria-Zero-success?style=for-the-badge" alt="Zero Telemetrii">
</p>

<p align="center">
  <a href="#-pobieranie-i-instalacja"><strong>Pobierz VISOR v1.0.2</strong></a>
  &nbsp;•&nbsp;
  <a href="#-dlaczego-visor">Funkcje</a>
  &nbsp;•&nbsp;
  <a href="#-obs%C5%82ugiwane-formaty-75">Formaty</a>
  &nbsp;•&nbsp;
  <a href="#-skr%C3%B3ty-klawiszowe">Skróty</a>
  &nbsp;•&nbsp;
  <a href="#-wesprzyj-rozw%C3%B3j--buy-me-a-coffee"><strong>Wesprzyj Projekt ☕</strong></a>
</p>

</div>

---

## 🖼️ Zrzuty Ekranu

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="Główny interfejs VISOR w Windows 11">
</p>

<p align="center">
  <em>Płynny interfejs Fluent Design, adaptacyjny pasek narzędzi i asynchroniczny pasek miniatur u dołu ekranu.</em>
</p>

---

## ⚡ Dlaczego VISOR?

Domyślne przeglądarki zdjęć w systemie Windows stały się powolne i przeładowane telemetrią. **VISOR** powstał od zera w **natywnym języku Rust** z akceleracją sprzętową GPU:

* 🚀 **Błyskawiczne Uruchamianie:** Otwiera duże pliki w milisekundach bez zawieszania okna.
* 🛡️ **Pełna Prywatność:** Działa **w 100% offline**. Żadne zdjęcia, miniatury ani metadane nigdy nie opuszczają Twojego komputera.
* 🍃 **Znikome Zużycie Zasobów:** 20 do 40 MB pamięci RAM i 0% użycia procesora w stanie spoczynku.
* 🎯 **Domyślne Powiększenie 100% (1:1):** Wyświetla każdy piksel w natywnej rozdzielczości od razu, z płynnym zoomem od 1% do 6400% zakotwiczonym na kursorze.
* 🔍 **Tryb Pixel-Perfect:** Skalowanie Nearest Neighbor bez rozmyć dla pixel artu i tekstur gier.
* 🎞️ **Pasek Miniatur (Filmstrip):** Szybkie przeglądanie katalogu z asynchronicznym renderowaniem w tle.
* 📊 **Panel Techniczny EXIF i ICC:** Dokładny odczyt parametrów aparatu, obiektywu, przysłony, ISO, czasu naświetlania, profilu ICC i GPS.
* 🔄 **Zintegrowany Konwerter Masowy:** Konwertuj całe foldery z zachowaniem do 16 bitów głębi kolorów.
* 🌐 **Wielojęzyczny Interfejs:** Polski, Angielski, Niemiecki, Francuski, Hiszpański, Włoski, Portugalski, Rosyjski, Chiński i Japoński.

---

## 📸 Prezentacja Wizualna

| Pasek Miniatur (Filmstrip) | Panel Techniczny EXIF i Menu |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="Nawigacja po pasku miniatur"> | <img src="assets/screenshots/info_panel.png" width="460" alt="Panel techniczny EXIF i menu opcji"> |
| *Płynna nawigacja z asynchronicznym ładowaniem w tle* | *Pełny odczyt parametrów EXIF, tryby interpolacji i konfiguracja* |

---

## 📦 Obsługiwane Formaty (75+)

* **Fotografia i Grafika Rastrowa:** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld, JPEG 2000 (JP2, J2K), Netpbm, SGI, Sun RAS, WBMP, XBM.
* **Wektory, CAD i DTP:** CorelDRAW (.cdr v3 do 2024+ z ekstrakcją podglądu wysokiej rozdzielczości), SVG i SVGZ, AutoCAD DXF, Windows EMF i WMF (GDI+), a także PDF, EPS, PS, AI przez ImageMagick w PATH.
* **Animacje:** GIF, APNG, animowane kursory Windows (ANI), MNG, animacje wektorowe Lottie (.lottie, .json) oraz naklejki Telegram (.tgs).
* **Formaty Cyfrowych Negatywów RAW:** Canon (CR2, CRW, CR3), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ, CAP), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Adobe DNG.
* **Tekstury z Gier:** DirectDraw Surface (DDS BC1 do BC7 i Mipmaps), Khronos KTX 1 / KTX 2, Valve VTF, Blizzard BLP, Half-Life Sprites (.spr), RenderWare TXD, ASTC, Basis Universal, PowerVR PVR, Klei KTEX.

---

## ⌨️ Skróty Klawiszowe

| Klawisz | Działanie |
| :--- | :--- |
| `Ctrl` + `O` | Otwórz plik obrazu |
| `←` / `→` | Poprzedni / następny obraz w folderze |
| `Home` / `End` | Pierwszy / ostatni obraz |
| `1` | Zoom 100% (Rzeczywisty rozmiar 1:1) |
| `F` | Przełącz dopasowanie do okna / 1:1 |
| `P` | Przełącz tryb próbkowania (Pixel Perfect / Gładki / Auto) |
| `R` / `Shift` + `R` | Obróć o 90° w prawo / w lewo |
| `+` / `−` | Powiększ / pomniejsz |
| `Kółko myszy` | Zoom w punkcie kursora (lub nawigacja) |
| `T` | Pokaż / ukryj dolny pasek miniatur |
| `I` | Pokaż / ukryj panel metadanych EXIF |
| `B` | Zmiana tła (Ciemna/jasna szachownica, czerń, biel, własny kolor) |
| `Spacja` | Odtwórz / wstrzymaj animację |
| `Ctrl` + `C` | Kopiuj obraz do schowka z przezroczystością |
| `Delete` | Bezpiecznie przenieś do Kosza Windows |
| `Ctrl` + `,` | Otwórz Ustawienia |
| `F11` | Tryb pełnoekranowy |

---

## 📥 Pobieranie i Instalacja

### Opcja 1: Oficjalny Instalator (.exe) lub Wersja Przenośna (.zip)
Pobierz najnowsze wydanie na stronie [Releases](https://github.com/Tsujiu/visor/releases/latest).

### Opcja 2: Kompilacja ze Źródeł
```bash
git clone https://github.com/Tsujiu/visor.git
cd visor
cargo build --release
# Gotowy plik wykonywalny znajduje się w: target/release/visor.exe
```

---

## ☕ Wesprzyj Rozwój / Buy Me a Coffee

**VISOR** jest niezależnym i w 100% darmowym projektem open-source, wolnym od reklam i śledzenia. Jeśli VISOR ułatwia Twoją codzienną pracę, możesz dobrowolnie wesprzeć projekt:

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_Global-Dowolna_kwota_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Wesprzyj dowolną kwotą przez Stripe"></a>
  &nbsp;&nbsp;
  <a href="https://linknabio.gg/mutsujii"><img src="https://img.shields.io/badge/PIX_&_Karta-Postaw_kawę_☕-00C9A7?style=for-the-badge&logo=pix&logoColor=white" alt="InfinitePay"></a>
</p>

* **🌍 Wpłaty Międzynarodowe (Stripe - Dowolna kwota):** [Link do płatności Stripe (\$1, \$5, \$10, \$20, \$50 lub własna kwota)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)

---

## 📄 Licencja

Dystrybuowane na licencji **PolyForm Noncommercial License 1.0.0** (Użytek osobisty i niekomercyjny). Szczegóły w pliku [LICENSE](LICENSE).

---

<p align="center">
  Stworzone z pasją przez <strong>Tsujiu</strong> • Zbudowane z ❤️ w Rust
</p>
