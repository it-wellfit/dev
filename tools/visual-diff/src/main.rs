//! 視覺回歸比對（計畫 1793782615513 §7.3）。
//!
//! 用法：`visual-diff <baseline.png> <candidate.png> [--threshold 0.005] [--out diff.png]`
//! 語義：逐像素 RGBA 精確比對；差異比例 > 閾值 → exit 1（阻擋合併）。
//! 輸出：`diff_ratio`、差異像素數；`--out` 時產生紅標差異圖（供 review）。

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use std::path::PathBuf;

/// 逐像素比對（RGBA 精確相等）；回傳（差異像素數，比例）。
/// 尺寸不一致回傳 Err（視覺回歸無法比較——視為候選圖錯誤）。
pub fn diff_pixels(
    baseline: &DynamicImage,
    candidate: &DynamicImage,
) -> Result<(u64, f64), String> {
    let (bw, bh) = baseline.dimensions();
    let (cw, ch) = candidate.dimensions();
    if (bw, bh) != (cw, ch) {
        return Err(format!(
            "dimension mismatch: baseline {bw}x{bh} vs candidate {cw}x{ch}"
        ));
    }
    let mut diff = 0u64;
    for y in 0..bh {
        for x in 0..bw {
            if baseline.get_pixel(x, y) != candidate.get_pixel(x, y) {
                diff += 1;
            }
        }
    }
    let total = (bw as u64) * (bh as u64);
    let ratio = if total == 0 {
        0.0
    } else {
        diff as f64 / total as f64
    };
    Ok((diff, ratio))
}

/// 產生差異標記圖：差異像素標紅（255,0,0,255），其餘半透明灰。
pub fn render_diff(baseline: &DynamicImage, candidate: &DynamicImage) -> RgbaImage {
    let (w, h) = baseline.dimensions();
    let mut out: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let b = baseline.get_pixel(x, y);
            let c = candidate.get_pixel(x, y);
            let pixel = if b != c {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([128, 128, 128, 64])
            };
            out.put_pixel(x, y, pixel);
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "用法：visual-diff <baseline.png> <candidate.png> [--threshold 0.005] [--out diff.png]"
        );
        std::process::exit(2);
    }
    let baseline_path = PathBuf::from(&args[1]);
    let candidate_path = PathBuf::from(&args[2]);
    let mut threshold: f64 = 0.005;
    let mut out_path: Option<PathBuf> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--threshold" if i + 1 < args.len() => {
                threshold = args[i + 1].parse().unwrap_or(0.005);
                i += 1;
            }
            "--out" if i + 1 < args.len() => {
                out_path = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            other => {
                eprintln!("未知參數：{other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let baseline = match image::open(&baseline_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("無法讀取 baseline {}: {e}", baseline_path.display());
            std::process::exit(2);
        }
    };
    let candidate = match image::open(&candidate_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("無法讀取 candidate {}: {e}", candidate_path.display());
            std::process::exit(2);
        }
    };

    let (diff_count, ratio) = match diff_pixels(&baseline, &candidate) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    if let Some(path) = out_path {
        let diff = render_diff(&baseline, &candidate);
        if let Err(e) = diff.save(&path) {
            eprintln!("無法寫出差異圖 {}: {e}", path.display());
        }
    }

    println!("diff_ratio={ratio:.4} threshold={threshold:.4} diff_pixels={diff_count}");
    if ratio > threshold {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    fn solid(w: u32, h: u32, px: Rgba<u8>) -> DynamicImage {
        let buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(w, h, |_, _| px);
        DynamicImage::ImageRgba8(buf)
    }

    #[test]
    fn identical_images_have_zero_diff() {
        let a = solid(4, 4, Rgba([1, 2, 3, 255]));
        let b = solid(4, 4, Rgba([1, 2, 3, 255]));
        let (diff, ratio) = diff_pixels(&a, &b).unwrap();
        assert_eq!(diff, 0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn fully_different_images_max_diff() {
        let a = solid(4, 4, Rgba([0, 0, 0, 255]));
        let b = solid(4, 4, Rgba([255, 255, 255, 255]));
        let (diff, ratio) = diff_pixels(&a, &b).unwrap();
        assert_eq!(diff, 16);
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dimension_mismatch_is_error() {
        let a = solid(4, 4, Rgba([0, 0, 0, 255]));
        let b = solid(2, 2, Rgba([0, 0, 0, 255]));
        let err = diff_pixels(&a, &b).unwrap_err();
        assert!(err.contains("dimension mismatch"));
    }

    #[test]
    fn render_diff_marks_changed_pixels_red() {
        let a = solid(2, 1, Rgba([0, 0, 0, 255]));
        let b = solid(2, 1, Rgba([0, 0, 0, 255]));
        let _ = render_diff(&a, &b);
        let c = solid(2, 1, Rgba([9, 9, 9, 255]));
        let diff = render_diff(&a, &c);
        assert_eq!(diff.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(diff.get_pixel(1, 0), &Rgba([255, 0, 0, 255]));
    }
}
