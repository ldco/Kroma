use std::fs;
use std::path::{Path, PathBuf};

use image::{DynamicImage, RgbImage};
use serde_json::Value;

use super::pathing::is_image_path;

#[derive(Debug, Clone)]
pub(super) struct ColorProfileConfig {
    brightness: f32,
    contrast: f32,
    saturation: f32,
    sharpness: f32,
    gamma: f32,
    autocontrast_cutoff: f32,
    red_multiplier: f32,
    green_multiplier: f32,
    blue_multiplier: f32,
}

impl Default for ColorProfileConfig {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            sharpness: 1.0,
            gamma: 1.0,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.0,
            green_multiplier: 1.0,
            blue_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ColorSettingsConfig {
    pub(super) profiles: std::collections::BTreeMap<String, ColorProfileConfig>,
}

pub(super) fn load_color_settings_config(
    settings_path: Option<&Path>,
) -> Result<ColorSettingsConfig, String> {
    let Some(path) = settings_path else {
        return Ok(default_color_settings_config());
    };
    let raw =
        fs::read_to_string(path).map_err(|e| format!("read settings '{}': {e}", path.display()))?;
    let parsed: Value = serde_json::from_str(raw.as_str())
        .map_err(|e| format!("parse settings '{}': {e}", path.display()))?;
    let profiles_obj = parsed
        .get("profiles")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "settings file '{}' is missing object field 'profiles'",
                path.display()
            )
        })?;
    let mut profiles = std::collections::BTreeMap::new();
    for (name, value) in profiles_obj {
        let obj = value.as_object().ok_or_else(|| {
            format!(
                "profile '{}' in '{}' must be an object",
                name,
                path.display()
            )
        })?;
        profiles.insert(name.clone(), parse_color_profile_config(obj));
    }
    Ok(ColorSettingsConfig { profiles })
}

fn parse_color_profile_config(obj: &serde_json::Map<String, Value>) -> ColorProfileConfig {
    let mut cfg = ColorProfileConfig::default();
    cfg.brightness = obj
        .get("brightness")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.brightness);
    cfg.contrast = obj
        .get("contrast")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.contrast);
    cfg.saturation = obj
        .get("saturation")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.saturation);
    cfg.sharpness = obj
        .get("sharpness")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.sharpness);
    cfg.gamma = obj
        .get("gamma")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.gamma);
    cfg.autocontrast_cutoff = obj
        .get("autocontrast_cutoff")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.autocontrast_cutoff);
    cfg.red_multiplier = obj
        .get("red_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.red_multiplier);
    cfg.green_multiplier = obj
        .get("green_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.green_multiplier);
    cfg.blue_multiplier = obj
        .get("blue_multiplier")
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .unwrap_or(cfg.blue_multiplier);
    cfg
}

fn default_color_settings_config() -> ColorSettingsConfig {
    let mut profiles = std::collections::BTreeMap::new();
    profiles.insert(
        String::from("neutral"),
        ColorProfileConfig {
            brightness: 1.0,
            contrast: 1.02,
            saturation: 1.0,
            sharpness: 1.0,
            gamma: 1.0,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.0,
            green_multiplier: 1.0,
            blue_multiplier: 1.0,
        },
    );
    profiles.insert(
        String::from("cinematic_warm"),
        ColorProfileConfig {
            brightness: 0.98,
            contrast: 1.12,
            saturation: 1.08,
            sharpness: 1.04,
            gamma: 1.03,
            autocontrast_cutoff: 0.0,
            red_multiplier: 1.04,
            green_multiplier: 1.0,
            blue_multiplier: 0.96,
        },
    );
    profiles.insert(
        String::from("cold_rain"),
        ColorProfileConfig {
            brightness: 0.97,
            contrast: 1.1,
            saturation: 0.95,
            sharpness: 1.03,
            gamma: 1.01,
            autocontrast_cutoff: 0.0,
            red_multiplier: 0.97,
            green_multiplier: 1.0,
            blue_multiplier: 1.05,
        },
    );
    ColorSettingsConfig { profiles }
}

pub(super) fn apply_color_profile_to_path(
    input_abs: &Path,
    output_abs: &Path,
    profile: &ColorProfileConfig,
) -> Result<(), String> {
    let image =
        image::open(input_abs).map_err(|e| format!("open '{}': {e}", input_abs.display()))?;
    let corrected = apply_color_profile_to_image(&image, profile);
    if let Some(parent) = output_abs.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir '{}': {e}", parent.display()))?;
    }
    corrected
        .save(output_abs)
        .map_err(|e| format!("save '{}': {e}", output_abs.display()))
}

fn apply_color_profile_to_image(
    image: &DynamicImage,
    profile: &ColorProfileConfig,
) -> DynamicImage {
    let mut out = image.to_rgb8();

    if profile.autocontrast_cutoff > 0.0 {
        apply_autocontrast_in_place(&mut out, profile.autocontrast_cutoff);
    }
    if (profile.brightness - 1.0).abs() > f32::EPSILON {
        apply_brightness_in_place(&mut out, profile.brightness);
    }
    if (profile.contrast - 1.0).abs() > f32::EPSILON {
        apply_contrast_in_place(&mut out, profile.contrast);
    }
    if (profile.saturation - 1.0).abs() > f32::EPSILON {
        apply_saturation_in_place(&mut out, profile.saturation);
    }
    if (profile.sharpness - 1.0).abs() > f32::EPSILON {
        out = apply_sharpness(&out, profile.sharpness);
    }
    if profile.gamma > 0.0 && (profile.gamma - 1.0).abs() > f32::EPSILON {
        apply_gamma_in_place(&mut out, profile.gamma);
    }
    if (profile.red_multiplier - 1.0).abs() > f32::EPSILON
        || (profile.green_multiplier - 1.0).abs() > f32::EPSILON
        || (profile.blue_multiplier - 1.0).abs() > f32::EPSILON
    {
        apply_rgb_multipliers_in_place(
            &mut out,
            profile.red_multiplier,
            profile.green_multiplier,
            profile.blue_multiplier,
        );
    }

    DynamicImage::ImageRgb8(out)
}

pub(super) fn resolve_color_output_path(
    src_abs: &Path,
    input_root_abs: &Path,
    output_abs: &Path,
    input_is_dir: bool,
) -> Result<PathBuf, std::io::Error> {
    if !input_is_dir {
        if is_image_path(output_abs) {
            if let Some(parent) = output_abs.parent() {
                fs::create_dir_all(parent)?;
            }
            return Ok(output_abs.to_path_buf());
        }
        fs::create_dir_all(output_abs)?;
        return Ok(output_abs.join(
            src_abs
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("image.png"),
        ));
    }

    let rel = src_abs.strip_prefix(input_root_abs).unwrap_or(src_abs);
    let dst = output_abs.join(rel);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(dst)
}

fn apply_autocontrast_in_place(image: &mut RgbImage, _cutoff: f32) {
    let mut min_v = [u8::MAX; 3];
    let mut max_v = [u8::MIN; 3];
    for pixel in image.pixels() {
        for i in 0..3 {
            min_v[i] = min_v[i].min(pixel[i]);
            max_v[i] = max_v[i].max(pixel[i]);
        }
    }
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            let minc = f32::from(min_v[i]);
            let maxc = f32::from(max_v[i]);
            if (maxc - minc).abs() < f32::EPSILON {
                continue;
            }
            let v = f32::from(pixel[i]);
            pixel[i] = clamp_u8(((v - minc) / (maxc - minc)) * 255.0);
        }
    }
}

fn apply_brightness_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            pixel[i] = clamp_u8(f32::from(pixel[i]) * factor);
        }
    }
}

fn apply_contrast_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        for i in 0..3 {
            let centered = f32::from(pixel[i]) - 128.0;
            pixel[i] = clamp_u8(centered * factor + 128.0);
        }
    }
}

fn apply_saturation_in_place(image: &mut RgbImage, factor: f32) {
    for pixel in image.pixels_mut() {
        let r = f32::from(pixel[0]);
        let g = f32::from(pixel[1]);
        let b = f32::from(pixel[2]);
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
        pixel[0] = clamp_u8(gray + (r - gray) * factor);
        pixel[1] = clamp_u8(gray + (g - gray) * factor);
        pixel[2] = clamp_u8(gray + (b - gray) * factor);
    }
}

fn apply_sharpness(image: &RgbImage, factor: f32) -> RgbImage {
    if factor <= 1.0 + f32::EPSILON {
        return image.clone();
    }
    let sigma = 1.0_f32;
    let blurred = image::imageops::blur(image, sigma);
    let amount = factor - 1.0;
    let mut out = image.clone();
    for (dst, (orig, blur)) in out.pixels_mut().zip(image.pixels().zip(blurred.pixels())) {
        for i in 0..3 {
            let val = f32::from(orig[i]) + amount * (f32::from(orig[i]) - f32::from(blur[i]));
            dst[i] = clamp_u8(val);
        }
    }
    out
}

fn apply_gamma_in_place(image: &mut RgbImage, gamma: f32) {
    let inv_gamma = 1.0 / gamma.max(f32::EPSILON);
    let mut lut = [0_u8; 256];
    for (i, out) in lut.iter_mut().enumerate() {
        *out = clamp_u8((((i as f32) / 255.0).powf(inv_gamma)) * 255.0);
    }
    for pixel in image.pixels_mut() {
        pixel[0] = lut[pixel[0] as usize];
        pixel[1] = lut[pixel[1] as usize];
        pixel[2] = lut[pixel[2] as usize];
    }
}

fn apply_rgb_multipliers_in_place(image: &mut RgbImage, r_mul: f32, g_mul: f32, b_mul: f32) {
    for pixel in image.pixels_mut() {
        pixel[0] = clamp_u8(f32::from(pixel[0]) * r_mul);
        pixel[1] = clamp_u8(f32::from(pixel[1]) * g_mul);
        pixel[2] = clamp_u8(f32::from(pixel[2]) * b_mul);
    }
}

fn clamp_u8(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    value.clamp(0.0, 255.0).round() as u8
}
