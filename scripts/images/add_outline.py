#! /usr/bin/env python3

import argparse
import os
import sys
import tempfile
import subprocess
import shlex


def parse_args():
    parser = argparse.ArgumentParser(prog="add_outline.py")
    parser.add_argument("--directory", required=True)
    parser.add_argument("--width", required=True, type=int)
    parser.add_argument("--threshold", required=False, type=int)
    parser.add_argument("--recursive", action="store_true")
    parser.add_argument("--new-files", action="store_true")
    parser.add_argument("--smooth", required=False, type=float)
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


def build_im_args(in_path, out_path, width, threshold, smooth):
    args = [
        "magick",
        in_path,
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
            f"Disk:{width}",
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
            out_path,
        ]
    )
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


def process_file(path, width, threshold, smooth, new_files):
    d = os.path.dirname(path)
    base = os.path.basename(path)
    if new_files:
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
                    im_args = build_im_args(
                        os.path.abspath(path),
                        os.path.abspath(tmp_path),
                        width,
                        threshold,
                        smooth,
                    )
                    result = run_im(im_args)
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
            im_args = build_im_args(
                os.path.abspath(path),
                os.path.abspath(out_path),
                width,
                threshold,
                smooth,
            )
            result = run_im(im_args)
            if result.returncode != 0:
                return False, result.stderr.strip() or result.stdout.strip(), ""
            return True, "", out_path
        except Exception as e:
            return False, str(e), ""
    fd, tmp_path = tempfile.mkstemp(prefix=base + ".outlined.", suffix=".png", dir=d)
    os.close(fd)
    try:
        im_args = build_im_args(
            os.path.abspath(path), os.path.abspath(tmp_path), width, threshold, smooth
        )
        result = run_im(im_args)
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
        ok, err, produced = process_file(p, args.width, thr, sm, args.new_files)
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
