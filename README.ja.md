<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**Windows 10 & 11 向けにピュア Rust で構築された、超高速・モダン・完全オフラインの画像ビューアー。**

<br>

### 🌐 言語を選択 / Select Language / Selecione o Idioma / 选择语言:

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
  <a href="https://github.com/Tsujiu/visor/releases/latest"><img src="https://img.shields.io/github/v/release/Tsujiu/visor?style=for-the-badge&logo=github&logoColor=white&color=0078D4&label=%E3%83%AA%E3%83%AA%E3%83%BC%E3%82%B9" alt="最新リリース"></a>
  <a href="https://github.com/Tsujiu/visor/actions/workflows/release-distribution.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tsujiu/visor/release-distribution.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=Build" alt="CI ビルド状態"></a>
  <img src="https://img.shields.io/badge/Windows-10_%7C_11_(x64)-0078D4?style=for-the-badge" alt="Windows 10 & 11 x64">
  <a href="LICENSE"><img src="https://img.shields.io/badge/%E3%83%A9%E3%82%A4%E3%82%BB%E3%83%B3%E3%82%B9-PolyForm_Noncommercial-F59E0B?style=for-the-badge" alt="PolyForm Noncommercial 1.0.0"></a>
  <br>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 100%"></a>
  <img src="https://img.shields.io/badge/フォーマット-75+-2ea44f?style=for-the-badge" alt="75+ Formats">
  <img src="https://img.shields.io/badge/プライバシー-100%25_オフライン-blueviolet?style=for-the-badge" alt="100% Offline">
  <img src="https://img.shields.io/badge/テレメトリ-ゼロ-success?style=for-the-badge" alt="Zero Telemetry">
</p>

<p align="center">
  <a href="#-ダウンロード--インストール"><strong>VISOR v1.0.1 をダウンロード</strong></a>
  &nbsp;•&nbsp;
  <a href="#-visor-を選ぶ理由">機能</a>
  &nbsp;•&nbsp;
  <a href="#-対応フォーマット-75">フォーマット</a>
  &nbsp;•&nbsp;
  <a href="#-ショートカットキー">ショートカット</a>
  &nbsp;•&nbsp;
  <a href="#-開発を支援する--buy-me-a-coffee"><strong>開発を支援 ☕</strong></a>
</p>

</div>

---

## 🖼️ スクリーンショット

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="Windows 11 での VISOR メイン画面">
</p>

<p align="center">
  <em>Windows 11 Fluent Design スタイル、適応型ツールバー、非同期サムネイルフィルムストリップ。</em>
</p>

---

## ⚡ VISOR を選ぶ理由

Windows 標準のフォトアプリは動作が重く、不要な機能やテレメトリが増加しています。**VISOR** は、GPU ハードウェアアクセラレーションを活用し、**純粋なネイティブ Rust** でゼロから構築されました：

* 🚀 **超高速起動:** 大容量ファイルも数ミリ秒で開き、フリーズしません。
* 🛡️ **確実なプライバシー:** **100% オフライン**で動作。写真、サムネイル、メタデータが外部に送信されることは一切ありません。
* 🍃 **極めて低いアイドル時負荷:** 待機時のメモリ消費は 20〜40 MB、CPU 使用率は 0%。
* 🎯 **デフォルト 100% (1:1) 表示:** 常に原寸の鮮明さで即時表示。カーソル位置を中心に 1% から 6400% まで無段階スムーズズーム。
* 🔍 **Pixel-Perfect モード:** ドット絵やゲームテクスチャの確認に最適な Nearest Neighbor ニアレストネイバー補間。
* 🎞️ **フィルムストリップ (Filmstrip):** 画面下部で高速サムネイル一覧、バックグラウンド非同期レンダリング。
* 📊 **EXIF & ICC 詳細パネル:** カメラ機種、レンズ、F値、ISO、シャッタースピード、ICC プロファイル、GPS 座標の解析。
* 🔄 **一括コンバーター (Batch Export):** フォルダ内の画像を一括変換、最大 16 ビット深度対応。
* 🌐 **多言語対応:** 日本語、英語、ポルトガル語、スペイン語、フランス語、ドイツ語、イタリア語、ロシア語、簡体字中国語。

---

## 📸 視覚的デモ

| サムネイル・フィルムストリップ (Filmstrip) | 技術情報パネル (EXIF) & メニュー |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="フィルムストリップナビゲーション"> | <img src="assets/screenshots/info_panel.png" width="460" alt="EXIF情報パネルと設定メニュー"> |
| *非同期プリロードによる滑らかな画像切り替え* | *詳細な撮影・技術メタデータ解析と補間モード切り替え* |

---

## 📦 対応フォーマット (75+)

* **写真・高精度ラスター:** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld, JPEG 2000 (JP2, J2K), Netpbm, SGI, Sun RAS, WBMP, XBM.
* **ベクター・CAD・デザイン:** CorelDRAW (.cdr v3〜2024+ 高解像度プレビュー抽出), SVG / SVGZ, AutoCAD DXF, Windows EMF / WMF (GDI+), PATH 上の ImageMagick による PDF, EPS, PS, AI.
* **アニメーション:** GIF, APNG, Windows アニメーションカーソル (ANI), MNG, Lottie ベクターアニメーション (.lottie, .json), Telegram アニメーションステッカー (.tgs).
* **カメラ RAW:** Canon (CR2, CRW, CR3), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ, CAP), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Adobe DNG.
* **ゲームテクスチャ:** DirectDraw Surface (DDS BC1〜BC7 & Mipmaps), Khronos KTX 1 / KTX 2, Valve VTF, Blizzard BLP, Half-Life Sprites (.spr), RenderWare TXD, ASTC, Basis Universal, PowerVR PVR, Klei KTEX.

---

## ⌨️ ショートカットキー

| キー | 動作 |
| :--- | :--- |
| `Ctrl` + `O` | 画像を開く |
| `←` / `→` | 前 / 次の画像 |
| `Home` / `End` | フォルダ内の最初 / 最後の画像 |
| `1` | ズーム 100% (原寸 1:1) |
| `F` | ウィンドウに合わせる / 原寸表示の切り替え |
| `P` | 補間モード切り替え (Pixel Perfect / 滑らか / 自動) |
| `R` / `Shift` + `R` | 右 / 左に 90° 回転 |
| `+` / `−` | 拡大 / 縮小 |
| `マウスホイール` | カーソル位置でズーム (または画像送り) |
| `T` | フィルムストリップの表示 / 非表示 |
| `I` | EXIF パネルの表示 / 非表示 |
| `B` | 背景色切り替え (黒/白チェッカー、黒、白、カスタム) |
| `Space` | アニメーションの再生 / 一時停止 |
| `Ctrl` + `C` | 透過付きでクリップボードにコピー |
| `Delete` | ごみ箱へ安全に移動 |
| `Ctrl` + `,` | 設定画面を開く |
| `F11` | フルスクリーン切り替え |

---

## 📥 ダウンロード & インストール

### 方法 1: 公式インストーラー (.exe) またはポータブル版 (.zip)
[Releases](https://github.com/Tsujiu/visor/releases/latest) から最新版をダウンロードしてください。

### 方法 2: ソースコードからビルド
```bash
git clone https://github.com/Tsujiu/visor.git
cd visor
cargo build --release
# 生成されるバイナリ: target/release/visor.exe
```

---

## ☕ 開発を支援する / Buy Me a Coffee

**VISOR** は広告なし・テレメトリなしの独立系フリーオープンソースプロジェクトです。日頃の作業に役立ちましたら、ぜひご支援ください：

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_Global-任意の金額で支援_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="Stripe 支援リンク"></a>
</p>

* **🌍 国際支援 (Stripe - 任意の金額を入力可能):** [Stripe 寄付リンク (\$1, \$5, \$10, \$20, \$50, \$100 など)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)

---

## 📄 ライセンス

**PolyForm Noncommercial License 1.0.0**（個人利用および非商用利用）の下で配布されています。詳細は [LICENSE](LICENSE) をご覧ください。

---

<p align="center">
  開発: <strong>Tsujiu</strong> • Rust で ❤️ を込めて作成
</p>
