#!/usr/bin/env python3
import argparse
import json
from pathlib import Path
from PIL import Image, ImageEnhance, ImageOps

IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".tif", ".tiff"}
DEFAULT_COLOR_SETTINGS = {
    "default_profile": "neutral",
    "profiles": {
        "neutral": {
            "brightness": 1.0,
            "contrast": 1.02,
            "saturation": 1.0,
            "sharpness": 1.0,
            "gamma": 1.0,
            "autocontrast_cutoff": 0,
            "red_multiplier": 1.0,
            "green_multiplier": 1.0,
            "blue_multiplier": 1.0,
        },
        "cinematic_warm": {
            "brightness": 0.98,
            "contrast": 1.12,
            "saturation": 1.08,
            "sharpness": 1.04,
            "gamma": 1.03,
            "autocontrast_cutoff": 0,
            "red_multiplier": 1.04,
            "green_multiplier": 1.0,
            "blue_multiplier": 0.96,
        },
        "cold_rain": {
            "brightness": 0.97,
            "contrast": 1.1,
            "saturation": 0.95,
            "sharpness": 1.03,
            "gamma": 1.01,
            "autocontrast_cutoff": 0,
            "red_multiplier": 0.97,
            "green_multiplier": 1.0,
            "blue_multiplier": 1.05,
        },
    },
}


def clamp_byte(value: float) -> int:
    return max(0, min(255, int(round(value))))


def list_images(input_path: Path):
    if input_path.is_file():
        return [input_path] if input_path.suffix.lower() in IMAGE_EXTS else []

    files = []
    for item in sorted(input_path.rglob("*")):
        if item.is_file() and item.suffix.lower() in IMAGE_EXTS:
            files.append(item)
    return files


def apply_profile(image: Image.Image, profile: dict) -> Image.Image:
    out = image.convert("RGB")

    cutoff = float(profile.get("autocontrast_cutoff", 0.0))
    if cutoff > 0:
        out = ImageOps.autocontrast(out, cutoff=cutoff)

    brightness = float(profile.get("brightness", 1.0))
    contrast = float(profile.get("contrast", 1.0))
    saturation = float(profile.get("saturation", 1.0))
    sharpness = float(profile.get("sharpness", 1.0))

    if brightness != 1.0:
        out = ImageEnhance.Brightness(out).enhance(brightness)
    if contrast != 1.0:
        out = ImageEnhance.Contrast(out).enhance(contrast)
    if saturation != 1.0:
        out = ImageEnhance.Color(out).enhance(saturation)
    if sharpness != 1.0:
        out = ImageEnhance.Sharpness(out).enhance(sharpness)

    gamma = float(profile.get("gamma", 1.0))
    if gamma > 0 and gamma != 1.0:
        inv_gamma = 1.0 / gamma
        lut = [clamp_byte(((i / 255.0) ** inv_gamma) * 255.0) for i in range(256)]
        out = out.point(lut * 3)

    r_mul = float(profile.get("red_multiplier", 1.0))
    g_mul = float(profile.get("green_multiplier", 1.0))
    b_mul = float(profile.get("blue_multiplier", 1.0))

    if r_mul != 1.0 or g_mul != 1.0 or b_mul != 1.0:
        r, g, b = out.split()
        r = r.point(lambda x: clamp_byte(x * r_mul))
        g = g.point(lambda x: clamp_byte(x * g_mul))
        b = b.point(lambda x: clamp_byte(x * b_mul))
        out = Image.merge("RGB", (r, g, b))

    return out


def resolve_output_path(src: Path, input_root: Path, output_path: Path, input_is_dir: bool) -> Path:
    if not input_is_dir:
        if output_path.suffix.lower() in IMAGE_EXTS:
            return output_path
        output_path.mkdir(parents=True, exist_ok=True)
        return output_path / src.name

    rel = src.relative_to(input_root)
    final_path = output_path / rel
    final_path.parent.mkdir(parents=True, exist_ok=True)
    return final_path


def main():
    parser = argparse.ArgumentParser(description="Batch color correction from JSON profile.")
    parser.add_argument("--settings", required=False, default="", help="Optional path to color settings JSON")
    parser.add_argument("--profile", required=True, help="Profile name in settings file")
    parser.add_argument("--input", required=True, help="Input image file or directory")
    parser.add_argument("--output", required=True, help="Output image file or directory")
    args = parser.parse_args()

    settings_raw = str(args.settings or "").strip()
    settings_path = Path(settings_raw).resolve() if settings_raw else None
    input_path = Path(args.input).resolve()
    output_path = Path(args.output).resolve()

    settings_source = "builtin"
    settings = DEFAULT_COLOR_SETTINGS
    if settings_path:
        if not settings_path.exists():
            raise SystemExit(f"Settings file not found: {settings_path}")
        with settings_path.open("r", encoding="utf-8") as fh:
            settings = json.load(fh)
        settings_source = str(settings_path)
    if not input_path.exists():
        raise SystemExit(f"Input not found: {input_path}")

    profiles = settings.get("profiles", {})
    profile = profiles.get(args.profile)
    if profile is None:
        raise SystemExit(f"Profile '{args.profile}' not found in {settings_source}")

    images = list_images(input_path)
    if not images:
        raise SystemExit("No image files found for correction")

    input_is_dir = input_path.is_dir()
    processed = 0

    for src in images:
        dst = resolve_output_path(src, input_path, output_path, input_is_dir)
        with Image.open(src) as image:
            corrected = apply_profile(image, profile)
            corrected.save(dst)
        processed += 1

    print(f"Processed {processed} image(s) with profile '{args.profile}' (settings={settings_source})")


if __name__ == "__main__":
    main()
