<div align="center">

<p align="center">
  <img src="assets/moldura.png" width="128" height="128" alt="VISOR Logo">
</p>

# VISOR

**专为 Windows 10 与 11 设计的极速、现代、完全离线且保护隐私的原生 Rust 图像查看器。**

<br>

### 🌐 选择语言 / Select Language / Selecione o Idioma / Выберите язык:

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
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Pure_Native-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="100% Rust"></a>
  <img src="https://img.shields.io/badge/%E6%A0%BC%E5%BC%8F-75+-2ea44f?style=for-the-badge" alt="75+ 种格式">
  <img src="https://img.shields.io/badge/%E9%9A%90%E7%A7%81-100%25_%E7%A6%BB%E7%BA%BF-blueviolet?style=for-the-badge" alt="100% 离线">
  <img src="https://img.shields.io/badge/%E9%81%A5%E6%B5%8B-%E9%9B%B6-success?style=for-the-badge" alt="零遥测">
</p>

<p align="center">
  <a href="#-下载与安装"><strong>下载 VISOR v1.0.0</strong></a>
  &nbsp;•&nbsp;
  <a href="#-为什么选择-visor">核心特性</a>
  &nbsp;•&nbsp;
  <a href="#-支持的格式-75">支持格式</a>
  &nbsp;•&nbsp;
  <a href="#-键盘快捷键">快捷键</a>
  &nbsp;•&nbsp;
  <a href="#-支持开发--buy-me-a-coffee"><strong>赞助支持 ☕</strong></a>
</p>

</div>

---

## 🖼️ 界面截图

<p align="center">
  <img src="assets/screenshots/main_interface.png" width="95%" alt="Windows 11 上的 VISOR 主界面">
</p>

<p align="center">
  <em>流畅的 Windows 11 Fluent Design 风格、自适应工具栏与底部异步胶卷缩略图栏。</em>
</p>

---

## ⚡ 为什么选择 VISOR？

Windows 自带的照片查看器逐渐变得臃肿迟缓，充斥着不必要的联网遥测和冗余功能。**VISOR** 使用 **纯原生 Rust** 从零构建，依托 GPU 硬件加速，带来极致体验：

* 🚀 **毫秒级极速启动：** 打开超大分辨率图像无需等待，告别卡顿。
* 🛡️ **绝对隐私保护：** **100% 离线运行**。绝不上传任何照片、缩略图、元数据或浏览记录。
* 🍃 **超低闲置占用：** 闲置时仅消耗 20 至 40 MB 内存，CPU 占用率为 0%。
* 🎯 **默认 100% (1:1) 原生渲染：** 打开即展示真实像素，支持以鼠标指针为中心的 1% 至 6400% 平滑连续无级缩放。
* 🔍 **动态点对点模式 (Pixel-Perfect)：** 纯净 Nearest Neighbor 采样，专为像素画、游戏贴图与矢量检查设计。
* 🎞️ **底部胶卷栏 (Filmstrip)：** 多图浏览更便捷，后台异步线程预先渲染缩略图。
* 📊 **EXIF 与 ICC 专业摄影面板：** 深度读取相机型号、镜头、光圈、ISO、快门速度、色彩配置文件与 GPS 坐标。
* 🔄 **内置批量导出器：** 支持整文件夹批量格式转换，保留高达 16 位色彩深度。
* 🌐 **多语言界面：** 支持简体中文、英语、葡萄牙语、西班牙语、法语、德语、意大利语、俄语与日语。

---

## 📸 视觉演示

| 底部胶卷缩略图 (Filmstrip) | 技术信息面板 (EXIF) 与菜单 |
| :---: | :---: |
| <img src="assets/screenshots/filmstrip_view.png" width="460" alt="胶卷缩略图浏览"> | <img src="assets/screenshots/info_panel.png" width="460" alt="EXIF 技术面板与控制菜单"> |
| *解耦异步加载，平滑顺畅* | *全面查看摄影与技术元数据，插值模式与右键菜单快速切换* |

---

## 📊 横向对比

| 功能特性 | VISOR | Windows 照片 | IrfanView | ImageGlass |
| :--- | :---: | :---: | :---: | :---: |
| **核心开发语言** | **原生 Rust** | C# / WinUI | C++ | C# / .NET |
| **冷启动时间** | **< 30 ms** | ~1.5 s - 3 s | ~100 ms | ~800 ms |
| **后台遥测与追踪** | **零 (100% 离线)** | 持续存在 | 无 | 部分存在 |
| **现代格式 (AVIF, JXL, WebP)** | **原生支持** | 有限支持 | 需外置插件 | 支持 |
| **专业单反 RAW (23+ 品牌)** | **原生支持** | 需安装扩展包 | 需 C 插件 | 部分支持 |
| **游戏开发贴图 (DDS, KTX2, VTF)**| **原生支持** | 不支持 | 不支持 | 不支持 |
| **Windows 11 现代 Fluent 美学**| **现代原生** | 现代原生 | 90年代复古 | 现代风格 |

---

## 📦 支持的格式 (75+)

* **专业摄影与常见光栅格式：** PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16位), BMP, DIB, QOI, HDR Radiance, OpenEXR (EXR), ICO, CUR, Adobe Photoshop (PSD, PSB 合并图层), GIMP (XCF), OpenRaster (ORA), Farbfeld, JPEG 2000 (JP2, J2K), Netpbm (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP, XBM。
* **矢量、CAD 与设计排版：** CorelDRAW (.cdr v3 至 2024+ 内嵌预览图提取), SVG 与 SVGZ 原生解析, AutoCAD DXF, Windows EMF 与 WMF (基于 GDI+), 以及通过 PATH 中的 ImageMagick 支持 PDF, EPS, PS, AI。
* **动图与交互动画：** GIF, APNG, Windows 动态光标 (ANI), MNG, Lottie 矢量动画 (.lottie, .json) 与 Telegram 贴纸 (.tgs)。
* **单反相机 RAW 底片：** 佳能 (CR2, CRW, CR3), 尼康 (NEF, NRW), 索尼 (ARW, SRF, SR2), 富士 (RAF), 奥林巴斯 (ORF), 松下 (RW2), 宾得 (PEF), 哈苏 (3FR), 飞思 (IIQ, CAP), 爱普生 (ERF), 柯达 (KDC, DCR), 玛米亚 (MOS), 美能达 (MRW), 适马 (X3F), 卡西欧 (BAY) 与 Adobe DNG。
* **游戏引擎贴图：** DirectDraw Surface (DDS 支持 BC1 至 BC7 与 Mipmaps), Khronos KTX 1 / KTX 2, Valve VTF, 暴雪 BLP, 半条命 Sprites (.spr), RenderWare TXD, ASTC, Basis Universal, PowerVR PVR 与 Klei KTEX。

---

## ⌨️ 键盘快捷键

| 快捷键 | 功能操作 |
| :--- | :--- |
| `Ctrl` + `O` | 打开图片文件 |
| `←` / `→` | 查看上一张 / 下一张图片 |
| `Home` / `End` | 跳转至文件夹第一张 / 最后一张图片 |
| `1` | 缩放至 100% (原始尺寸 1:1) |
| `F` | 在适应窗口与原始大小之间切换 |
| `P` | 切换采样模式 (点对点 / 平滑 / 自动) |
| `R` / `Shift` + `R` | 顺时针 / 逆时针旋转 90° |
| `+` / `−` | 放大 / 缩小 |
| `鼠标滚轮` | 以光标为中心缩放 (可在设置中改为翻页) |
| `T` | 开启 / 关闭底部胶卷缩略图栏 |
| `I` | 开启 / 关闭 EXIF 技术信息面板 |
| `B` | 循环切换画布背景 (暗黑棋盘 / 明亮棋盘 / 纯黑 / 纯白 / 自定义颜色) |
| `空格键` | 播放 / 暂停动画 (GIF, APNG, Lottie) |
| `Ctrl` + `C` | 复制图片至剪贴板 (带 Alpha 透明通道) |
| `Delete` | 安全移动图片至 Windows 回收站 |
| `Ctrl` + `,` | 打开偏好设置 |
| `F11` | 全屏模式切换 |

---

## 📥 下载与安装

### 方式 1: 官方安装包 (.exe) 或 免安装便携版 (.zip)
前往 [Releases 页面](https://github.com/Tsujiu/visor/releases/latest) 下载最新版本（包含 Inno Setup 官方安装包与随拷随用的便携压缩包）。

### 方式 2: 从源码编译
确保系统已安装 **Rust** 编译器：
```bash
git clone https://github.com/Tsujiu/visor.git
cd visor
cargo build --release
# 生成的可执行文件位于: target/release/visor.exe
```

---

## ☕ 支持开发 / Buy Me a Coffee

**VISOR** 是一款完全免费、无广告、无商业捆绑且高度注重隐私的独立开源软件。如果您喜欢 VISOR 并希望支持后续开发，欢迎赞助：

<p align="center">
  <a href="https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04"><img src="https://img.shields.io/badge/Stripe_%E5%85%A8%E7%90%83%E8%B5%9E%E5%8A%A9-%E8%87%AA%E5%AE%9A%E4%B9%89%E9%87%91%E9%A2%9D_(USD)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="通过 Stripe 赞助任意金额 USD"></a>
  &nbsp;&nbsp;
  <a href="https://linknabio.gg/mutsujii"><img src="https://img.shields.io/badge/PIX_%E5%8F%8A%E5%B7%B4%E8%A5%BF%E6%94%AF%E4%BB%98-%E8%AF%B7%E5%96%9D%E5%92%96%E5%95%A1_☕-00C9A7?style=for-the-badge&logo=pix&logoColor=white" alt="InfinitePay 巴西支付通道"></a>
  &nbsp;&nbsp;
  <a href="https://donate.stripe.com/test_5kQ4gydHc5tO6Cf5Ko3VC05"><img src="https://img.shields.io/badge/Stripe_%E5%B7%B4%E8%A5%BF%E5%8C%BA-%E8%87%AA%E5%AE%9A%E4%B9%89%E9%87%91%E9%A2%9D_(BRL)_☕-635BFF?style=for-the-badge&logo=stripe&logoColor=white" alt="通过 Stripe BRL 赞助"></a>
</p>

* **🌍 国际多币种赞助 (Stripe - 自定义任意金额):** [点击进入 Stripe 赞助页面 (可自选 \$1, \$5, \$10, \$20, \$50 或任意金额)](https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04)
* **🇧🇷 巴西用户专属通道 (PIX / InfinitePay):** [https://linknabio.gg/mutsujii](https://linknabio.gg/mutsujii)

---

## 📄 开源许可证

遵循开源及个人免费许可。详见 [LICENSE](LICENSE) 文件。

---

<p align="center">
  由 <strong>Tsujiu</strong> 匠心打造 • 用 ❤️ 以 Rust 编写
</p>
