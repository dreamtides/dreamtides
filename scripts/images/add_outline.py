#! /usr/bin/env python3

import argparse
import math
import os
import sys
import tempfile
import subprocess
import shlex


# Outline widths are specified relative to this reference edge length (in
# pixels). The per-image disk radius is scaled by min(width, height) /
# REFERENCE_SIZE so the outline reads as the same relative thickness across a
# mixed-resolution image set.
REFERENCE_SIZE = 512


def parse_args():
    parser = argparse.ArgumentParser(prog="add_outline.py")
    parser.add_argument("--directory", required=True)
    parser.add_argument("--width", type=int, default=20)
    parser.add_argument("--threshold", required=False, type=int)
    parser.add_argument("--recursive", action="store_true")
    parser.add_argument("--new-files", action="store_true")
    parser.add_argument("--smooth", required=False, type=float)
    parser.add_argument(
        "--reference-size",
        type=int,
        default=REFERENCE_SIZE,
        help="Edge length (px) that --width is measured against (default 512). "
        "Disk radius is scaled by min(w,h)/reference per image.",
    )
    parser.add_argument(
        "--no-scale",
        action="store_true",
        help="Treat --width as an absolute pixel radius without scaling to "
        "each image's resolution.",
    )
    parser.add_argument(
        "--grow",
        action="store_true",
        help="Keep the padded canvas (larger output) instead of resizing back "
        "to the original dimensions.",
    )
    return parser.parse_args()


def is_png(path):
    return os.path.isfile(path) and path.lower().endswith(".png")


def iter_pngs(root_dir, recursive):
    if recursive:
        for dirpath, _, filenames in os.walk(root_dir):
            for name in filenames:
                p = os.path.join(dirpath, name)
                if is_png(p):
                    yield p
    else:
        for name in os.listdir(root_dir):
            p = os.path.join(root_dir, name)
            if is_png(p):
                yield p


def get_dimensions(path):
    result = subprocess.run(
        ["magick", "identify", "-format", "%w %h", path],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None, result.stderr.strip() or result.stdout.strip()
    try:
        w, h = result.stdout.strip().split()
        return (int(w), int(h)), ""
    except ValueError:
        return None, f"could not parse dimensions: {result.stdout.strip()!r}"


def compute_radius(width, w, h, reference, scale):
    if not scale:
        return max(1, width)
    return max(1, round(width * min(w, h) / reference))


def compute_margin(radius, smooth):
    # The dilated silhouette grows by `radius` and a Gaussian blur of sigma
    # `smooth` bleeds roughly 3*sigma further. Pad enough that neither is
    # clipped at the canvas edge, plus a small safety margin.
    margin = radius
    if smooth is not None and smooth > 0:
        margin += int(math.ceil(3 * smooth))
    return margin + 2


def build_im_args(
    in_path, out_path, radius, margin, threshold, smooth, out_w, out_h, grow
):
    args = [
        "magick",
        in_path,
        # Pad with a transparent border so the dilated outline has room to grow
        # instead of being clipped at the original canvas edge.
        "-bordercolor",
        "none",
        "-border",
        str(margin),
        "(",
        "+clone",
        "-alpha",
        "extract",
    ]
    if threshold is not None:
        args.extend(["-threshold", f"{threshold}%"])
    args.extend(
        [
            "-morphology",
            "Dilate",
            f"Disk:{radius}",
        ]
    )
    if smooth is not None and smooth > 0:
        args.extend(["-blur", f"0x{smooth}"])
    args.extend(
        [
            "-alpha",
            "copy",
            "-fill",
            "black",
            "-colorize",
            "100",
            ")",
            "-compose",
            "DstOver",
            "-composite",
        ]
    )
    if not grow:
        # Shrink the padded result back to the original dimensions so the
        # outlined image keeps its size; the subject scales down just enough to
        # fit the new border.
        args.extend(["-resize", f"{out_w}x{out_h}!"])
    args.append(out_path)
    return args


def run_im(args):
    if os.name == "nt":
        cmdline = subprocess.list2cmdline(args)
        return subprocess.run(cmdline, shell=False, capture_output=True, text=True)
    else:
        cmd_str = " ".join(shlex.quote(a) for a in args)
        return subprocess.run(
            shlex.split(cmd_str), shell=False, capture_output=True, text=True
        )


def process_file(path, opts):
    dims, err = get_dimensions(path)
    if dims is None:
        return False, err, ""
    w, h = dims
    radius = compute_radius(opts.width, w, h, opts.reference_size, not opts.no_scale)
    margin = compute_margin(radius, opts.smooth)

    def make_args(out_path):
        return build_im_args(
            os.path.abspath(path),
            os.path.abspath(out_path),
            radius,
            margin,
            opts.threshold,
            opts.smooth,
            w,
            h,
            opts.grow,
        )

    d = os.path.dirname(path)
    base = os.path.basename(path)
    if opts.new_files:
        if base.lower().startswith("outline_"):
            out_path = path
        else:
            out_path = os.path.join(d, f"outline_{base}")
        try:
            if os.path.abspath(out_path) == os.path.abspath(path):
                fd, tmp_path = tempfile.mkstemp(
                    prefix=base + ".outlined.", suffix=".png", dir=d
                )
                os.close(fd)
                try:
                    result = run_im(make_args(tmp_path))
                    if result.returncode != 0:
                        if os.path.exists(tmp_path):
                            try:
                                os.remove(tmp_path)
                            except Exception:
                                pass
                        return False, result.stderr.strip() or result.stdout.strip(), ""
                    os.replace(tmp_path, out_path)
                    return True, "", out_path
                except Exception as e:
                    if os.path.exists(tmp_path):
                        try:
                            os.remove(tmp_path)
                        except Exception:
                            pass
                    return False, str(e), ""
            result = run_im(make_args(out_path))
            if result.returncode != 0:
                return False, result.stderr.strip() or result.stdout.strip(), ""
            return True, "", out_path
        except Exception as e:
            return False, str(e), ""
    fd, tmp_path = tempfile.mkstemp(prefix=base + ".outlined.", suffix=".png", dir=d)
    os.close(fd)
    try:
        result = run_im(make_args(tmp_path))
        if result.returncode != 0:
            if os.path.exists(tmp_path):
                try:
                    os.remove(tmp_path)
                except Exception:
                    pass
            return False, result.stderr.strip() or result.stdout.strip(), ""
        os.replace(tmp_path, path)
        return True, "", path
    except Exception as e:
        if os.path.exists(tmp_path):
            try:
                os.remove(tmp_path)
            except Exception:
                pass
        return False, str(e), ""


def main():
    args = parse_args()
    root = os.path.abspath(args.directory)
    if not os.path.isdir(root):
        print("Directory not found:", root, file=sys.stderr)
        sys.exit(2)
    if args.width < 1:
        print("Width must be >= 1", file=sys.stderr)
        sys.exit(2)
    if args.reference_size < 1:
        print("Reference size must be >= 1", file=sys.stderr)
        sys.exit(2)
    thr = args.threshold
    if thr is not None and (thr < 0 or thr > 100):
        print("Threshold must be between 0 and 100", file=sys.stderr)
        sys.exit(2)
    sm = args.smooth
    if sm is not None and sm < 0:
        print("Smooth must be >= 0", file=sys.stderr)
        sys.exit(2)
    files = list(iter_pngs(root, args.recursive))
    if args.new_files:
        files = [
            p for p in files if not os.path.basename(p).lower().startswith("outline_")
        ]
    if not files:
        print("No PNG files found.")
        sys.exit(0)
    failures = 0
    for p in files:
        ok, err, produced = process_file(p, args)
        if not ok:
            failures += 1
            print(f"Failed: {p}: {err}", file=sys.stderr)
        else:
            if args.new_files and produced:
                print(f"Outlined: {p} -> {produced}")
            else:
                print(f"Outlined: {p}")
    if failures:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
