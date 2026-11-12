use std::{mem::size_of, sync::OnceLock};

use ::windows::{
    core::HSTRING,
    Globalization::Language,
    Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::DataWriter,
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            DeleteDC, DeleteObject, GetDIBits, GetWindowDC, ReleaseDC, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS},
        UI::WindowsAndMessaging::{
            GetWindowRect, IsIconic, IsWindowVisible, ShowWindow, PW_RENDERFULLCONTENT, SW_HIDE,
            SW_MINIMIZE, SW_SHOWNOACTIVATE,
        },
    },
};
use regex::Regex;

const PROGRESS_CROP_HEIGHT: i32 = 70;
const OCR_SCALE: i32 = 3;

pub(crate) fn parse_netease_progress_text(text: &str) -> Option<(i64, i64)> {
    let normalized = text.replace('：', ":");
    let captures = progress_pattern().captures(&normalized)?;
    let position = timestamp_millis(captures.get(1)?.as_str(), captures.get(2)?.as_str())?;
    let duration = timestamp_millis(captures.get(3)?.as_str(), captures.get(4)?.as_str())?;
    (duration > 0 && position <= duration.saturating_add(5_000)).then_some((position, duration))
}

pub(crate) fn netease_progress_crop(width: i32, height: i32) -> (i32, i32, i32, i32) {
    (
        0,
        (height - PROGRESS_CROP_HEIGHT).max(0),
        width.max(0),
        height.max(0),
    )
}

pub(crate) fn netease_time_label_crops(width: i32, height: i32) -> [(i32, i32, i32, i32); 2] {
    [
        (
            0,
            (height - 60).max(0),
            (width * 180 / 1_057).max(1),
            height.max(0),
        ),
        (
            (width - width * 180 / 1_057).max(0),
            (height - 60).max(0),
            width.max(0),
            height.max(0),
        ),
    ]
}

pub(crate) fn dib_source_row(height: i32, visual_row: i32) -> i32 {
    (height - 1 - visual_row).max(0)
}

pub(crate) fn capture_netease_timeline(hwnd: HWND) -> Option<(i64, i64)> {
    unsafe {
        let was_visible = IsWindowVisible(hwnd).as_bool();
        let was_iconic = IsIconic(hwnd).as_bool();
        if !was_visible || was_iconic {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            std::thread::sleep(std::time::Duration::from_millis(120));
        }

        let result = capture_and_recognize(hwnd);

        if was_iconic {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        } else if !was_visible {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        result
    }
}

unsafe fn capture_and_recognize(hwnd: HWND) -> Option<(i64, i64)> {
    unsafe {
        let mut rect = Default::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return None;
        }

        let window_dc = GetWindowDC(Some(hwnd));
        if window_dc.is_invalid() {
            return None;
        }
        let memory_dc = ::windows::Win32::Graphics::Gdi::CreateCompatibleDC(Some(window_dc));
        let bitmap =
            ::windows::Win32::Graphics::Gdi::CreateCompatibleBitmap(window_dc, width, height);
        let previous = SelectObject(memory_dc, bitmap.into());
        let printed =
            PrintWindow(hwnd, memory_dc, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool();

        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let copied = if printed {
            GetDIBits(
                window_dc,
                bitmap,
                0,
                height as u32,
                Some(pixels.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            )
        } else {
            0
        };

        if !previous.is_invalid() {
            SelectObject(memory_dc, previous);
        }
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory_dc);
        let _ = ReleaseDC(Some(hwnd), window_dc);

        if copied <= 0 {
            return None;
        }

        let progress_crop = netease_progress_crop(width, height);
        let scaled = scale_region(&pixels, width, height, progress_crop, OCR_SCALE)?;
        let recognized = recognize_progress_text(scaled.0, scaled.1, scaled.2);
        if let Some(value) = recognized.as_deref().and_then(parse_netease_progress_text) {
            return Some(value);
        }

        let [left_crop, right_crop] = netease_time_label_crops(width, height);
        let left = scale_region(&pixels, width, height, left_crop, 6).and_then(
            |(scaled_width, scaled_height, mut pixels)| {
                threshold_pixels(&mut pixels);
                recognize_progress_text(scaled_width, scaled_height, pixels)
            },
        );
        let right = scale_region(&pixels, width, height, right_crop, 6).and_then(
            |(scaled_width, scaled_height, mut pixels)| {
                threshold_pixels(&mut pixels);
                recognize_progress_text(scaled_width, scaled_height, pixels)
            },
        );
        let combined = format!(
            "{} / {}",
            left.unwrap_or_default(),
            right.unwrap_or_default()
        );
        parse_netease_progress_text(&combined)
    }
}

fn scale_region(
    pixels: &[u8],
    width: i32,
    height: i32,
    crop: (i32, i32, i32, i32),
    scale: i32,
) -> Option<(i32, i32, Vec<u8>)> {
    let (left, top, right, bottom) = crop;
    let crop_width = right - left;
    let crop_height = bottom - top;
    if crop_width <= 0 || crop_height <= 0 {
        return None;
    }
    let scaled_width = crop_width * scale;
    let scaled_height = crop_height * scale;
    let mut scaled = vec![0u8; scaled_width as usize * scaled_height as usize * 4];

    for y in 0..crop_height {
        for x in 0..crop_width {
            let source_row = dib_source_row(height, top + y);
            let source = (source_row * width + left + x) as usize * 4;
            for scale_y in 0..scale {
                for scale_x in 0..scale {
                    let target_x = x * scale + scale_x;
                    let target_y = y * scale + scale_y;
                    let target = (target_y * scaled_width + target_x) as usize * 4;
                    scaled[target..target + 4].copy_from_slice(&pixels[source..source + 4]);
                }
            }
        }
    }
    Some((scaled_width, scaled_height, scaled))
}

fn threshold_pixels(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let luminance =
            (pixel[2] as u32 * 299 + pixel[1] as u32 * 587 + pixel[0] as u32 * 114) / 1_000;
        let value = if luminance >= 105 { 255 } else { 0 };
        pixel.copy_from_slice(&[value, value, value, 255]);
    }
}

fn recognize_progress_text(width: i32, height: i32, pixels: Vec<u8>) -> Option<String> {
    let writer = DataWriter::new().ok()?;
    writer.WriteBytes(&pixels).ok()?;
    let buffer = writer.DetachBuffer().ok()?;
    let bitmap =
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, width, height)
            .ok()?;
    let alpha_bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        width,
        height,
        BitmapAlphaMode::Ignore,
    )
    .or_else(|_| Ok::<_, ::windows::core::Error>(bitmap.clone()))
    .ok()?;
    let engine = OcrEngine::TryCreateFromLanguage(
        &Language::CreateLanguage(&HSTRING::from("zh-Hans")).ok()?,
    )
    .or_else(|_| OcrEngine::TryCreateFromUserProfileLanguages())
    .ok()?;
    let result = engine.RecognizeAsync(&alpha_bitmap).ok()?.get().ok()?;
    let text = result.Text().ok()?.to_string();
    let _ = bitmap.Close();
    let _ = alpha_bitmap.Close();
    Some(text)
}

fn timestamp_millis(minutes: &str, seconds: &str) -> Option<i64> {
    let minutes = minutes.parse::<i64>().ok()?;
    let seconds = seconds.parse::<i64>().ok()?;
    (seconds < 60).then_some(minutes * 60_000 + seconds * 1_000)
}

fn progress_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(\d{1,3})\s*:\s*(\d{2})\s*/\s*(\d{1,3})\s*:\s*(\d{2})")
            .expect("valid Netease progress OCR regex")
    })
}

pub(crate) struct TimelineFailureTracker {
    failures: u8,
    estimated: bool,
}

impl TimelineFailureTracker {
    pub(crate) fn new() -> Self {
        Self {
            failures: 0,
            estimated: false,
        }
    }

    pub(crate) fn record_failure(&mut self) -> bool {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= 3 && !self.estimated {
            self.estimated = true;
            return true;
        }
        false
    }

    pub(crate) fn record_success(&mut self) {
        self.failures = 0;
        self.estimated = false;
    }

    pub(crate) fn is_estimated(&self) -> bool {
        self.estimated
    }
}
