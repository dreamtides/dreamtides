#!/usr/bin/env python3

r"""
Usage:
  python run.py --input /path/to/input --output /path/to/output [--verbose]

Input images should be 1000px tall.

Example Usage:

    ./scripts/images/resize.py \
        --input client/Assets/ThirdParty/GameAssets/SourceImages/Standard
    ./scripts/images/card_images.py \
        --input client/Assets/ThirdParty/GameAssets/SourceImages/Standard \
        --output client/Assets/ThirdParty/GameAssets/CardImages/Standard \
        -r 45

    ./scripts/images/resize.py \
        --input client/Assets/ThirdParty/GameAssets/SourceImages/Dreamwell
    ./scripts/images/card_images.py \
        --input client/Assets/ThirdParty/GameAssets/SourceImages/Dreamwell \
        --output client/Assets/ThirdParty/GameAssets/CardImages/Dreamwell \
        --landscape \
        -r 45

    ./scripts/images/card_images.py \
        --input client/Assets/ThirdParty/GameAssets/SourceImages/Circular \
        --output client/Assets/ThirdParty/GameAssets/CardImages/Circular \
        --circle       
"""

import os
import subprocess
import argparse
import tempfile
import sys


def log(message, verbose_mode=False):
    """Helper function to log messages only in verbose mode."""
    if verbose_mode:
        print(message)


def get_image_dimensions(image_path, verbose=False):
    """Get the dimensions of an image using ImageMagick identify command."""
    log(f"Getting dimensions of {image_path}", verbose)
    result = subprocess.run(
        ["magick", "identify", "-format", "%[width]x%[height]", image_path],
        capture_output=True,
        text=True,
        check=True,
    )

    dimensions = result.stdout.strip().split("x")
    return int(dimensions[0]), int(dimensions[1])


def create_rounded_rectangle_mask(
    width, height, output_path, corner_radius=45, verbose=False
):
    """Create a rounded rectangle mask with the specified dimensions."""
    log(
        f"Creating rounded rectangle mask with dimensions {width}x{height} and corner radius {corner_radius}",
        verbose,
    )
    subprocess.run(
        [
            "magick",
            "-size",
            f"{width}x{height}",
            "xc:none",
            "-fill",
            "white",
            "-draw",
            f"roundrectangle 0,0,{width-1},{height-1},{corner_radius},{corner_radius}",
            output_path,
        ],
        check=True,
    )


def create_circle_mask(diameter, output_path, verbose=False):
    """Create a circular mask with the specified diameter."""
    log(f"Creating circle mask with diameter {diameter}", verbose)
    center = diameter / 2
    subprocess.run(
        [
            "magick",
            "-size",
            f"{diameter}x{diameter}",
            "xc:none",
            "-fill",
            "white",
            "-draw",
            f"circle {center},{center} {center},{diameter - 1}",
            output_path,
        ],
        check=True,
    )


def process_image(
    image_file,
    output_file,
    corner_radius=45,
    verbose=False,
    landscape=False,
    circle=False,
):
    """Process a single image file, adding rounded corners and resizing."""
    log(f"Processing: {image_file}", verbose)

    # Get image dimensions
    _, orig_height = get_image_dimensions(image_file, verbose)

    # Check if image height is exactly 1000 pixels
    if orig_height != 1000:
        raise ValueError(
            f"Error: Image {image_file} height is {orig_height}px. All images must be exactly 1000px tall."
        )

    if circle:
        target_width = orig_height
    elif landscape:
        target_width = int(orig_height * 1.6)
    else:
        target_width = int(orig_height / 1.15)

    # Create a temporary directory for our intermediate files
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create temporary file paths
        resized_file = os.path.join(temp_dir, "resized.png")
        mask_file = os.path.join(temp_dir, "mask.png")

        log(
            f"Step 1: Creating a {target_width}x{orig_height} version of the image",
            verbose,
        )
        # Create a resized version that maintains the original aspect ratio but fills the target space
        subprocess.run(
            [
                "magick",
                image_file,
                "-resize",
                f"{target_width}x{orig_height}^",  # ^ means maintain aspect ratio and fill the space
                "-gravity",
                "center",
                "-extent",
                f"{target_width}x{orig_height}",
                resized_file,
            ],
            check=True,
        )

        # Double-check the dimensions of the output file to ensure proper mask creation
        resized_width, resized_height = get_image_dimensions(resized_file, verbose)

        if circle:
            create_circle_mask(resized_width, mask_file, verbose)
        else:
            create_rounded_rectangle_mask(
                resized_width, resized_height, mask_file, corner_radius, verbose
            )

        mask_label = "circle" if circle else "rounded rectangle"
        log(f"Step 2: Applying {mask_label} mask", verbose)
        # Apply the mask to create rounded corners
        subprocess.run(
            [
                "magick",
                resized_file,
                mask_file,
                "-compose",
                "CopyOpacity",
                "-composite",
                output_file,
            ],
            check=True,
        )


def main():
    parser = argparse.ArgumentParser(
        description="Process images with ImageMagick to create rounded rectangles."
    )
    parser.add_argument(
        "--input", "-i", required=True, help="Input directory containing JPG files"
    )
    parser.add_argument(
        "--output", "-o", required=True, help="Output directory for processed PNG files"
    )
    parser.add_argument(
        "--corner-radius",
        "-r",
        type=int,
        default=45,
        help="Corner radius for rounded rectangle (default: 45)",
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Enable verbose logging"
    )
    parser.add_argument(
        "--landscape", "-l", action="store_true", help="Crop to 16:10 aspect ratio"
    )
    parser.add_argument(
        "--circle", "-c", action="store_true", help="Crop output to a circle"
    )

    # Parse arguments
    args = parser.parse_args()

    input_dir = args.input
    output_dir = args.output
    corner_radius = args.corner_radius
    verbose = args.verbose
    landscape = args.landscape
    circle = args.circle

    if circle and landscape:
        parser.error("--circle cannot be combined with --landscape")

    # Count of processed files
    processed_count = 0

    # Walk through all directories and subdirectories in the input directory
    for root, _, files in os.walk(input_dir):
        # Get all .jpg files in the current directory
        jpg_files = [f for f in files if f.lower().endswith(".jpg")]

        if jpg_files:
            # Get the relative path from input directory to create the same structure in output
            rel_path = os.path.relpath(root, input_dir)
            # Create the corresponding output directory
            curr_output_dir = (
                os.path.join(output_dir, rel_path) if rel_path != "." else output_dir
            )
            os.makedirs(curr_output_dir, exist_ok=True)

            if verbose:
                print(f"Found {len(jpg_files)} JPG files in {root}")

            for jpg_file in jpg_files:
                # Full path to the input file
                image_file = os.path.join(root, jpg_file)

                # Extract base name (without extension)
                filename = os.path.splitext(jpg_file)[0]

                # Construct the full path for the output
                output_file = os.path.join(curr_output_dir, f"{filename}.png")

                try:
                    # Process the image
                    process_image(
                        image_file,
                        output_file,
                        corner_radius,
                        verbose,
                        landscape,
                        circle,
                    )
                    processed_count += 1
                except ValueError as e:
                    print(e)
                    sys.exit(1)

    if processed_count == 0:
        print(f"No JPG files found in {input_dir} or its subdirectories")
    else:
        log(f"Done! Processed {processed_count} files, saved to: {output_dir}", verbose)


if __name__ == "__main__":
    main()
