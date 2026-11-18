//! Módulo de integração avançada com a Área de Transferência (Clipboard) do Windows.
//!
//! Fornece suporte simultâneo a:
//! - **PNG:** formato moderno registrado (`"PNG"`), preservando canal alfa de 8 bits e compressão sem perdas.
//! - **CF_DIBV5:** formato padrão DIB com cabeçalho `BITMAPV5HEADER`, orientação top-down e canal alfa premultiplicado.
//!
//! Toda a alocação de memória global (`GlobalAlloc`) segue estritamente as regras de propriedade do Windows.

#![allow(dead_code)]

use std::ptr;
use std::time::Duration;

pub const CF_DIBV5: u32 = 17;
pub const BI_BITFIELDS: u32 = 3;
pub const LCS_SRGB: u32 = 0x73524742; // 'sRGB'
pub const LCS_GM_IMAGES: u32 = 4;
pub const GMEM_MOVEABLE: u32 = 0x0002;

/// Cabeçalho Win32 `BITMAPV5HEADER` (124 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapV5Header {
    pub b_v5_size: u32,
    pub b_v5_width: i32,
    pub b_v5_height: i32,
    pub b_v5_planes: u16,
    pub b_v5_bit_count: u16,
    pub b_v5_compression: u32,
    pub b_v5_size_image: u32,
    pub b_v5_x_pels_per_meter: i32,
    pub b_v5_y_pels_per_meter: i32,
    pub b_v5_clr_used: u32,
    pub b_v5_clr_important: u32,
    pub b_v5_red_mask: u32,
    pub b_v5_green_mask: u32,
    pub b_v5_blue_mask: u32,
    pub b_v5_alpha_mask: u32,
    pub b_v5_cs_type: u32,
    pub b_v5_endpoints: [u8; 36],
    pub b_v5_gamma_red: u32,
    pub b_v5_gamma_green: u32,
    pub b_v5_gamma_blue: u32,
    pub b_v5_intent: u32,
    pub b_v5_profile_data: u32,
    pub b_v5_profile_size: u32,
    pub b_v5_reserved: u32,
}

impl Default for BitmapV5Header {
    fn default() -> Self {
        Self {
            b_v5_size: 124,
            b_v5_width: 0,
            b_v5_height: 0,
            b_v5_planes: 1,
            b_v5_bit_count: 32,
            b_v5_compression: BI_BITFIELDS,
            b_v5_size_image: 0,
            b_v5_x_pels_per_meter: 0,
            b_v5_y_pels_per_meter: 0,
            b_v5_clr_used: 0,
            b_v5_clr_important: 0,
            b_v5_red_mask: 0x00FF0000,
            b_v5_green_mask: 0x0000FF00,
            b_v5_blue_mask: 0x000000FF,
            b_v5_alpha_mask: 0xFF000000,
            b_v5_cs_type: LCS_SRGB,
            b_v5_endpoints: [0; 36],
            b_v5_gamma_red: 0,
            b_v5_gamma_green: 0,
            b_v5_gamma_blue: 0,
            b_v5_intent: LCS_GM_IMAGES,
            b_v5_profile_data: 0,
            b_v5_profile_size: 0,
            b_v5_reserved: 0,
        }
    }
}

pub type HWND = *mut std::ffi::c_void;
pub type HANDLE = *mut std::ffi::c_void;
pub type HGLOBAL = *mut std::ffi::c_void;
pub type BOOL = i32;

#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    fn CloseClipboard() -> BOOL;
    fn EmptyClipboard() -> BOOL;
    fn SetClipboardData(uFormat: u32, hMem: HANDLE) -> HANDLE;
    fn RegisterClipboardFormatW(lpszFormat: *const u16) -> u32;
    fn GetActiveWindow() -> HWND;
    fn GetForegroundWindow() -> HWND;
    fn GetDesktopWindow() -> HWND;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> *mut std::ffi::c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
}

/// Guard RAII para fechamento garantido do clipboard do Windows.
pub struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}

/// Resultado detalhado da operação de publicação na área de transferência.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardPublishResult {
    /// Tanto PNG quanto DIBV5 foram publicados com sucesso.
    Success,
    /// Apenas o formato PNG foi publicado com sucesso.
    PartialPngOnly,
    /// Apenas o formato CF_DIBV5 foi publicado com sucesso.
    PartialDibOnly,
    /// Nenhum formato pôde ser publicado.
    Failed(String),
}

/// Codifica os pixels brutos RGBA em um stream de bytes PNG válido em memória.
///
/// Preserva dimensões originais, transparência direta (straight alpha) sem premultiplicação,
/// cabeçalhos e marcadores padronizados da especificação W3C/PNG.
pub fn prepare_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("dimensões inválidas (largura ou altura zero)".to_owned());
    }

    let expected_bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|px| px.checked_mul(4))
        .ok_or_else(|| "overflow ao calcular tamanho do buffer RGBA".to_owned())?;

    if rgba.len() != expected_bytes {
        return Err(format!(
            "tamanho do buffer ({}) não corresponde às dimensões {}x{} (esperado {})",
            rgba.len(),
            width,
            height,
            expected_bytes
        ));
    }

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        rgba,
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|err| format!("falha ao codificar PNG: {err}"))?;

    Ok(png_bytes)
}

/// Prepara a estrutura `CF_DIBV5` contendo `BITMAPV5HEADER` (124 bytes) e pixels BGRA premultiplicados.
///
/// Características técnicas:
/// - Orientação **top-down** (`b_v5_height = -(height as i32)`), garantindo ordem natural de linhas de cima para baixo.
/// - Formato 32 bpp com compressão `BI_BITFIELDS` e máscaras de cor explícitas para sRGB e canal alfa.
/// - Premultiplicação exata dos canais RGB pelo alfa para compatibilidade com o subsistema gráfico do Windows (GDI/DWM):
///   `Color_prem = (Color * Alpha + 127) / 255`.
/// - Stride com alinhamento DWORD (garantido em 32-bit: `width * 4`).
pub fn prepare_dibv5(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("dimensões inválidas (largura ou altura zero)".to_owned());
    }

    let stride = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| "overflow no cálculo do stride".to_owned())?;

    let image_size = stride
        .checked_mul(height as usize)
        .ok_or_else(|| "overflow no cálculo do tamanho da imagem DIBV5".to_owned())?;

    let total_size = image_size
        .checked_add(124)
        .ok_or_else(|| "overflow no cálculo do tamanho total do DIBV5".to_owned())?;

    let expected_input = image_size;
    if rgba.len() != expected_input {
        return Err(format!(
            "tamanho do buffer ({}) não corresponde às dimensões {}x{} (esperado {})",
            rgba.len(),
            width,
            height,
            expected_input
        ));
    }

    let header = BitmapV5Header {
        b_v5_size: 124,
        b_v5_width: width as i32,
        b_v5_height: -(height as i32), // Top-down
        b_v5_planes: 1,
        b_v5_bit_count: 32,
        b_v5_compression: BI_BITFIELDS,
        b_v5_size_image: image_size as u32,
        b_v5_x_pels_per_meter: 0,
        b_v5_y_pels_per_meter: 0,
        b_v5_clr_used: 0,
        b_v5_clr_important: 0,
        b_v5_red_mask: 0x00FF0000,
        b_v5_green_mask: 0x0000FF00,
        b_v5_blue_mask: 0x000000FF,
        b_v5_alpha_mask: 0xFF000000,
        b_v5_cs_type: LCS_SRGB,
        b_v5_endpoints: [0; 36],
        b_v5_gamma_red: 0,
        b_v5_gamma_green: 0,
        b_v5_gamma_blue: 0,
        b_v5_intent: LCS_GM_IMAGES,
        b_v5_profile_data: 0,
        b_v5_profile_size: 0,
        b_v5_reserved: 0,
    };

    let mut dib_bytes = Vec::with_capacity(total_size);
    let header_slice =
        unsafe { std::slice::from_raw_parts(&header as *const BitmapV5Header as *const u8, 124) };
    dib_bytes.extend_from_slice(header_slice);

    // Ordem top-down: percorre linhas de y=0 até height-1
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // Premultiplicação matemática precisa: (C * A + 127) / 255
        let pb = ((b as u32 * a as u32 + 127) / 255) as u8;
        let pg = ((g as u32 * a as u32 + 127) / 255) as u8;
        let pr = ((r as u32 * a as u32 + 127) / 255) as u8;

        dib_bytes.push(pb);
        dib_bytes.push(pg);
        dib_bytes.push(pr);
        dib_bytes.push(a);
    }

    Ok(dib_bytes)
}

/// Retorna o identificador numérico registrado do formato `"PNG"` no Windows.
pub fn get_png_clipboard_format() -> u32 {
    let name: Vec<u16> = "PNG\0".encode_utf16().collect();
    unsafe { RegisterClipboardFormatW(name.as_ptr()) }
}

/// Obtém um HWND de janela válido para associar como proprietário do clipboard.
pub fn get_valid_window_handle() -> Option<HWND> {
    let active = unsafe { GetActiveWindow() };
    if !active.is_null() {
        return Some(active);
    }
    let fg = unsafe { GetForegroundWindow() };
    if !fg.is_null() {
        return Some(fg);
    }
    let desktop = unsafe { GetDesktopWindow() };
    if !desktop.is_null() {
        return Some(desktop);
    }
    None
}

/// Abre o clipboard do Windows com tentativas limitadas e backoff para evitar contenção temporária.
pub fn open_clipboard_with_retry(
    hwnd: HWND,
    max_attempts: usize,
    delay: Duration,
) -> Result<ClipboardGuard, String> {
    for attempt in 0..max_attempts {
        if unsafe { OpenClipboard(hwnd) } != 0 {
            return Ok(ClipboardGuard);
        }
        if attempt + 1 < max_attempts {
            std::thread::sleep(delay);
        }
    }
    Err("a área de transferência do Windows está ocupada por outro aplicativo".to_owned())
}

/// Aloca memória global móvel (`GMEM_MOVEABLE`) e copia o buffer fornecido.
pub fn allocate_and_copy_hglobal(data: &[u8]) -> Result<HGLOBAL, String> {
    if data.is_empty() {
        return Err("dados vazios para alocação de clipboard".to_owned());
    }

    let hmem = unsafe { GlobalAlloc(GMEM_MOVEABLE, data.len()) };
    if hmem.is_null() {
        return Err("GlobalAlloc falhou ao alocar memória para o clipboard".to_owned());
    }

    let ptr = unsafe { GlobalLock(hmem) };
    if ptr.is_null() {
        unsafe {
            GlobalFree(hmem);
        }
        return Err("GlobalLock falhou ao travar memória do clipboard".to_owned());
    }

    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
        GlobalUnlock(hmem);
    }

    Ok(hmem)
}

/// Publica os formatos PNG e CF_DIBV5 na área de transferência do Windows.
///
/// Gerencia adequadamente a transferência de propriedade dos handles `HGLOBAL`:
/// - Se `SetClipboardData` tiver êxito, a propriedade é transferida ao Windows (não chama `GlobalFree`).
/// - Se `SetClipboardData` falhar, o handle é desalocado imediatamente para evitar vazamento de memória.
pub fn publish_to_clipboard(
    hwnd: HWND,
    png: Option<&[u8]>,
    dibv5: Option<&[u8]>,
) -> ClipboardPublishResult {
    if png.is_none() && dibv5.is_none() {
        return ClipboardPublishResult::Failed("nenhum dado de imagem para copiar".to_owned());
    }

    if hwnd.is_null() {
        return ClipboardPublishResult::Failed("handle de janela inválido".to_owned());
    }

    let _guard = match open_clipboard_with_retry(hwnd, 5, Duration::from_millis(10)) {
        Ok(g) => g,
        Err(err) => return ClipboardPublishResult::Failed(err),
    };

    if unsafe { EmptyClipboard() } == 0 {
        return ClipboardPublishResult::Failed("EmptyClipboard falhou".to_owned());
    }

    let mut png_published = false;
    let mut dib_published = false;

    // 1. Publica formato registrado "PNG"
    if let Some(png_bytes) = png {
        let png_fmt = get_png_clipboard_format();
        if png_fmt != 0 {
            if let Ok(hmem) = allocate_and_copy_hglobal(png_bytes) {
                let res = unsafe { SetClipboardData(png_fmt, hmem) };
                if !res.is_null() {
                    png_published = true;
                } else {
                    unsafe {
                        GlobalFree(hmem);
                    }
                }
            }
        }
    }

    // 2. Publica formato padrão CF_DIBV5
    if let Some(dib_bytes) = dibv5 {
        if let Ok(hmem) = allocate_and_copy_hglobal(dib_bytes) {
            let res = unsafe { SetClipboardData(CF_DIBV5, hmem) };
            if !res.is_null() {
                dib_published = true;
            } else {
                unsafe {
                    GlobalFree(hmem);
                }
            }
        }
    }

    match (png_published, dib_published) {
        (true, true) => ClipboardPublishResult::Success,
        (true, false) => ClipboardPublishResult::PartialPngOnly,
        (false, true) => ClipboardPublishResult::PartialDibOnly,
        (false, false) => ClipboardPublishResult::Failed(
            "não foi possível publicar os dados na área de transferência".to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmapv5_header_layout_and_size() {
        assert_eq!(std::mem::size_of::<BitmapV5Header>(), 124);
        let header = BitmapV5Header::default();
        assert_eq!(header.b_v5_size, 124);
        assert_eq!(header.b_v5_bit_count, 32);
        assert_eq!(header.b_v5_planes, 1);
        assert_eq!(header.b_v5_compression, BI_BITFIELDS);
        assert_eq!(header.b_v5_red_mask, 0x00FF0000);
        assert_eq!(header.b_v5_green_mask, 0x0000FF00);
        assert_eq!(header.b_v5_blue_mask, 0x000000FF);
        assert_eq!(header.b_v5_alpha_mask, 0xFF000000);
        assert_eq!(header.b_v5_cs_type, LCS_SRGB);
    }

    #[test]
    fn test_prepare_png_valid_dimensions_and_transparency() {
        let width = 2u32;
        let height = 2u32;
        // 4 pixels: transparente, semitransparente, opaco vermelho, opaco azul
        let rgba = vec![
            0, 0, 0, 0, // totalmente transparente
            255, 255, 255, 128, // semitransparente
            255, 0, 0, 255, // opaco vermelho
            0, 0, 255, 255, // opaco azul
        ];

        let png_bytes = prepare_png(&rgba, width, height).expect("codificação PNG");
        assert!(png_bytes.len() > 8);
        // Assinatura PNG: 89 50 4E 47 0D 0A 1A 0A
        assert_eq!(
            &png_bytes[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        // Decodifica de volta e valida preservação de pixels e transparência
        let decoded = image::load_from_memory(&png_bytes).expect("decodificação do PNG gerado");
        assert_eq!(decoded.width(), width);
        assert_eq!(decoded.height(), height);

        let img_rgba = decoded.to_rgba8();
        assert_eq!(img_rgba.get_pixel(0, 0).0, [0, 0, 0, 0]);
        assert_eq!(img_rgba.get_pixel(1, 0).0, [255, 255, 255, 128]);
        assert_eq!(img_rgba.get_pixel(0, 1).0, [255, 0, 0, 255]);
        assert_eq!(img_rgba.get_pixel(1, 1).0, [0, 0, 255, 255]);
    }

    #[test]
    fn test_prepare_png_invalid_dimensions_or_buffer() {
        assert!(prepare_png(&[], 0, 10).is_err());
        assert!(prepare_png(&[], 10, 0).is_err());
        assert!(prepare_png(&[1, 2, 3], 2, 2).is_err());
    }

    #[test]
    fn test_prepare_png_overflow_protection() {
        let huge_w = u32::MAX;
        let huge_h = u32::MAX;
        assert!(prepare_png(&[0; 4], huge_w, huge_h).is_err());
    }

    #[test]
    fn test_prepare_dibv5_1x1_and_header_fields() {
        let rgba = vec![100, 150, 200, 255];
        let dib = prepare_dibv5(&rgba, 1, 1).expect("preparar DIB 1x1");
        assert_eq!(dib.len(), 124 + 4);

        let header: BitmapV5Header = unsafe {
            let mut h = std::mem::MaybeUninit::<BitmapV5Header>::uninit();
            ptr::copy_nonoverlapping(dib.as_ptr(), h.as_mut_ptr() as *mut u8, 124);
            h.assume_init()
        };

        assert_eq!(header.b_v5_size, 124);
        assert_eq!(header.b_v5_width, 1);
        assert_eq!(header.b_v5_height, -1); // Top-down
        assert_eq!(header.b_v5_bit_count, 32);
        assert_eq!(header.b_v5_size_image, 4);

        // Pixel BGRA (opaco: sem alteração)
        let pixel = &dib[124..128];
        assert_eq!(pixel, &[200, 150, 100, 255]); // B, G, R, A
    }

    #[test]
    fn test_prepare_dibv5_alpha_premultiplication() {
        // Testa:
        // 1. Totalmente transparente: A=0 -> RGB deve ser 0
        // 2. Semitransparente: R=200, G=100, B=50, A=128
        //    pB = (50*128 + 127)/255 = (6400+127)/255 = 6527/255 = 25
        //    pG = (100*128 + 127)/255 = (12800+127)/255 = 12927/255 = 50
        //    pR = (200*128 + 127)/255 = (25600+127)/255 = 25727/255 = 100
        // 3. Totalmente opaco: A=255 -> RGB idêntico
        let rgba = vec![
            255, 255, 255, 0, // 1. Transparente
            200, 100, 50, 128, // 2. Semitransparente
            10, 20, 30, 255, // 3. Opaco
            0, 0, 0, 255, // 4. Preto opaco
        ];

        let dib = prepare_dibv5(&rgba, 4, 1).expect("preparar DIB");
        assert_eq!(dib.len(), 124 + 16);

        let pixels = &dib[124..];
        // Pixel 1: Transparente
        assert_eq!(&pixels[0..4], &[0, 0, 0, 0]);

        // Pixel 2: Semitransparente
        assert_eq!(&pixels[4..8], &[25, 50, 100, 128]);

        // Pixel 3: Opaco
        assert_eq!(&pixels[8..12], &[30, 20, 10, 255]);

        // Pixel 4: Preto opaco
        assert_eq!(&pixels[12..16], &[0, 0, 0, 255]);
    }

    #[test]
    fn test_prepare_dibv5_top_down_orientation() {
        // Imagem 2x2:
        // Linha 0 (topo): Vermelho [255, 0, 0, 255], Verde [0, 255, 0, 255]
        // Linha 1 (base): Azul [0, 0, 255, 255], Branco [255, 255, 255, 255]
        let rgba = vec![
            255, 0, 0, 255, 0, 255, 0, 255, // Linha 0
            0, 0, 255, 255, 255, 255, 255, 255, // Linha 1
        ];

        let dib = prepare_dibv5(&rgba, 2, 2).expect("DIB top-down");
        let header: BitmapV5Header = unsafe {
            let mut h = std::mem::MaybeUninit::<BitmapV5Header>::uninit();
            ptr::copy_nonoverlapping(dib.as_ptr(), h.as_mut_ptr() as *mut u8, 124);
            h.assume_init()
        };

        assert_eq!(header.b_v5_height, -2); // Altura negativa = Top-down

        let pixels = &dib[124..];
        // Linha 0 no buffer DIB deve ser a Linha 0 de entrada:
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]); // Vermelho em BGRA
        assert_eq!(&pixels[4..8], &[0, 255, 0, 255]); // Verde em BGRA

        // Linha 1 no buffer DIB deve ser a Linha 1 de entrada:
        assert_eq!(&pixels[8..12], &[255, 0, 0, 255]); // Azul em BGRA
        assert_eq!(&pixels[12..16], &[255, 255, 255, 255]); // Branco em BGRA
    }

    #[test]
    fn test_prepare_dibv5_overflow_protection() {
        let huge_w = u32::MAX;
        let huge_h = u32::MAX;
        assert!(prepare_dibv5(&[0; 4], huge_w, huge_h).is_err());
        assert!(prepare_dibv5(&[], 0, 0).is_err());
    }

    #[test]
    fn test_get_png_clipboard_format_id_is_nonzero() {
        let id = get_png_clipboard_format();
        assert!(
            id != 0,
            "RegisterClipboardFormatW de PNG deve retornar ID válido"
        );
    }

    #[test]
    fn test_allocate_and_copy_hglobal_empty_rejected() {
        assert!(allocate_and_copy_hglobal(&[]).is_err());
    }

    #[test]
    fn test_allocate_and_copy_hglobal_valid_data() {
        let test_data = b"VISOR_CLIPBOARD_TEST_BUFFER";
        let hmem = allocate_and_copy_hglobal(test_data).expect("GlobalAlloc e cópia");
        assert!(!hmem.is_null());

        let ptr = unsafe { GlobalLock(hmem) };
        assert!(!ptr.is_null());

        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, test_data.len()) };
        assert_eq!(slice, test_data);

        unsafe {
            GlobalUnlock(hmem);
            GlobalFree(hmem);
        }
    }

    #[test]
    fn test_integration_snapshot_animation_multi_frame() {
        // Simula uma animação com 2 quadros distintos
        let frame0 = visor::Frame {
            rgba: vec![255, 0, 0, 255], // Vermelho
            width: 1,
            height: 1,
            delay_ms: 100,
        };
        let frame1 = visor::Frame {
            rgba: vec![0, 255, 0, 255], // Verde
            width: 1,
            height: 1,
            delay_ms: 100,
        };

        let decoded = std::sync::Arc::new(visor::Decoded {
            frames: vec![frame0, frame1],
            info: visor::ImageInfo {
                id: "gif",
                name: "anim.gif".to_owned(),
                category: visor::Category::Animation,
                support: visor::Support::Full,
                bit_depth: 8,
                color: "RGBA8".to_owned(),
                original_w: 1,
                original_h: 1,
                display_w: 1,
                display_h: 1,
                reduced: false,
                has_alpha: false,
                frame_count: 2,
                loop_forever: true,
                mipmaps: 0,
                exif: Vec::new(),
                icc: None,
                notes: Vec::new(),
            },
        });

        // Quando frame_i == 1, o snapshot extrai exatamente o frame 1 (verde)
        let frame_i = 1;
        let active_frame = &decoded.frames[frame_i];
        let png = prepare_png(&active_frame.rgba, active_frame.width, active_frame.height).unwrap();
        let dib =
            prepare_dibv5(&active_frame.rgba, active_frame.width, active_frame.height).unwrap();

        // Verifica que o pixel copiado foi o verde (B=0, G=255, R=0, A=255)
        assert_eq!(&dib[124..128], &[0, 255, 0, 255]);
        let reloaded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(reloaded.get_pixel(0, 0).0, [0, 255, 0, 255]);
    }

    #[test]
    fn test_integration_snapshot_rotation_preservation() {
        // Imagem 2x1 (horizontal): Vermelho, Azul
        let pixels = vec![255, 0, 0, 255, 0, 0, 255, 255];
        let (rot_pixels, rot_w, rot_h) = crate::core::image_ops::rotate_pixels(&pixels, 2, 1, true);

        // Após rotação 90° horário, dimensões passam a ser 1x2 (vertical)
        assert_eq!(rot_w, 1);
        assert_eq!(rot_h, 2);

        let png = prepare_png(&rot_pixels, rot_w, rot_h).unwrap();
        let dib = prepare_dibv5(&rot_pixels, rot_w, rot_h).unwrap();

        let header: BitmapV5Header = unsafe {
            let mut h = std::mem::MaybeUninit::<BitmapV5Header>::uninit();
            ptr::copy_nonoverlapping(dib.as_ptr(), h.as_mut_ptr() as *mut u8, 124);
            h.assume_init()
        };
        assert_eq!(header.b_v5_width, 1);
        assert_eq!(header.b_v5_height, -2); // Top-down com 2 linhas

        let reloaded = image::load_from_memory(&png).unwrap();
        assert_eq!(reloaded.width(), 1);
        assert_eq!(reloaded.height(), 2);
    }

    #[test]
    fn test_integration_clipboard_null_hwnd_rejected() {
        let dummy_png = [1, 2, 3];
        let res = publish_to_clipboard(std::ptr::null_mut(), Some(&dummy_png), None);
        assert_eq!(
            res,
            ClipboardPublishResult::Failed("handle de janela inválido".to_owned())
        );
    }

    #[test]
    fn test_integration_clipboard_empty_inputs_rejected() {
        let fake_hwnd = 0x1234 as HWND;
        let res = publish_to_clipboard(fake_hwnd, None, None);
        assert_eq!(
            res,
            ClipboardPublishResult::Failed("nenhum dado de imagem para copiar".to_owned())
        );
    }

    #[test]
    fn test_integration_clipboard_guard_drop_safety() {
        // Garante que a instanciação e descarte de ClipboardGuard executa sem pânico
        let guard = ClipboardGuard;
        drop(guard);
    }
}
