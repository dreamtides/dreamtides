#!/usr/bin/env python3
"""
Usage:
  python run.py --input /path/to/input --output /path/to/output [--verbose]
"""

import os
import glob
import subprocess
import argparse
import tempfile
import json

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

def create_rounded_rectangle_mask(width, height, output_path, corner_radius=15, verbose=False):
    """Create a rounded rectangle mask with the specified dimensions."""
    log(f"Creating rounded rectangle mask with dimensions {width}x{height} and corner radius {corner_radius}", verbose)
    subprocess.run([
        "magick", "-size", f"{width}x{height}", "xc:none", 
        "-fill", "white",
        "-draw", f"roundrectangle 0,0,{width-1},{height-1},{corner_radius},{corner_radius}", 
        output_path
    ], check=True)

def main():
    parser = argparse.ArgumentParser(description='Process images with ImageMagick to create rounded rectangles.')
    parser.add_argument('--input', '-i', required=True, help='Input directory containing JPG files')
    parser.add_argument('--output', '-o', required=True, help='Output directory for processed PNG files')
    parser.add_argument('--corner-radius', '-r', type=int, default=15, help='Corner radius for rounded rectangle (default: 15)')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose logging')
    
    # Parse arguments
    args = parser.parse_args()
    
    input_dir = args.input
    output_dir = args.output
    corner_radius = args.corner_radius
    verbose = args.verbose

    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)

    # Get all .jpg files in the input directory
    jpg_files = glob.glob(os.path.join(input_dir, "*.jpg"))
    
    if not jpg_files:
        print(f"No JPG files found in {input_dir}")
        return
    
    if verbose:
        print(f"Found {len(jpg_files)} JPG files to process")
    
    for image_file in jpg_files:
        # Extract base name (without path and extension)
        filename = os.path.splitext(os.path.basename(image_file))[0]
        
        # Construct the full path for the output
        output_file = os.path.join(output_dir, f"{filename}.png")

        log(f"Processing: {filename}.jpg", verbose)
        
        # Get image dimensions
        _, orig_height = get_image_dimensions(image_file, verbose)  
        
        # Calculate target width (height / 1.15)
        target_width = int(orig_height / 1.15)
        
        # Create a temporary directory for our intermediate files
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create temporary file paths
            resized_file = os.path.join(temp_dir, "resized.png")
            mask_file = os.path.join(temp_dir, "mask.png")
            
            log(f"Step 1: Creating a {target_width}x{orig_height} version of the image", verbose)
            # Create a resized version that maintains the original aspect ratio but fills the target space
            subprocess.run([
                "magick",
                image_file,
                "-resize", f"{target_width}x{orig_height}^",  # ^ means maintain aspect ratio and fill the space
                "-gravity", "center", 
                "-extent", f"{target_width}x{orig_height}",
                resized_file
            ], check=True)
            
            # Double-check the dimensions of the output file to ensure proper mask creation
            resized_width, resized_height = get_image_dimensions(resized_file, verbose)
            
            # Create a rounded rectangle mask with the same dimensions as the resized image
            create_rounded_rectangle_mask(resized_width, resized_height, mask_file, corner_radius, verbose)
            
            log(f"Step 2: Applying rounded rectangle mask", verbose)
            # Apply the mask to create rounded corners
            subprocess.run([
                "magick",
                resized_file,
                mask_file,
                "-compose", "CopyOpacity",
                "-composite",
                output_file
            ], check=True)
    
    log(f"Done! Files saved to: {output_dir}", verbose)

if __name__ == "__main__":
    main() 