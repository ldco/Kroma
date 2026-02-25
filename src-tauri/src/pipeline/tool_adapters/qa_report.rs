use std::path::Path;

use image::DynamicImage;
use serde_json::{json, Value};

use super::pathing::{list_image_files_recursive, path_for_output};

pub(super) fn build_output_guard_report_value(
    app_root: &Path,
    input_abs: &Path,
    max_chroma_delta: f64,
    enforce_grayscale: bool,
    fail_on_chroma_exceed: bool,
) -> Value {
    let input_display = path_for_output(app_root, input_abs);
    let mut report = json!({
        "input": input_display,
        "settings": {
            "max_chroma_delta": max_chroma_delta.max(0.0),
            "enforce_grayscale": enforce_grayscale,
            "fail_on_chroma_exceed": fail_on_chroma_exceed,
        },
        "summary": {
            "total_files": 0_u64,
            "hard_failures": 0_u64,
            "soft_warnings": 0_u64,
        },
        "files": []
    });

    if !input_abs.exists() {
        report["summary"]["hard_failures"] = json!(1_u64);
        report["files"] = json!([{
            "file": path_for_output(app_root, input_abs),
            "error": "input_not_found",
            "hard_fail_reasons": ["input_not_found"],
            "soft_warnings": []
        }]);
        return report;
    }

    let images = match list_image_files_recursive(input_abs) {
        Ok(v) => v,
        Err(err) => {
            report["summary"]["hard_failures"] = json!(1_u64);
            report["files"] = json!([{
                "file": path_for_output(app_root, input_abs),
                "error": err.to_string(),
                "hard_fail_reasons": ["image_read_failed"],
                "soft_warnings": []
            }]);
            return report;
        }
    };
    report["summary"]["total_files"] = json!(images.len() as u64);
    if images.is_empty() {
        report["summary"]["hard_failures"] = json!(1_u64);
        report["files"] = json!([{
            "file": path_for_output(app_root, input_abs),
            "error": "no_images_found",
            "hard_fail_reasons": ["no_images_found"],
            "soft_warnings": []
        }]);
        return report;
    }

    let mut hard_failures = 0_u64;
    let mut soft_warnings = 0_u64;
    let mut file_entries = Vec::<Value>::with_capacity(images.len());

    for img_path in images {
        let file_display = path_for_output(app_root, img_path.as_path());
        match compute_chroma_delta_from_image_path(img_path.as_path()) {
            Ok(chroma) => {
                let grayscale_like = chroma <= max_chroma_delta;
                let mut hard = Vec::<&str>::new();
                let mut soft = Vec::<&str>::new();
                if enforce_grayscale && !grayscale_like {
                    hard.push("not_grayscale_like");
                }
                if chroma > max_chroma_delta {
                    if fail_on_chroma_exceed {
                        hard.push("chroma_exceeds_threshold");
                    } else {
                        soft.push("chroma_exceeds_threshold");
                    }
                }
                if !hard.is_empty() {
                    hard_failures += 1;
                }
                if !soft.is_empty() {
                    soft_warnings += 1;
                }
                file_entries.push(json!({
                    "file": file_display,
                    "chroma_delta": round_to_4(chroma),
                    "grayscale_like": grayscale_like,
                    "hard_fail_reasons": hard,
                    "soft_warnings": soft,
                }));
            }
            Err(err) => {
                hard_failures += 1;
                file_entries.push(json!({
                    "file": file_display,
                    "chroma_delta": Value::Null,
                    "grayscale_like": Value::Null,
                    "error": err.to_string(),
                    "hard_fail_reasons": ["image_read_failed"],
                    "soft_warnings": [],
                }));
            }
        }
    }

    report["summary"]["hard_failures"] = json!(hard_failures);
    report["summary"]["soft_warnings"] = json!(soft_warnings);
    report["files"] = Value::Array(file_entries);
    report
}

fn compute_chroma_delta_from_image_path(path: &Path) -> Result<f64, image::ImageError> {
    let image = image::open(path)?;
    Ok(compute_chroma_delta_for_image(&image))
}

fn compute_chroma_delta_for_image(image: &DynamicImage) -> f64 {
    let rgb = image.to_rgb8();
    let mut rg_sum = 0.0_f64;
    let mut rb_sum = 0.0_f64;
    let mut gb_sum = 0.0_f64;
    let mut count = 0_u64;
    for pixel in rgb.pixels() {
        let [r, g, b] = pixel.0;
        rg_sum += f64::from((i16::from(r) - i16::from(g)).abs());
        rb_sum += f64::from((i16::from(r) - i16::from(b)).abs());
        gb_sum += f64::from((i16::from(g) - i16::from(b)).abs());
        count += 1;
    }
    if count == 0 {
        return 0.0;
    }
    ((rg_sum / count as f64) + (rb_sum / count as f64) + (gb_sum / count as f64)) / 3.0
}

fn round_to_4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}
