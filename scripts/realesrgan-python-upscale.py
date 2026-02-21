#!/usr/bin/env python3
import argparse
import os
import sys
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(message)


def add_local_source_paths() -> None:
    project_root = Path(__file__).resolve().parents[1]
    src_root = project_root / "tools" / "realesrgan-python" / "src"
    for folder in ("Real-ESRGAN", "BasicSR"):
        candidate = src_root / folder
        if candidate.exists():
            sys.path.insert(0, str(candidate))


def clamp_extension(value: str) -> str:
    ext = value.lower().strip()
    if ext == "jpeg":
        ext = "jpg"
    allowed = {"auto", "jpg", "png", "webp"}
    if ext not in allowed:
        fail(f"Unsupported --ext '{value}'. Expected one of: {', '.join(sorted(allowed))}")
    return ext


def import_runtime_deps():
    add_local_source_paths()
    try:
        import cv2  # noqa: F401
        from basicsr.archs.rrdbnet_arch import RRDBNet  # noqa: F401
        from basicsr.utils.download_util import load_file_from_url  # noqa: F401
        from realesrgan import RealESRGANer  # noqa: F401
        from realesrgan.archs.srvgg_arch import SRVGGNetCompact  # noqa: F401
    except Exception as exc:
        fail(
            "Python Real-ESRGAN dependencies are missing. "
            "Run: bash scripts/setup-realesrgan-python.sh\n"
            f"Original error: {exc}"
        )



def build_model_spec(model_name: str):
    from basicsr.archs.rrdbnet_arch import RRDBNet
    from realesrgan.archs.srvgg_arch import SRVGGNetCompact

    if model_name == "RealESRGAN_x4plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth"
        ]
    if model_name == "RealESRNet_x4plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.1/RealESRNet_x4plus.pth"
        ]
    if model_name == "RealESRGAN_x4plus_anime_6B":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=6, num_grow_ch=32, scale=4)
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth"
        ]
    if model_name == "RealESRGAN_x2plus":
        model = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=2)
        return 2, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.1/RealESRGAN_x2plus.pth"
        ]
    if model_name == "realesr-animevideov3":
        model = SRVGGNetCompact(num_in_ch=3, num_out_ch=3, num_feat=64, num_conv=16, upscale=4, act_type="prelu")
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-animevideov3.pth"
        ]
    if model_name == "realesr-general-x4v3":
        model = SRVGGNetCompact(num_in_ch=3, num_out_ch=3, num_feat=64, num_conv=32, upscale=4, act_type="prelu")
        return 4, model, [
            "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesr-general-x4v3.pth"
        ]

    fail(
        "Unsupported --model-name. "
        "Use one of: RealESRGAN_x4plus, RealESRNet_x4plus, RealESRGAN_x4plus_anime_6B, "
        "RealESRGAN_x2plus, realesr-animevideov3, realesr-general-x4v3"
    )



def resolve_model_path(model_name: str, weights_dir: Path) -> str:
    from basicsr.utils.download_util import load_file_from_url

    weights_dir.mkdir(parents=True, exist_ok=True)
    candidate = weights_dir / f"{model_name}.pth"
    if candidate.exists():
        return str(candidate)

    _, _, urls = build_model_spec(model_name)
    resolved = None
    for url in urls:
        resolved = load_file_from_url(url=url, model_dir=str(weights_dir), progress=True, file_name=None)
    if not resolved:
        fail(f"Could not download model weights for {model_name}")
    return str(resolved)



def list_input_images(input_path: Path):
    if input_path.is_file():
        return [input_path]

    image_exts = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".tif", ".tiff"}
    files = []
    for item in sorted(input_path.rglob("*")):
        if item.is_file() and item.suffix.lower() in image_exts:
            files.append(item)
    return files



def output_path_for(src: Path, input_root: Path, output_path: Path, ext: str, input_is_dir: bool) -> Path:
    if not input_is_dir:
        if output_path.suffix.lower() in {".jpg", ".jpeg", ".png", ".webp"}:
            final = output_path
        else:
            output_path.mkdir(parents=True, exist_ok=True)
            final = output_path / src.name
    else:
        rel = src.relative_to(input_root)
        final = output_path / rel
        final.parent.mkdir(parents=True, exist_ok=True)

    if ext == "auto":
        normalized_ext = src.suffix.lower()
        if normalized_ext == ".jpeg":
            normalized_ext = ".jpg"
    else:
        normalized_ext = f".{ext}"

    return final.with_suffix(normalized_ext)



def main():
    parser = argparse.ArgumentParser(description="Real-ESRGAN python backend wrapper")
    parser.add_argument("--input", required=True, help="Input image file or directory")
    parser.add_argument("--output", required=True, help="Output image file or directory")
    parser.add_argument("--model-name", default="RealESRGAN_x4plus", help="Real-ESRGAN model name")
    parser.add_argument("--outscale", type=float, default=2.0, help="Final upscale ratio")
    parser.add_argument("--tile", type=int, default=0, help="Tile size, 0 for no tile")
    parser.add_argument("--tile-pad", type=int, default=10, help="Tile padding")
    parser.add_argument("--pre-pad", type=int, default=0, help="Pre padding")
    parser.add_argument("--ext", default="png", help="Output extension: auto|jpg|png|webp")
    parser.add_argument("--weights-dir", default="", help="Optional custom weights directory")
    parser.add_argument("--gpu-id", type=int, default=None, help="GPU id (optional)")
    parser.add_argument("--fp32", action="store_true", help="Force fp32")
    args = parser.parse_args()

    input_path = Path(args.input).resolve()
    output_path = Path(args.output).resolve()
    if not input_path.exists():
        fail(f"Input path not found: {input_path}")

    ext = clamp_extension(args.ext)
    import_runtime_deps()

    from realesrgan import RealESRGANer
    import cv2

    netscale, model, _ = build_model_spec(args.model_name)
    default_weights_dir = Path(__file__).resolve().parents[1] / "tools" / "realesrgan-python" / "weights"
    weights_dir = Path(args.weights_dir).resolve() if args.weights_dir else default_weights_dir
    model_path = resolve_model_path(args.model_name, weights_dir)

    upsampler = RealESRGANer(
        scale=netscale,
        model_path=model_path,
        model=model,
        tile=args.tile,
        tile_pad=args.tile_pad,
        pre_pad=args.pre_pad,
        half=not args.fp32,
        gpu_id=args.gpu_id,
    )

    images = list_input_images(input_path)
    if not images:
        fail("No image files found in input path")

    input_is_dir = input_path.is_dir()
    if input_is_dir:
        output_path.mkdir(parents=True, exist_ok=True)
    else:
        output_path.parent.mkdir(parents=True, exist_ok=True)

    done = 0
    for src in images:
        dst = output_path_for(src, input_path, output_path, ext, input_is_dir)

        img = cv2.imread(str(src), cv2.IMREAD_UNCHANGED)
        if img is None:
            print(f"Skip unreadable image: {src}", file=sys.stderr)
            continue

        try:
            output, _ = upsampler.enhance(img, outscale=args.outscale)
        except RuntimeError as exc:
            fail(f"Upscale failed for {src}: {exc}. Try smaller --tile value.")

        dst.parent.mkdir(parents=True, exist_ok=True)
        ok = cv2.imwrite(str(dst), output)
        if not ok:
            fail(f"Failed to write output: {dst}")
        done += 1

    print(f"Processed {done} image(s) with model {args.model_name} (outscale x{args.outscale})")


if __name__ == "__main__":
    main()
