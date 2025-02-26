#!/usr/bin/env bash
#
# Usage:
#   ./run.sh /path/to/input /path/to/output

set -e
set -u

# Check for correct number of args
if [ $# -lt 2 ]; then
  echo "Usage: $0 <input_directory> <output_directory>"
  exit 1
fi

INPUT_DIR="$1"
OUTPUT_DIR="$2" # Path to mask file — adjust if needed
MASK_FILE="image_mask.png"

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Iterate over all .jpg files in the input directory
for image_file in "$INPUT_DIR"/*.jpg; do
  
  # If no .jpg files exist, skip to avoid errors
  [ -e "$image_file" ] || continue
  
  # Extract base name (without path and extension)
  filename="$(basename "$image_file" .jpg)"
  
  # Construct the full path for the output
  output_file="$OUTPUT_DIR/${filename}.png"
  
  echo "Processing: $image_file -> $output_file"
  
  # Perform the ImageMagick command:
  # - Combine the JPG with the mask
  # - Remove any existing alpha from the base
  # - Use the mask's alpha via -compose CopyOpacity
  # - Composite them
  # - Set colorspace to RGB
  # - Save as PNG
  magick "$image_file" "$MASK_FILE" \
    -alpha Off \
    -compose CopyOpacity \
    -composite \
    -trim +repage \
    -colorspace RGB \
    "$output_file"
    
done

echo "Done! Files are in: $OUTPUT_DIR"
