use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use image::{Delay, DynamicImage, ExtendedColorType, ImageFormat, Rgba, RgbaImage};

use crate::catalog::{self, formats};
use crate::meta::{icc_note, jpeg_is_cmyk};
use crate::model::Support;
use crate::{export_image, open_image, ExportKind};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "visor-codec-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn px(path: &Path, x: u32, y: u32) -> [u8; 4] {
    let decoded = open_image(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    decoded.frames[0]
        .pixel(x, y)
        .unwrap_or_else(|| panic!("pixel fora {x},{y}"))
}

fn near(pixel: [u8; 4], expected: [u8; 4]) {
    for (got, want) in pixel.into_iter().zip(expected) {
        assert!((got as i16 - want as i16).abs() <= 3, "{pixel:?} vs {expected:?}");
    }
}

#[test]
fn filters_match_the_support_level() {
    let verified = catalog::verified_extensions();
    let partial = catalog::partial_extensions();
    for spec in formats() {
        for ext in spec.extensions {
            match spec.support {
                Support::Full | Support::ReadOnly => {
                    assert!(verified.contains(ext), "{ext} deveria estar no filtro verificado");
                    assert!(!partial.contains(ext), "{ext} não é parcial");
                }
                Support::Partial => {
                    assert!(partial.contains(ext), "{ext} deveria estar no filtro parcial");
                    assert!(!verified.contains(ext), "{ext} não foi verificado como completo");
                }
                Support::External | Support::Unsupported => {
                    assert!(!verified.contains(ext), "{ext} não é suportado de fato");
                    assert!(!partial.contains(ext), "{ext} não é leitura parcial");
                }
            }
        }
    }
    for ext in ["avif", "pdf", "cr3", "vtf", "blp", "jp2", "mng", "tgs", "dxf", "astc"] {
        assert!(!verified.contains(&ext));
        assert!(!partial.contains(&ext));
    }
}

#[test]
fn content_wins_over_the_extension() {
    let dir = temp_dir("sniff");
    let jpeg = dir.join("foto.png");
    DynamicImage::ImageRgb8(image::RgbImage::from_pixel(6, 3, image::Rgb([9, 8, 7])))
        .save_with_format(&jpeg, ImageFormat::Jpeg)
        .unwrap();
    let decoded = open_image(&jpeg).unwrap();
    assert_eq!(decoded.info.id, "jpeg");
    assert_eq!((decoded.info.original_w, decoded.info.original_h), (6, 3));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncated_png_is_rejected() {
    let dir = temp_dir("bad");
    let path = dir.join("ruim.png");
    fs::write(&path, b"\x89PNG\r\n\x1a\ntruncado").unwrap();
    let err = open_image(&path).unwrap_err();
    assert!(!err.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn raster_color_alpha_depth_and_exif() {
    let dir = temp_dir("raster");
    let png = dir.join("a.png");
    RgbaImage::from_pixel(8, 4, Rgba([10, 20, 30, 128]))
        .save(&png)
        .unwrap();
    let decoded = open_image(&png).unwrap();
    assert_eq!(decoded.info.id, "png");
    assert!(decoded.info.has_alpha);
    near(px(&png, 1, 1), [10, 20, 30, 128]);

    let wide = image::RgbImage::from_fn(8, 4, |x, y| image::Rgb([x as u8, y as u8, 40]));
    let jpeg = dir.join("a.jpg");
    DynamicImage::ImageRgb8(wide).save_with_format(&jpeg, ImageFormat::Jpeg).unwrap();
    let bytes = fs::read(&jpeg).unwrap();
    let oriented = with_orientation(&bytes, 6);
    let turned = dir.join("turn.jpg");
    fs::write(&turned, &oriented).unwrap();
    let decoded = open_image(&turned).unwrap();
    assert_eq!(
        (decoded.info.original_w, decoded.info.original_h),
        (4, 8),
        "exif={:?} inicio={:02x?}",
        decoded.info.exif,
        &oriented[..oriented.len().min(48)]
    );
    assert!(decoded.info.exif.iter().any(|(tag, _)| tag.contains("Orientation")));

    let luma = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_pixel(4, 2, image::Luma([40000]));
    let deep = dir.join("deep.png");
    DynamicImage::ImageLuma16(luma)
        .save_with_format(&deep, ImageFormat::Png)
        .unwrap();
    let decoded = open_image(&deep).unwrap();
    assert_eq!(decoded.info.bit_depth, 16);
    assert!(decoded.frames[0].pixel(0, 0).unwrap()[0] > 100);

    let bmp = dir.join("a.bmp");
    let tiff = dir.join("a.tiff");
    let tga = dir.join("a.tga");
    let qoi = dir.join("a.qoi");
    let ppm = dir.join("a.ppm");
    let webp = dir.join("a.webp");
    let ico = dir.join("a.ico");
    let rgb = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(5, 3, image::Rgb([200, 10, 20])));
    rgb.save_with_format(&bmp, ImageFormat::Bmp).unwrap();
    rgb.save_with_format(&tiff, ImageFormat::Tiff).unwrap();
    rgb.save_with_format(&tga, ImageFormat::Tga).unwrap();
    rgb.save_with_format(&qoi, ImageFormat::Qoi).unwrap();
    rgb.save_with_format(&ppm, ImageFormat::Pnm).unwrap();
    rgb.save_with_format(&webp, ImageFormat::WebP).unwrap();
    rgb.save_with_format(&ico, ImageFormat::Ico).unwrap();
    for path in [&bmp, &tiff, &tga, &qoi, &ppm, &webp, &ico] {
        near(px(path, 0, 0), [200, 10, 20, 255]);
    }

    let mut hdr = image::Rgb32FImage::new(4, 2);
    hdr.put_pixel(0, 0, image::Rgb([1.0, 0.0, 0.0]));
    let hdr_path = dir.join("a.hdr");
    DynamicImage::ImageRgb32F(hdr)
        .save_with_format(&hdr_path, ImageFormat::Hdr)
        .unwrap();
    let decoded = open_image(&hdr_path).unwrap();
    assert_eq!(decoded.info.id, "hdr");
    assert_eq!(decoded.info.bit_depth, 32);
    assert!(decoded.frames[0].pixel(0, 0).unwrap()[0] > 200);

    let exported = dir.join("out.png");
    export_image(&deep, &exported, ExportKind::Png).unwrap();
    assert_eq!(open_image(&exported).unwrap().info.bit_depth, 16);
    let jpeg_out = dir.join("out.jpg");
    export_image(&png, &jpeg_out, ExportKind::Jpeg).unwrap();
    assert_eq!(open_image(&jpeg_out).unwrap().info.original_w, 8);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn animations_keep_order_and_delay() {
    let dir = temp_dir("anim");
    let gif = dir.join("a.gif");
    let mut bytes = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        let red = RgbaImage::from_pixel(4, 2, Rgba([255, 0, 0, 255]));
        let blue = RgbaImage::from_pixel(4, 2, Rgba([0, 0, 255, 255]));
        encoder
            .encode_frame(image::Frame::from_parts(red, 0, 0, Delay::from_numer_denom_ms(80, 1)))
            .unwrap();
        encoder
            .encode_frame(image::Frame::from_parts(blue, 0, 0, Delay::from_numer_denom_ms(120, 1)))
            .unwrap();
    }
    fs::write(&gif, bytes).unwrap();
    let decoded = open_image(&gif).unwrap();
    assert_eq!(decoded.frames.len(), 2);
    assert_eq!(decoded.frames[0].delay_ms, 80);
    assert_eq!(decoded.frames[1].delay_ms, 120);
    near(decoded.frames[0].pixel(0, 0).unwrap(), [255, 0, 0, 255]);
    near(decoded.frames[1].pixel(0, 0).unwrap(), [0, 0, 255, 255]);

    let apng = dir.join("a.apng");
    fs::write(&apng, apng_bytes()).unwrap();
    let decoded = open_image(&apng).unwrap();
    assert!(decoded.frames.len() >= 2, "APNG deveria ter dois quadros");
    near(decoded.frames[0].pixel(0, 0).unwrap(), [255, 0, 0, 255]);
    near(decoded.frames[1].pixel(0, 0).unwrap(), [0, 255, 0, 255]);
    assert!(decoded.frames[0].delay_ms >= 70 && decoded.frames[0].delay_ms <= 90);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn svg_keeps_alpha_and_svgz_opens() {
    let dir = temp_dir("svg");
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="4"><rect x="4" width="4" height="4" fill="#ff0000"/><rect width="4" height="4" fill="#00ff00" fill-opacity="0.5"/></svg>"##;
    let path = dir.join("a.svg");
    fs::write(&path, svg).unwrap();
    let decoded = open_image(&path).unwrap();
    assert_eq!(decoded.info.id, "svg");
    near(decoded.frames[0].pixel(6, 1).unwrap(), [255, 0, 0, 255]);
    let left = decoded.frames[0].pixel(1, 1).unwrap();
    assert!(left[3] < 255 && left[1] > 80, "{left:?}");

    let gz = dir.join("a.svgz");
    let file = fs::File::create(&gz).unwrap();
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
    encoder.write_all(svg).unwrap();
    encoder.finish().unwrap();
    assert_eq!(open_image(&gz).unwrap().info.id, "svg");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn simple_formats_match_a_known_pixel() {
    let dir = temp_dir("simple");
    let pcx = dir.join("a.pcx");
    fs::write(&pcx, pcx24(4, 2, [0, 0, 255])).unwrap();
    near(px(&pcx, 1, 1), [0, 0, 255, 255]);

    let wbmp = dir.join("a.wbmp");
    fs::write(&wbmp, wbmp_bytes()).unwrap();
    near(px(&wbmp, 0, 0), [0, 0, 0, 255]);
    near(px(&wbmp, 1, 0), [255, 255, 255, 255]);

    let ras = dir.join("a.ras");
    fs::write(&ras, ras_bytes()).unwrap();
    near(px(&ras, 0, 0), [10, 20, 30, 255]);

    let sgi = dir.join("a.sgi");
    fs::write(&sgi, sgi_bytes()).unwrap();
    near(px(&sgi, 0, 0), [255, 0, 0, 255]);
    near(px(&sgi, 0, 1), [0, 0, 255, 255]);

    let wal = dir.join("a.wal");
    fs::write(&wal, wal_bytes()).unwrap();
    let decoded = open_image(&wal).unwrap();
    assert_eq!(decoded.info.support, Support::Partial);
    near(decoded.frames[0].pixel(0, 0).unwrap(), [0, 0, 0, 255]);
    near(decoded.frames[0].pixel(1, 0).unwrap(), [255, 255, 255, 255]);
    assert!(decoded.info.mipmaps >= 2);

    let dib = dir.join("a.dib");
    fs::write(&dib, dib_bytes()).unwrap();
    near(px(&dib, 0, 0), [7, 8, 9, 255]);

    let cur = dir.join("a.cur");
    fs::write(&cur, cur_with_png()).unwrap();
    near(px(&cur, 0, 0), [1, 2, 3, 255]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dds_rgba_and_bc1_and_ktx() {
    let dir = temp_dir("tex");
    let rgba = dir.join("a.dds");
    fs::write(&rgba, dds_dx10(2, 2, 28, 1, &[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 0, 0, 128])).unwrap();
    let decoded = open_image(&rgba).unwrap();
    near(decoded.frames[0].pixel(0, 0).unwrap(), [255, 0, 0, 255]);
    near(decoded.frames[0].pixel(1, 1).unwrap(), [0, 0, 0, 128]);

    let bc1 = dir.join("bc1.dds");
    fs::write(&bc1, dds_dx10(4, 4, 71, 1, &bc1_red())).unwrap();
    near(px(&bc1, 0, 0), [255, 0, 0, 255]);
    near(px(&bc1, 3, 3), [255, 0, 0, 255]);

    let ktx = dir.join("a.ktx");
    fs::write(&ktx, ktx1_two_mips()).unwrap();
    let decoded = open_image(&ktx).unwrap();
    assert_eq!(decoded.info.mipmaps, 2);
    near(decoded.frames[0].pixel(0, 0).unwrap(), [0, 255, 0, 255]);

    let ktx2 = dir.join("a.ktx2");
    fs::write(&ktx2, ktx2_rgba()).unwrap();
    near(px(&ktx2, 0, 0), [0, 0, 255, 128]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn exr_and_jxl_decode() {
    let dir = temp_dir("pro");
    let exr_path = dir.join("a.exr");
    write_exr(&exr_path);
    let decoded = open_image(&exr_path).unwrap();
    assert_eq!(decoded.info.id, "exr");
    assert!(decoded.frames[0].pixel(0, 0).unwrap()[0] > 200);

    let jxl = dir.join("a.jxl");
    fs::write(
        &jxl,
        [
            0xff, 0x0a, 0x30, 0x54, 0x10, 0x09, 0x08, 0x06, 0x01, 0x00, 0x78, 0x00, 0x4b, 0x38,
            0x41, 0x3c, 0xb6, 0x3a, 0x51, 0xfe, 0x00, 0x47, 0x1e, 0xa0, 0x85, 0xb8, 0x27, 0x1a,
            0x48, 0x45, 0x84, 0x1b, 0x71, 0x4f, 0xa8, 0x3e, 0x8e, 0x30, 0x03, 0x92, 0x84, 0x01,
        ],
    )
    .unwrap();
    let decoded = open_image(&jxl).unwrap();
    assert_eq!(decoded.info.id, "jxl");
    assert!(decoded.info.original_w > 0 && decoded.info.original_h > 0);
    assert_eq!(
        decoded.frames[0].rgba.len(),
        decoded.frames[0].width as usize * decoded.frames[0].height as usize * 4
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn emf_draws_a_flat_color() {
    let dir = temp_dir("emf");
    let path = dir.join("a.emf");
    let script = dir.join("make.ps1");
    let body = format!(
        r#"Add-Type -AssemblyName System.Drawing
$ms = New-Object System.IO.MemoryStream
$bmp = New-Object System.Drawing.Bitmap 8,4
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
$rect = New-Object System.Drawing.Rectangle 0,0,8,4
$mf = New-Object System.Drawing.Imaging.Metafile($ms, $hdc, $rect, [System.Drawing.Imaging.MetafileFrameUnit]::Pixel)
$g.ReleaseHdc($hdc)
$mg = [System.Drawing.Graphics]::FromImage($mf)
$mg.Clear([System.Drawing.Color]::FromArgb(255, 20, 40, 60))
$mg.Dispose(); $mf.Dispose(); $g.Dispose(); $bmp.Dispose()
[IO.File]::WriteAllBytes('{path}', $ms.ToArray())
"#,
        path = path.display().to_string().replace('\'', "''")
    );
    fs::write(&script, body).unwrap();
    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&script)
        .status()
        .unwrap();
    assert!(status.success());
    let decoded = open_image(&path).unwrap();
    assert_eq!(decoded.info.id, "emf");
    near(decoded.frames[0].pixel(1, 1).unwrap(), [20, 40, 60, 255]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn named_unsupported_files_do_not_decode() {
    let dir = temp_dir("nope");
    let vtf = dir.join("a.vtf");
    fs::write(&vtf, b"VTF\0rest").unwrap();
    let err = open_image(&vtf).unwrap_err();
    assert!(err.contains("não suportado") || err.contains("VTF"), "{err}");
    let blp = dir.join("a.blp");
    fs::write(&blp, b"BLP2xxxx").unwrap();
    assert!(open_image(&blp).is_err());
    let cr3 = dir.join("a.cr3");
    fs::write(&cr3, b"not a cr3").unwrap();
    assert!(open_image(&cr3).unwrap_err().to_lowercase().contains("cr3") || open_image(&cr3).is_err());
    let raw = dir.join("a.nef");
    fs::write(&raw, b"this is not a camera raw file at all!!").unwrap();
    assert!(open_image(&raw).is_err());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn metadata_helpers_do_not_execute_profiles() {
    assert!(jpeg_is_cmyk(&[
        0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x02, 0x04
    ]));
    assert!(!jpeg_is_cmyk(&[
        0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x02, 0x03
    ]));
    let mut icc = vec![0u8; 40];
    icc[0..4].copy_from_slice(&128u32.to_be_bytes());
    icc[36..40].copy_from_slice(b"acsp");
    let note = icc_note(&icc).unwrap();
    assert!(note.contains("128"));
    assert!(note.contains("não aplicado"));
}

#[test]
fn batch_export_writes_each_file() {
    let dir = temp_dir("batch");
    let a = dir.join("a.png");
    let b = dir.join("b.png");
    RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255])).save(&a).unwrap();
    RgbaImage::from_pixel(3, 1, Rgba([4, 5, 6, 255])).save(&b).unwrap();
    let out = dir.join("out");
    fs::create_dir_all(&out).unwrap();
    let report = crate::export_many(&[a, b], &out, ExportKind::Bmp);
    assert!(report.iter().all(|(_, result)| result.is_ok()));
    assert!(out.join("a.bmp").is_file());
    assert!(out.join("b.bmp").is_file());
    let _ = fs::remove_dir_all(dir);
}

fn with_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II");
    tiff.extend_from_slice(&42u16.to_le_bytes());
    tiff.extend_from_slice(&8u32.to_le_bytes());
    tiff.extend_from_slice(&1u16.to_le_bytes());
    tiff.extend_from_slice(&0x0112u16.to_le_bytes());
    tiff.extend_from_slice(&3u16.to_le_bytes());
    tiff.extend_from_slice(&1u32.to_le_bytes());
    tiff.extend_from_slice(&orientation.to_le_bytes());
    tiff.extend_from_slice(&0u16.to_le_bytes());
    tiff.extend_from_slice(&0u32.to_le_bytes());
    let mut app = Vec::new();
    app.extend_from_slice(b"Exif\0\0");
    app.extend_from_slice(&tiff);
    let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
    out.extend_from_slice(&(app.len() as u16 + 2).to_be_bytes());
    out.extend_from_slice(&app);
    out.extend_from_slice(&jpeg[2..]);
    out
}

fn apng_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, 2, 2);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_animated(2, 0).unwrap();
    encoder.set_frame_delay(8, 100).unwrap();
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&[255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255])
        .unwrap();
    writer.set_frame_delay(8, 100).unwrap();
    writer
        .write_image_data(&[0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255])
        .unwrap();
    drop(writer);
    out
}

fn pcx24(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let bpl = (width as usize + 1) & !1;
    let mut header = vec![0u8; 128];
    header[0] = 0x0A;
    header[1] = 5;
    header[2] = 1;
    header[3] = 8;
    header[8..10].copy_from_slice(&(width as u16 - 1).to_le_bytes());
    header[10..12].copy_from_slice(&(height as u16 - 1).to_le_bytes());
    header[65] = 3;
    header[66..68].copy_from_slice(&(bpl as u16).to_le_bytes());
    let mut body = Vec::new();
    for _y in 0..height {
        for channel in 0..3 {
            let mut left = bpl;
            while left > 0 {
                let count = left.min(63);
                body.push(0xC0 | count as u8);
                body.push(rgb[channel]);
                left -= count;
            }
        }
    }
    header.extend(body);
    header
}

fn wbmp_bytes() -> Vec<u8> {
    vec![0, 0, 8, 1, 0x80]
}

fn ras_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    for value in [0x59A66A95u32, 2, 1, 24, 8, 1, 0, 0] {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out.extend_from_slice(&[30, 20, 10, 0, 0, 0, 0, 0]);
    out
}

fn sgi_bytes() -> Vec<u8> {
    let mut out = vec![0u8; 512];
    out[0..2].copy_from_slice(&474u16.to_be_bytes());
    out[2] = 0;
    out[3] = 1;
    out[4..6].copy_from_slice(&3u16.to_be_bytes());
    out[6..8].copy_from_slice(&1u16.to_be_bytes());
    out[8..10].copy_from_slice(&2u16.to_be_bytes());
    out[10..12].copy_from_slice(&3u16.to_be_bytes());
    // file row 0 is the bottom: blue. file row 1 is the top: red.
    let mut pixels = Vec::new();
    for channel in [[0u8, 255], [0, 0], [255, 0]] {
        pixels.extend_from_slice(&channel);
    }
    out.extend(pixels);
    out
}

fn wal_bytes() -> Vec<u8> {
    let mut out = vec![0u8; 100];
    out[32..36].copy_from_slice(&2u32.to_le_bytes());
    out[36..40].copy_from_slice(&1u32.to_le_bytes());
    out[40..44].copy_from_slice(&100u32.to_le_bytes());
    out[44..48].copy_from_slice(&102u32.to_le_bytes());
    out.extend_from_slice(&[0, 255]);
    out.extend_from_slice(&[128]);
    out
}

fn dib_bytes() -> Vec<u8> {
    let mut out = vec![0u8; 40];
    out[0..4].copy_from_slice(&40u32.to_le_bytes());
    out[4..8].copy_from_slice(&1i32.to_le_bytes());
    out[8..12].copy_from_slice(&1i32.to_le_bytes());
    out[12..14].copy_from_slice(&1u16.to_le_bytes());
    out[14..16].copy_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[9, 8, 7, 0]);
    out
}

fn cur_with_png() -> Vec<u8> {
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, 2, 1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&[1, 2, 3, 255, 1, 2, 3, 255])
        .unwrap();
    let mut out = vec![0, 0, 2, 0, 1, 0, 2, 1, 0, 0, 0, 0, 0, 0];
    out.extend_from_slice(&(png.len() as u32).to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());
    out.extend(png);
    out
}

fn le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn dds_dx10(width: u32, height: u32, format: u32, mipmaps: u32, pixels: &[u8]) -> Vec<u8> {
    use ddsfile::{AlphaMode, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};
    let format = match format {
        28 => DxgiFormat::R8G8B8A8_UNorm,
        71 => DxgiFormat::BC1_UNorm,
        other => panic!("formato DXGI {other} sem construtor no teste"),
    };
    let mut dds = Dds::new_dxgi(NewDxgiParams {
        height,
        width,
        depth: None,
        format,
        mipmap_levels: Some(mipmaps.max(1)),
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: D3D10ResourceDimension::Texture2D,
        alpha_mode: AlphaMode::Straight,
    })
    .unwrap();
    let n = pixels.len().min(dds.data.len());
    dds.data[..n].copy_from_slice(&pixels[..n]);
    let mut out = Vec::new();
    dds.write(&mut out).unwrap();
    out
}

fn bc1_red() -> Vec<u8> {
    vec![0x00, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

fn ktx1_two_mips() -> Vec<u8> {
    let mut out = vec![
        0xAB, b'K', b'T', b'X', b' ', b'1', b'1', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ];
    le(&mut out, 0x04030201);
    le(&mut out, 0x1401);
    le(&mut out, 1);
    le(&mut out, 0x1908);
    le(&mut out, 0x8058);
    le(&mut out, 0x1908);
    le(&mut out, 2);
    le(&mut out, 2);
    le(&mut out, 0);
    le(&mut out, 0);
    le(&mut out, 1);
    le(&mut out, 2);
    le(&mut out, 0);
    le(&mut out, 16);
    out.extend_from_slice(&[0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255]);
    le(&mut out, 4);
    out.extend_from_slice(&[0, 255, 0, 255]);
    out
}

fn ktx2_rgba() -> Vec<u8> {
    let pixels = [0u8, 0, 255, 128, 0, 0, 255, 128, 0, 0, 255, 128, 0, 0, 255, 128];
    let dfd = dfd_rgba8();
    let header_len = 80usize;
    let index_len = 24usize;
    let dfd_offset = header_len + index_len;
    let data_offset = dfd_offset + dfd.len();
    let mut out = Vec::new();
    out.extend_from_slice(&[
        0xAB, b'K', b'T', b'X', b' ', b'2', b'0', 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]);
    le(&mut out, 37);
    le(&mut out, 1);
    le(&mut out, 2);
    le(&mut out, 2);
    le(&mut out, 0);
    le(&mut out, 0);
    le(&mut out, 1);
    le(&mut out, 1);
    le(&mut out, 0);
    le(&mut out, dfd_offset as u32);
    le(&mut out, dfd.len() as u32);
    le(&mut out, 0);
    le(&mut out, 0);
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&(data_offset as u64).to_le_bytes());
    out.extend_from_slice(&(pixels.len() as u64).to_le_bytes());
    out.extend_from_slice(&(pixels.len() as u64).to_le_bytes());
    out.extend_from_slice(&dfd);
    out.extend_from_slice(&pixels);
    out
}

fn dfd_rgba8() -> Vec<u8> {
    let mut block = Vec::new();
    block.extend_from_slice(&0u16.to_le_bytes());
    block.extend_from_slice(&0u16.to_le_bytes());
    block.extend_from_slice(&2u16.to_le_bytes());
    block.extend_from_slice(&88u16.to_le_bytes());
    block.extend_from_slice(&[1, 1, 1, 0, 0, 0, 0, 0]);
    block.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]);
    for (offset, channel) in [(0u16, 0u8), (8, 1), (16, 2), (24, 15)] {
        block.extend_from_slice(&offset.to_le_bytes());
        block.push(7);
        block.push(channel);
        block.extend_from_slice(&[0, 0, 0, 0]);
        block.extend_from_slice(&0u32.to_le_bytes());
        block.extend_from_slice(&255u32.to_le_bytes());
    }
    let mut dfd = Vec::new();
    dfd.extend_from_slice(&((4 + block.len()) as u32).to_le_bytes());
    dfd.extend(block);
    dfd
}

fn write_exr(path: &Path) {
    use exr::prelude::*;
    let image = Image::from_channels(
        (2, 2),
        SpecificChannels::rgb(|_pixel: Vec2<usize>| (1.0, 0.0, 0.0)),
    );
    image.write().to_file(path).unwrap();
}

#[allow(dead_code)]
fn _color_type_mention(color: ExtendedColorType) -> ExtendedColorType {
    color
}
