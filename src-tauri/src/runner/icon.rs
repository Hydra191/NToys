#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use image::codecs::webp::WebPEncoder;
use image::ImageEncoder;


#[link(name = "Shell32")]
extern "system" {
    fn SHGetFileInfoW(
        pszPath: *const u16,
        dwFileAttributes: u32,
        psfi: *mut SHFILEINFOW,
        cbFileInfo: u32,
        uFlags: u32,
    ) -> usize;
}

#[link(name = "User32")]
extern "system" {
    fn GetIconInfo(hIcon: isize, piconinfo: *mut ICONINFO) -> i32;
    fn DestroyIcon(hIcon: isize) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
}

#[link(name = "Gdi32")]
extern "system" {
    fn GetDIBits(
        hdc: isize,
        hbm: isize,
        start: u32,
        cLines: u32,
        lpvBits: *mut u8,
        lpbmi: *mut BITMAPINFOHEADER,
        usage: u32,
    ) -> i32;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn DeleteDC(hdc: isize) -> i32;
    fn DeleteObject(ho: isize) -> i32;
    fn CreateDIBSection(
        hdc: isize,
        lpbmi: *const BITMAPINFOHEADER,
        usage: u32,
        ppvBits: *mut *mut u8,
        hSection: isize,
        offset: u32,
    ) -> isize;
    fn SelectObject(hdc: isize, h: isize) -> isize;
    fn StretchBlt(
        hdcDest: isize,
        xDest: i32,
        yDest: i32,
        wDest: i32,
        hDest: i32,
        hdcSrc: isize,
        xSrc: i32,
        ySrc: i32,
        wSrc: i32,
        hSrc: i32,
        rop: u32,
    ) -> i32;
}

const SHGFI_ICON: u32 = 0x100;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const SRCCOPY: u32 = 0xCC0020;
const SM_CXICON: i32 = 11;
const SM_CYICON: i32 = 12;

#[repr(C)]
struct SHFILEINFOW {
    hIcon: isize,
    iIcon: i32,
    dwAttributes: u32,
    szDisplayName: [u16; 260],
    szTypeName: [u16; 80],
}

#[repr(C)]
struct ICONINFO {
    fIcon: i32,
    xHotspot: u32,
    yHotspot: u32,
    hbmMask: isize,
    hbmColor: isize,
}

#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn hicon_to_webp(hIcon: isize) -> Option<Vec<u8>> {
    unsafe {
        let mut ii = ICONINFO {
            fIcon: 0,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: 0,
            hbmColor: 0,
        };
        if GetIconInfo(hIcon, &mut ii) == 0 {
            return None;
        }

        let w = 32;
        let h = 32;
        let bmi = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let dc = CreateCompatibleDC(0);
        if dc == 0 {
            DeleteObject(ii.hbmColor);
            DeleteObject(ii.hbmMask);
            DestroyIcon(hIcon);
            return None;
        }

        let mut bits: *mut u8 = std::ptr::null_mut();
        let dib = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, 0, 0);
        if dib == 0 || bits.is_null() {
            DeleteDC(dc);
            DeleteObject(ii.hbmColor);
            DeleteObject(ii.hbmMask);
            DestroyIcon(hIcon);
            return None;
        }

        let old_bmp = SelectObject(dc, dib);
        let old_dc = CreateCompatibleDC(0);
        let old_icon = SelectObject(old_dc, ii.hbmColor);

        let icon_w = GetSystemMetrics(SM_CXICON);
        let icon_h = GetSystemMetrics(SM_CYICON);
        StretchBlt(dc, 0, 0, w, h, old_dc, 0, 0, icon_w, icon_h, SRCCOPY);

        SelectObject(dc, old_bmp);
        SelectObject(old_dc, old_icon);
        DeleteDC(old_dc);

        let mut pixels: Vec<u8> = vec![0u8; (w * h * 4) as usize];
        let mut bmi_flip = bmi;
        bmi_flip.biHeight = -h;
        let result = GetDIBits(
            dc,
            dib,
            0,
            h as u32,
            pixels.as_mut_ptr(),
            &mut bmi_flip,
            DIB_RGB_COLORS,
        );

        DeleteObject(SelectObject(dc, old_bmp));
        DeleteDC(dc);
        DeleteObject(ii.hbmColor);
        DeleteObject(ii.hbmMask);
        DestroyIcon(hIcon);

        if result == 0 {
            return None;
        }

        // BGRA -> RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let mut webp = Vec::new();
        let encoder = WebPEncoder::new_lossless(&mut webp);
        encoder.write_image(&pixels, w as u32, h as u32, image::ExtendedColorType::Rgba8)
            .ok()?;
        Some(webp)
    }
}

/// Extract the icon from a file path and return WebP bytes
pub fn extract_icon(path: &str) -> Option<Vec<u8>> {
    let wide = to_wide(path);
    let mut info = SHFILEINFOW {
        hIcon: 0,
        iIcon: 0,
        dwAttributes: 0,
        szDisplayName: [0; 260],
        szTypeName: [0; 80],
    };

    unsafe {
        let result = SHGetFileInfoW(
            wide.as_ptr(),
            FILE_ATTRIBUTE_NORMAL,
            &mut info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON,
        );
        if result == 0 || info.hIcon == 0 {
            return None;
        }

        hicon_to_webp(info.hIcon)
    }
}

/// Get display name from .lnk file path
pub fn parse_lnk_info(lnk_path: &Path) -> (String, String) {
    let name = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    (name, String::new())
}
