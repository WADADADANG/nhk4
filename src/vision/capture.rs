use chrono::Local;
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::path::{Path, PathBuf};

/// Crop image based on designated screen region
pub fn process_region(img: DynamicImage, region: &str) -> DynamicImage {
    let (img_w, img_h) = (img.width(), img.height());
    match region {
        "right" => {
            let left = (img_w as f32 * 0.45) as u32;
            let w = img_w.saturating_sub(left);
            img.crop_imm(left, 0, w, img_h)
        }
        "left" => {
            let w = (img_w as f32 * 0.52) as u32;
            img.crop_imm(0, 0, w, img_h)
        }
        "party" => {
            let left = (img_w as f32 * 0.55) as u32;
            let w = img_w.saturating_sub(left);
            let h = img_h.min(650);
            img.crop_imm(left, 0, w, h)
        }
        _ => img,
    }
}

/// Generate standardized screenshot filename and destination path
pub fn generate_screenshot_path(
    base_dir: &Path,
    subfolder: &str,
    prefix: &str,
    client_id: u32,
) -> Result<(PathBuf, String), String> {
    let clean_sub = subfolder.trim().replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_");
    let safe_sub = if clean_sub.is_empty() {
        format!("client_{}", client_id)
    } else {
        clean_sub
    };

    let clean_prefix = prefix.trim().replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_");
    let safe_prefix = if clean_prefix.is_empty() { "snap" } else { &clean_prefix };

    let dir = base_dir.join("screenshots").join(&safe_sub);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create screenshot dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_c{}_{}_{}.jpg", client_id, safe_prefix, timestamp);
    let filepath = dir.join(&filename);

    Ok((filepath, filename))
}

/// Process and save screenshot bytes received from CDP or memory
pub fn save_and_process_screenshot(
    raw_bytes: &[u8],
    base_dir: &Path,
    subfolder: &str,
    prefix: &str,
    client_id: u32,
    region: &str,
    _annotate: bool,
) -> Result<(PathBuf, String), String> {
    let (filepath, filename) = generate_screenshot_path(base_dir, subfolder, prefix, client_id)?;

    if region == "full" {
        fs::write(&filepath, raw_bytes).map_err(|e| format!("Failed to write screenshot file: {}", e))?;
    } else {
        let img = image::load_from_memory(raw_bytes).map_err(|e| format!("Image decode error: {}", e))?;
        let cropped = process_region(img, region);
        let mut file = fs::File::create(&filepath).map_err(|e| format!("Failed to create file: {}", e))?;
        cropped.write_to(&mut file, ImageFormat::Jpeg).map_err(|e| format!("Image encode error: {}", e))?;
    }

    Ok((filepath, filename))
}

/// Capture primary Windows Desktop screen using Win32 GDI BitBlt
#[cfg(windows)]
pub fn capture_desktop_screen() -> Result<DynamicImage, String> {
    use winapi::um::wingdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
    };
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        if width <= 0 || height <= 0 {
            return Err("Invalid screen dimensions".to_string());
        }

        let hdc_screen = winapi::um::wingdi::CreateDCA(b"DISPLAY\0".as_ptr() as _, std::ptr::null(), std::ptr::null(), std::ptr::null());
        if hdc_screen.is_null() {
            return Err("CreateDCA failed".to_string());
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() {
            DeleteDC(hdc_screen);
            return Err("CreateCompatibleDC failed".to_string());
        }

        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_null() {
            DeleteDC(hdc_mem);
            DeleteDC(hdc_screen);
            return Err("CreateCompatibleBitmap failed".to_string());
        }

        let old_obj = SelectObject(hdc_mem, hbitmap as _);
        let blt_res = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);
        if blt_res == 0 {
            let err = winapi::um::errhandlingapi::GetLastError();
            SelectObject(hdc_mem, old_obj);
            DeleteObject(hbitmap as _);
            DeleteDC(hdc_mem);
            DeleteDC(hdc_screen);
            return Err(format!("BitBlt failed with code {}", err));
        }

        let mut bi: BITMAPINFO = std::mem::zeroed();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = width;
        bi.bmiHeader.biHeight = -height; // Top-down DIB
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;

        let mut raw_pixels = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            raw_pixels.as_mut_ptr() as _,
            &mut bi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_obj);
        DeleteObject(hbitmap as _);
        DeleteDC(hdc_mem);
        DeleteDC(hdc_screen);

        if lines == 0 {
            return Err("GetDIBits failed".to_string());
        }

        // Convert BGRA (Windows DIB format) to RGBA
        for chunk in raw_pixels.chunks_exact_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
            chunk[3] = 255;
        }

        let img_buf = image::RgbaImage::from_raw(width as u32, height as u32, raw_pixels)
            .ok_or_else(|| "Failed to construct RgbaImage from pixels".to_string())?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    }
}

#[cfg(not(windows))]
pub fn capture_desktop_screen() -> Result<DynamicImage, String> {
    Err("Desktop screen capture fallback is only supported on Windows".to_string())
}

/// Fallback: Capture primary desktop screen and save to disk
pub fn capture_desktop_and_save(
    base_dir: &Path,
    subfolder: &str,
    prefix: &str,
    client_id: u32,
    region: &str,
    _annotate: bool,
) -> Result<(PathBuf, String), String> {
    let img = capture_desktop_screen()?;
    let cropped = process_region(img, region);
    let (filepath, filename) = generate_screenshot_path(base_dir, subfolder, prefix, client_id)?;

    let mut file = fs::File::create(&filepath).map_err(|e| format!("Failed to create file: {}", e))?;
    cropped.write_to(&mut file, ImageFormat::Jpeg).map_err(|e| format!("Image encode error: {}", e))?;

    Ok((filepath, filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_screenshot_path() {
        let temp_dir = std::env::temp_dir().join("nhk_snap_test");
        let (path, name) = generate_screenshot_path(&temp_dir, "client_1", "error_snap", 1).unwrap();
        assert!(name.starts_with("screenshot_c1_error_snap_"));
        assert!(name.ends_with(".jpg"));
        assert!(path.to_string_lossy().contains("client_1"));
    }

    #[test]
    fn test_process_region_crop() {
        let dummy = DynamicImage::new_rgb8(1000, 800);
        let cropped_right = process_region(dummy.clone(), "right");
        assert_eq!(cropped_right.width(), 550);
        assert_eq!(cropped_right.height(), 800);

        let cropped_left = process_region(dummy.clone(), "left");
        assert_eq!(cropped_left.width(), 520);
        assert_eq!(cropped_left.height(), 800);

        let cropped_party = process_region(dummy.clone(), "party");
        assert_eq!(cropped_party.width(), 450);
        assert_eq!(cropped_party.height(), 650);

        let full = process_region(dummy.clone(), "full");
        assert_eq!(full.width(), 1000);
        assert_eq!(full.height(), 800);
    }

    #[test]
    fn test_save_and_process_screenshot_flow() {
        let temp_dir = std::env::temp_dir().join("nhk_save_snap_test");
        let dummy = DynamicImage::new_rgb8(800, 600);
        let mut jpeg_bytes = Vec::new();
        dummy.write_to(&mut std::io::Cursor::new(&mut jpeg_bytes), ImageFormat::Jpeg).unwrap();

        let (path, name) = save_and_process_screenshot(
            &jpeg_bytes,
            &temp_dir,
            "client_1",
            "test_snap",
            1,
            "full",
            true,
        ).unwrap();

        assert!(path.exists());
        assert!(name.starts_with("screenshot_c1_test_snap_"));
        assert!(path.metadata().unwrap().len() > 100);

        // Test crop save
        let (crop_path, crop_name) = save_and_process_screenshot(
            &jpeg_bytes,
            &temp_dir,
            "client_1",
            "test_crop",
            1,
            "party",
            true,
        ).unwrap();
        assert!(crop_path.exists());
        assert!(crop_name.starts_with("screenshot_c1_test_crop_"));
    }

    #[test]
    #[cfg(windows)]
    fn test_desktop_screen_capture_live() {
        match capture_desktop_screen() {
            Ok(img) => {
                assert!(img.width() >= 640);
                assert!(img.height() >= 480);
            }
            Err(e) => {
                eprintln!("Desktop capture test notice: {}", e);
            }
        }
    }
}
