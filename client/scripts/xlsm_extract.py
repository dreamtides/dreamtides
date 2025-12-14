#!/usr/bin/env python3
import sys
import zipfile
import argparse
from pathlib import Path

PLACEHOLDER_PNG = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\tpHYs\x00\x00\x0b\x13\x00\x00\x0b\x13\x01\x00\x9a\x9c\x18\x00\x00\x00\nIDATx\x9cc\xf8\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00IEND\xaeB`\x82'

def extract_xlsm(xlsm_path, output_dir, clean=False):
    xlsm_path = Path(xlsm_path)
    output_dir = Path(output_dir)
    
    if not xlsm_path.exists():
        sys.stderr.write(f"Error: File not found: {xlsm_path}\n")
        sys.exit(1)
    
    if not zipfile.is_zipfile(xlsm_path):
        sys.stderr.write(f"Error: Not a valid ZIP file: {xlsm_path}\n")
        sys.exit(1)
    
    if clean and output_dir.exists():
        import shutil
        shutil.rmtree(output_dir)
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    with zipfile.ZipFile(xlsm_path, 'r') as zip_ref:
        for entry_name in zip_ref.namelist():
            entry_path = output_dir / entry_name
            entry_path.parent.mkdir(parents=True, exist_ok=True)
            
            if entry_name.startswith('xl/media/'):
                entry_path.write_bytes(PLACEHOLDER_PNG)
            else:
                entry_data = zip_ref.read(entry_name)
                entry_path.write_bytes(entry_data)

def main():
    parser = argparse.ArgumentParser(description='Extract .xlsm file to directory, replacing images with 1x1 PNGs')
    parser.add_argument('xlsm_path', help='Path to .xlsm file')
    parser.add_argument('output_dir', help='Output directory')
    parser.add_argument('--clean', action='store_true', help='Remove output directory before extracting')
    args = parser.parse_args()
    
    extract_xlsm(args.xlsm_path, args.output_dir, clean=args.clean)

if __name__ == '__main__':
    main()
