#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageStat

IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".tif", ".tiff"}


def list_images(input_path: Path):
    if input_path.is_file():
        return [input_path] if input_path.suffix.lower() in IMAGE_EXTS else []

    files = []
    for item in sorted(input_path.rglob("*")):
        if item.is_file() and item.suffix.lower() in IMAGE_EXTS:
            files.append(item)
    return files


def mean_channel_diff(a: Image.Image, b: Image.Image) -> float:
    diff = ImageChops.difference(a, b)
    stat = ImageStat.Stat(diff)
    return float(stat.mean[0]) if stat.mean else 0.0


def compute_chroma_delta(image: Image.Image) -> float:
    rgb = image.convert("RGB")
    r, g, b = rgb.split()
    rg = mean_channel_diff(r, g)
    rb = mean_channel_diff(r, b)
    gb = mean_channel_diff(g, b)
    return (rg + rb + gb) / 3.0


def main():
    parser = argparse.ArgumentParser(description="Quality guard for generated images")
    parser.add_argument("--input", required=True, help="Input image file or directory")
    parser.add_argument("--max-chroma-delta", type=float, default=2.0, help="Allowed mean channel delta")
    parser.add_argument("--enforce-grayscale", action="store_true", help="Hard fail if image is not grayscale-like")
    parser.add_argument(
        "--fail-on-chroma-exceed",
        action="store_true",
        help="Hard fail when chroma delta exceeds threshold",
    )
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    threshold = max(0.0, float(args.max_chroma_delta))

    report = {
        "input": str(input_path),
        "settings": {
            "max_chroma_delta": threshold,
            "enforce_grayscale": bool(args.enforce_grayscale),
            "fail_on_chroma_exceed": bool(args.fail_on_chroma_exceed),
        },
        "summary": {
            "total_files": 0,
            "hard_failures": 0,
            "soft_warnings": 0,
        },
        "files": [],
    }

    if not input_path.exists():
        report["summary"]["hard_failures"] = 1
        report["files"].append(
            {
                "file": str(input_path),
                "error": "input_not_found",
                "hard_fail_reasons": ["input_not_found"],
                "soft_warnings": [],
            }
        )
        print(json.dumps(report, ensure_ascii=False))
        return

    images = list_images(input_path)
    report["summary"]["total_files"] = len(images)

    if not images:
        report["summary"]["hard_failures"] = 1
        report["files"].append(
            {
                "file": str(input_path),
                "error": "no_images_found",
                "hard_fail_reasons": ["no_images_found"],
                "soft_warnings": [],
            }
        )
        print(json.dumps(report, ensure_ascii=False))
        return

    for img_path in images:
        entry = {
            "file": str(img_path),
            "chroma_delta": None,
            "grayscale_like": None,
            "hard_fail_reasons": [],
            "soft_warnings": [],
        }

        try:
            with Image.open(img_path) as img:
                chroma = compute_chroma_delta(img)
            is_grayscale_like = chroma <= threshold
            entry["chroma_delta"] = round(chroma, 4)
            entry["grayscale_like"] = bool(is_grayscale_like)

            if args.enforce_grayscale and not is_grayscale_like:
                entry["hard_fail_reasons"].append("not_grayscale_like")

            if chroma > threshold:
                if args.fail_on_chroma_exceed:
                    entry["hard_fail_reasons"].append("chroma_exceeds_threshold")
                else:
                    entry["soft_warnings"].append("chroma_exceeds_threshold")
        except Exception as exc:
            entry["error"] = str(exc)
            entry["hard_fail_reasons"].append("image_read_failed")

        if entry["hard_fail_reasons"]:
            report["summary"]["hard_failures"] += 1
        if entry["soft_warnings"]:
            report["summary"]["soft_warnings"] += 1

        report["files"].append(entry)

    print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
