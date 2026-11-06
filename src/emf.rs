//! EMF via GDI+, só no Windows. O metafile é desenhado; nada é executado como script.

use std::os::raw::c_void;

use windows_sys::Win32::Graphics::GdiPlus::{
    BitmapData, GdipBitmapLockBits, GdipBitmapUnlockBits, GdipCreateBitmapFromScan0,
    GdipCreateMetafileFromStream, GdipDeleteGraphics, GdipDisposeImage, GdipDrawImageI,
    GdipGetImageGraphicsContext, GdipGetImageHeight, GdipGetImageWidth, GdiplusShutdown,
    GdiplusStartup, GdiplusStartupInput, GpBitmap, GpGraphics, GpImage, GpMetafile,
};
use windows_sys::Win32::UI::Shell::SHCreateMemStream;

use crate::model::{check_dims, Built, Frame};

const PIXEL_FORMAT_ARGB: i32 = 0x26200A;
const LOCK_READ: u32 = 1;

pub fn decode_emf(data: &[u8]) -> Result<Built, String> {
    if data.len() > 32 * 1024 * 1024 {
        return Err("EMF grande demais".to_owned());
    }
    unsafe {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: 0,
            SuppressExternalCodecs: 1,
        };
        let mut token = 0usize;
        let status = GdiplusStartup(&mut token, &input, std::ptr::null_mut());
        if status != 0 {
            return Err(format!("GDI+ não iniciou ({status})"));
        }
        let result = render(data);
        GdiplusShutdown(token);
        result
    }
}

unsafe fn render(data: &[u8]) -> Result<Built, String> {
    unsafe {
        let stream = SHCreateMemStream(data.as_ptr(), data.len() as u32);
        if stream.is_null() {
            return Err("não foi possível ler o EMF".to_owned());
        }
        let mut metafile: *mut GpMetafile = std::ptr::null_mut();
        let status = GdipCreateMetafileFromStream(stream, &mut metafile);
        release_stream(stream);
        if status != 0 || metafile.is_null() {
            return Err(format!("EMF recusado pelo GDI+ ({status})"));
        }
        let image = metafile.cast::<GpImage>();
        let mut width = 0u32;
        let mut height = 0u32;
        let status_w = GdipGetImageWidth(image, &mut width);
        let status_h = GdipGetImageHeight(image, &mut height);
        if status_w != 0 || status_h != 0 || width == 0 || height == 0 {
            GdipDisposeImage(image);
            return Err("EMF sem tamanho".to_owned());
        }
        if let Err(err) = check_dims(width, height) {
            GdipDisposeImage(image);
            return Err(err);
        }
        let mut bitmap: *mut GpBitmap = std::ptr::null_mut();
        let created = GdipCreateBitmapFromScan0(
            width as i32,
            height as i32,
            0,
            PIXEL_FORMAT_ARGB,
            std::ptr::null(),
            &mut bitmap,
        );
        if created != 0 || bitmap.is_null() {
            GdipDisposeImage(image);
            return Err("não foi possível criar o bitmap do EMF".to_owned());
        }
        let bitmap_image = bitmap.cast::<GpImage>();
        let mut graphics: *mut GpGraphics = std::ptr::null_mut();
        if GdipGetImageGraphicsContext(bitmap_image, &mut graphics) != 0 {
            GdipDisposeImage(bitmap_image);
            GdipDisposeImage(image);
            return Err("GDI+ sem contexto".to_owned());
        }
        let drawn = GdipDrawImageI(graphics, image, 0, 0);
        GdipDeleteGraphics(graphics);
        GdipDisposeImage(image);
        if drawn != 0 {
            GdipDisposeImage(bitmap_image);
            return Err(format!("não desenhou o EMF ({drawn})"));
        }
        let mut locked = BitmapData {
            Width: 0,
            Height: 0,
            Stride: 0,
            PixelFormat: 0,
            Scan0: std::ptr::null_mut(),
            Reserved: 0,
        };
        let rect = windows_sys::Win32::Graphics::GdiPlus::Rect {
            X: 0,
            Y: 0,
            Width: width as i32,
            Height: height as i32,
        };
        if GdipBitmapLockBits(bitmap, &rect, LOCK_READ, PIXEL_FORMAT_ARGB, &mut locked) != 0
            || locked.Scan0.is_null()
            || locked.Stride == 0
        {
            GdipDisposeImage(bitmap_image);
            return Err("não leu os pixels do EMF".to_owned());
        }
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        let stride = locked.Stride as isize;
        for y in 0..height as isize {
            let row = locked.Scan0.cast::<u8>().offset(y * stride);
            for x in 0..width as usize {
                let px = std::slice::from_raw_parts(row.add(x * 4), 4);
                let i = (y as usize * width as usize + x) * 4;
                rgba[i] = px[2];
                rgba[i + 1] = px[1];
                rgba[i + 2] = px[0];
                rgba[i + 3] = px[3];
            }
        }
        GdipBitmapUnlockBits(bitmap, &mut locked);
        GdipDisposeImage(bitmap_image);
        Ok(Built {
            frames: vec![Frame {
                rgba,
                width,
                height,
                delay_ms: 0,
            }],
            bit_depth: 8,
            color: "RGBA".to_owned(),
            mipmaps: 1,
            notes: vec!["EMF desenhado pelo GDI+ do Windows".to_owned()],
            id: "emf",
            loop_forever: false,
        })
    }
}

unsafe fn release_stream(stream: *mut c_void) {
    unsafe {
        if stream.is_null() {
            return;
        }
        let vtable = *(stream as *mut *mut usize);
        let release: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(vtable.add(2).read());
        release(stream);
    }
}
