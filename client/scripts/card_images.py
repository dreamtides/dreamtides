#!/usr/bin/env python3
"""
Usage:
  python run.py --input /path/to/input --output /path/to/output --mask /path/to/mask
"""

import os
import glob
import subprocess
import argparse

def main():
    parser = argparse.ArgumentParser(description='Process images with ImageMagick using a mask.')
    parser.add_argument('--input', '-i', required=True, help='Input directory containing JPG files')
    parser.add_argument('--output', '-o', required=True, help='Output directory for processed PNG files')
    parser.add_argument('--mask', '-m', required=True, help='Mask file to apply to images')
    
    # Parse arguments
    args = parser.parse_args()
    
    input_dir = args.input
    output_dir = args.output
    mask_file = args.mask

    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)

    # Get all .jpg files in the input directory
    jpg_files = glob.glob(os.path.join(input_dir, "*.jpg"))
    
    if not jpg_files:
        print(f"No JPG files found in {input_dir}")
        return
    
    for image_file in jpg_files:
        # Extract base name (without path and extension)
        filename = os.path.splitext(os.path.basename(image_file))[0]
        
        # Construct the full path for the output
        output_file = os.path.join(output_dir, f"{filename}.png")
        
        print(f"Processing: {image_file} -> {output_file}")
        
        # Run ImageMagick command
        subprocess.run([
            "magick",
            image_file,
            mask_file,
            "-alpha", "Off",
            "-compose", "CopyOpacity",
            "-composite",
            "-trim", "+repage",
            "-colorspace", "RGB",
            output_file
        ], check=True)
    
    print(f"Done! Files are in: {output_dir}")

if __name__ == "__main__":
    main() 