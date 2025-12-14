#!/usr/bin/env python3
import sys
import zipfile
import argparse
from pathlib import Path

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
            if entry_name.startswith('xl/media/') or entry_name.startswith('xl/embeddings/'):
                continue
            
            entry_path = output_dir / entry_name
            entry_path.parent.mkdir(parents=True, exist_ok=True)
            
            entry_data = zip_ref.read(entry_name)
            entry_path.write_bytes(entry_data)
    
    media_dir = output_dir / 'xl' / 'media'
    embeddings_dir = output_dir / 'xl' / 'embeddings'
    media_dir.mkdir(parents=True, exist_ok=True)
    embeddings_dir.mkdir(parents=True, exist_ok=True)

def main():
    parser = argparse.ArgumentParser(description='Extract .xlsm file to directory, excluding images')
    parser.add_argument('xlsm_path', help='Path to .xlsm file')
    parser.add_argument('output_dir', help='Output directory')
    parser.add_argument('--clean', action='store_true', help='Remove output directory before extracting')
    args = parser.parse_args()
    
    extract_xlsm(args.xlsm_path, args.output_dir, clean=args.clean)

if __name__ == '__main__':
    main()

