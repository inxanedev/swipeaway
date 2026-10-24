use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, GetObjectW, HDC, HGDIOBJ,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_RESIZETOFIT,
    SIIGBF_THUMBNAILONLY,
};
use windows::core::PCWSTR;

use super::Bitmap;

#[derive(Debug)]
pub struct ShellThumb {
    pub bitmap: Bitmap,
    pub is_icon: bool,
}

struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> Self {
        let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok() };
        Self { initialized }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

pub fn thumbnail(path: &Path, size: u32) -> Option<ShellThumb> {
    let _com = ComGuard::new();
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let size = SIZE {
        cx: size.clamp(32, 2048) as i32,
        cy: size.clamp(32, 2048) as i32,
    };

    unsafe {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None).ok()?;

        let (handle, is_icon) = match factory.GetImage(
            size,
            SIIGBF_BIGGERSIZEOK | SIIGBF_THUMBNAILONLY | SIIGBF_RESIZETOFIT,
        ) {
            Ok(handle) => (handle, false),
            Err(_) => (
                factory
                    .GetImage(size, SIIGBF_BIGGERSIZEOK | SIIGBF_RESIZETOFIT)
                    .ok()?,
                true,
            ),
        };

        if handle.is_invalid() {
            return None;
        }

        let bitmap = hbitmap_to_bitmap(handle);
        let _ = DeleteObject(HGDIOBJ(handle.0));
        bitmap.map(|bitmap| ShellThumb { bitmap, is_icon })
    }
}

unsafe fn hbitmap_to_bitmap(handle: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<Bitmap> {
    let mut info = BITMAP::default();
    let read = unsafe {
        GetObjectW(
            HGDIOBJ(handle.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut info as *mut _ as *mut c_void),
        )
    };
    if read == 0 || info.bmWidth <= 0 || info.bmHeight <= 0 {
        return None;
    }

    let width = info.bmWidth as u32;
    let height = info.bmHeight as u32;
    if width > 4096 || height > 4096 {
        return None;
    }

    let hdc: HDC = unsafe { CreateCompatibleDC(None) };
    if hdc.is_invalid() {
        return None;
    }

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut buffer = vec![0u8; (width * height * 4) as usize];
    let copied = unsafe {
        GetDIBits(
            hdc,
            handle,
            0,
            height,
            Some(buffer.as_mut_ptr() as *mut c_void),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        let _ = DeleteDC(hdc);
    }

    if copied == 0 {
        return None;
    }

    let has_alpha = buffer.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0);
    for pixel in buffer.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
        if !has_alpha {
            pixel[3] = 255;
        }
    }

    Some(Bitmap::new(width, height, buffer))
}
