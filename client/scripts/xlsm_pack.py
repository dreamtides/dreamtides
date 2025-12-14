#!/usr/bin/env python3
import sys
import zipfile
import argparse
from pathlib import Path

def pack_xlsm(source_dir, output_xlsm, overwrite=False):
    source_dir = Path(source_dir)
    output_xlsm = Path(output_xlsm)
    
    if not source_dir.exists() or not source_dir.is_dir():
        sys.stderr.write(f"Error: Directory not found: {source_dir}\n")
        sys.exit(1)
    
    if output_xlsm.exists() and not overwrite:
        sys.stderr.write(f"Error: Output file exists: {output_xlsm}\n")
        sys.stderr.write("  Use --overwrite to replace it\n")
        sys.exit(1)
    
    with zipfile.ZipFile(output_xlsm, 'w', zipfile.ZIP_DEFLATED) as zip_ref:
        for file_path in sorted(source_dir.rglob('*')):
            if file_path.is_file():
                rel_path = file_path.relative_to(source_dir)
                arcname = str(rel_path).replace('\\', '/')
                
                if arcname.startswith('xl/media/') or arcname.startswith('xl/embeddings/'):
                    continue
                
                zip_ref.write(file_path, arcname)

def main():
    parser = argparse.ArgumentParser(description='Pack directory into .xlsm file, excluding images')
    parser.add_argument('source_dir', help='Source directory')
    parser.add_argument('output_xlsm', help='Output .xlsm file')
    parser.add_argument('--overwrite', action='store_true', help='Overwrite existing output file')
    args = parser.parse_args()
    
    pack_xlsm(args.source_dir, args.output_xlsm, overwrite=args.overwrite)

if __name__ == '__main__':
    main()

