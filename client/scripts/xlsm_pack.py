#!/usr/bin/env python3
import sys
import zipfile
import argparse
import subprocess
from pathlib import Path

PLACEHOLDER_PNG = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\tpHYs\x00\x00\x0b\x13\x00\x00\x0b\x13\x01\x00\x9a\x9c\x18\x00\x00\x00\nIDATx\x9cc\xf8\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00IEND\xaeB`\x82'

def get_git_dir():
    try:
        result = subprocess.run(
            ['git', 'rev-parse', '--git-dir'],
            capture_output=True,
            text=True,
            check=True
        )
        return Path(result.stdout.strip()).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None

def get_image_cache_path(image_name):
    git_dir = get_git_dir()
    if not git_dir:
        return None
    cache_dir = git_dir / 'xlsm-image-cache'
    cache_path = cache_dir / image_name
    return cache_path

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
                
                if arcname.startswith('xl/media/'):
                    image_name = Path(arcname).name
                    cache_path = get_image_cache_path(image_name)
                    
                    if cache_path and cache_path.exists() and cache_path.is_file():
                        zip_ref.write(cache_path, arcname)
                    else:
                        zip_info = zipfile.ZipInfo(arcname)
                        zip_info.date_time = (1980, 1, 1, 0, 0, 0)
                        zip_ref.writestr(zip_info, PLACEHOLDER_PNG)
                else:
                    zip_ref.write(file_path, arcname)

def main():
    parser = argparse.ArgumentParser(description='Pack directory into .xlsm file')
    parser.add_argument('source_dir', help='Source directory')
    parser.add_argument('output_xlsm', help='Output .xlsm file')
    parser.add_argument('--overwrite', action='store_true', help='Overwrite existing output file')
    args = parser.parse_args()
    
    pack_xlsm(args.source_dir, args.output_xlsm, overwrite=args.overwrite)

if __name__ == '__main__':
    main()
