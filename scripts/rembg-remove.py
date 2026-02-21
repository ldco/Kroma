#!/usr/bin/env python3
import argparse
from io import BytesIO
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(message)


def normalize_format(value: str) -> str:
    fmt = value.lower().strip()
    if fmt == "jpeg":
        fmt = "jpg"
    if fmt not in {"png", "jpg", "webp"}:
        fail(f"Unsupported --format '{value}'. Expected png|jpg|webp")
    return fmt


def main():
    parser = argparse.ArgumentParser(description="Remove image background with rembg")
    parser.add_argument("--input", required=True, help="Input image file")
    parser.add_argument("--output", required=True, help="Output image file")
    parser.add_argument("--model", default="u2net", help="rembg model name")
    parser.add_argument("--format", default="png", help="Output format: png|jpg|webp")
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    output_path = Path(args.output).resolve()
    output_fmt = normalize_format(args.format)

    if not input_path.exists() or not input_path.is_file():
        fail(f"Input file not found: {input_path}")

    try:
        from rembg import remove, new_session
        from PIL import Image
    except Exception as exc:
        fail(f"rembg runtime is missing. Run: bash scripts/setup-rembg.sh\nOriginal error: {exc}")

    data = input_path.read_bytes()
    session = new_session(args.model)
    out_bytes = remove(data, session=session)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    if output_fmt == "png":
        output_path.write_bytes(out_bytes)
        return

    with Image.open(BytesIO(out_bytes)).convert("RGBA") as img:
        if output_fmt == "jpg":
            # Flatten alpha onto white for jpg output.
            bg = Image.new("RGB", img.size, (255, 255, 255))
            bg.paste(img, mask=img.getchannel("A"))
            bg.save(output_path, format="JPEG", quality=95)
        elif output_fmt == "webp":
            img.save(output_path, format="WEBP", quality=95)


if __name__ == "__main__":
    main()
