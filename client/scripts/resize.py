#!/usr/bin/env python3
"""
Usage:
  python scripts/resize.py --input /path/to/input [--max-height 1000] [--verbose]
"""

import os
import glob
import subprocess
import argparse

def log(message, verbose_mode=False):
    """Helper function to log messages only in verbose mode."""
    if verbose_mode:
        print(message)

def get_image_dimensions(image_path, verbose=False):
    """Get the dimensions of an image using ImageMagick identify command."""
    log(f"Getting dimensions of {image_path}", verbose) 
    result = subprocess.run([
        "magick", "identify", "-format", "%[width]x%[height]", image_path
    ], capture_output=True, text=True, check=True)
    
    dimensions = result.stdout.strip().split('x')
    return int(dimensions[0]), int(dimensions[1])

def resize_image(image_path, max_height, verbose=False):
    """Resize an image in place to have specified maximum height while preserving aspect ratio."""
    log(f"Resizing image {image_path} to max height of {max_height}px", verbose)

    # Create a temporary file for the resized image
    temp_output = f"{image_path}.temp"
    
    subprocess.run([
        "magick",
        image_path,
        "-resize", f"x{max_height}",  # x{height} means resize to specified height while preserving aspect ratio
        temp_output
    ], check=True)
    
    # Replace the original with the resized version
    subprocess.run(["mv", temp_output, image_path], check=True)
    
    # Log the new dimensions
    if verbose:
        new_width, new_height = get_image_dimensions(image_path, verbose)
        log(f"Resized to {new_width}x{new_height}", verbose)

def main():
    parser = argparse.ArgumentParser(description='Resize images to a specified maximum height while preserving aspect ratio.')
    parser.add_argument('--input', '-i', required=True, help='Input directory containing image files')
    parser.add_argument('--max-height', '-mh', type=int, default=1000, help='Maximum height of resized images (default: 1000)')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose logging')
    
    # Parse arguments
    args = parser.parse_args()
    
    input_dir = args.input
    max_height = args.max_height
    verbose = args.verbose

    # Get all image files in the input directory (supporting multiple formats)
    supported_extensions = ['*.jpg', '*.jpeg', '*.png', '*.gif', '*.bmp', '*.tiff']
    image_files = []
    
    for ext in supported_extensions:
        image_files.extend(glob.glob(os.path.join(input_dir, ext)))
        # Also check for uppercase extensions
        image_files.extend(glob.glob(os.path.join(input_dir, ext.upper())))
    
    if not image_files:
        print(f"No image files found in {input_dir}")
        return
    
    if verbose:
        print(f"Found {len(image_files)} image files to process")
    
    for image_file in image_files:
        # Extract base name and extension for logging
        filename = os.path.basename(image_file)
        
        log(f"Processing: {filename}", verbose)
        
        # Get original image dimensions
        width, height = get_image_dimensions(image_file, verbose)
        
        if height <= max_height:
            log(f"Image already smaller than or equal to max height ({height} <= {max_height}). Skipping.", verbose)
        else:
            # Resize the image in place
            resize_image(image_file, max_height, verbose)
    
    log(f"Done! All images in {input_dir} have been processed.", verbose)

if __name__ == "__main__":
    main() 