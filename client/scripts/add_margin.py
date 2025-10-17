#!/usr/bin/env python3
import os
import subprocess
import argparse
import sys

def log(message, verbose=False):
    if verbose:
        print(message)

def find_png_files(directory, recursive):
    if recursive:
        for dirpath, dirnames, filenames in os.walk(directory):
            for filename in filenames:
                if filename.lower().endswith('.png'):
                    yield os.path.join(dirpath, filename)
    else:
        for filename in os.listdir(directory):
            if filename.lower().endswith('.png'):
                yield os.path.join(directory, filename)

def add_margin(image_path, margin, verbose=False):
    temp_output = f"{image_path}.temp"
    log(f"Processing: {image_path}", verbose)
    subprocess.run([
        "magick",
        image_path,
        "-alpha", "set",
        "-bordercolor", "none",
        "-border", str(margin),
        temp_output
    ], check=True)
    subprocess.run(["mv", temp_output, image_path], check=True)

def main():
    parser = argparse.ArgumentParser(description='Add transparent margin to PNG images in a directory.')
    parser.add_argument('directory', help='Directory containing PNG files')
    parser.add_argument('--margin', '-m', type=int, required=True, help='Margin size in pixels')
    parser.add_argument('--recursive', '-r', action='store_true', help='Process subdirectories recursively')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose output')
    args = parser.parse_args()

    directory = args.directory
    margin = args.margin
    recursive = args.recursive
    verbose = args.verbose

    if not os.path.isdir(directory):
        print(f"Directory not found: {directory}")
        sys.exit(1)

    files = list(find_png_files(directory, recursive))
    if not files:
        print(f"No PNG files found in {directory}")
        return

    processed = 0
    for image_path in files:
        add_margin(image_path, margin, verbose)
        processed += 1

    log(f"Done! Processed {processed} files in {directory}", verbose)

if __name__ == "__main__":
    main()


