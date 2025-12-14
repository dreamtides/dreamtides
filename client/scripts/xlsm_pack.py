#!/usr/bin/env python3
import sys
import zipfile
import argparse
import xml.etree.ElementTree as ET
from pathlib import Path

PLACEHOLDER_PNG = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\tpHYs\x00\x00\x0b\x13\x00\x00\x0b\x13\x01\x00\x9a\x9c\x18\x00\x00\x00\nIDATx\x9cc\xf8\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00IEND\xaeB`\x82'
PLACEHOLDER_JPG = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb\x00C\x00\x08\x06\x06\x07\x06\x05\x08\x07\x07\x07\t\t\x08\n\x0c\x14\r\x0c\x0b\x0b\x0c\x19\x12\x13\x0f\x14\x1d\x1a\x1f\x1e\x1d\x1a\x1c\x1c $.\' ",#\x1c\x1c(7),01444\x1f\'9=82<.342\xff\xc0\x00\x0b\x08\x00\x01\x00\x01\x01\x01\x11\x00\xff\xc4\x00\x14\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\xff\xc4\x00\x14\x10\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xda\x00\x08\x01\x01\x00\x00?\x00\xaa\x00\x08\xff\xd9'
PLACEHOLDER_GIF = b'GIF89a\x01\x00\x01\x00\x00\x00\x00!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x04\x01\x00;'

def get_placeholder_for_extension(ext):
    ext_lower = ext.lower()
    if ext_lower in ['.png']:
        return PLACEHOLDER_PNG
    elif ext_lower in ['.jpg', '.jpeg']:
        return PLACEHOLDER_JPG
    elif ext_lower in ['.gif']:
        return PLACEHOLDER_GIF
    else:
        return PLACEHOLDER_PNG

def find_referenced_media_files(source_dir):
    referenced = set()
    source_path = Path(source_dir)
    
    for rels_file in source_path.rglob('*.rels'):
        try:
            tree = ET.parse(rels_file)
            root = tree.getroot()
            for relationship in root.findall('.//{http://schemas.openxmlformats.org/package/2006/relationships}Relationship'):
                target = relationship.get('Target', '')
                rel_type = relationship.get('Type', '')
                
                if 'image' in rel_type.lower() and target.startswith('../media/'):
                    media_file = target.replace('../media/', '')
                    referenced.add(media_file)
        except Exception as e:
            sys.stderr.write(f"Warning: Could not parse {rels_file}: {e}\n")
            continue
    
    return referenced

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
    
    referenced_media = find_referenced_media_files(source_dir)
    media_dir = source_dir / 'xl' / 'media'
    
    with zipfile.ZipFile(output_xlsm, 'w', zipfile.ZIP_DEFLATED) as zip_ref:
        for file_path in sorted(source_dir.rglob('*')):
            if file_path.is_file():
                rel_path = file_path.relative_to(source_dir)
                arcname = str(rel_path).replace('\\', '/')
                
                if arcname.startswith('xl/media/') or arcname.startswith('xl/embeddings/'):
                    continue
                
                zip_ref.write(file_path, arcname)
        
        for media_file in referenced_media:
            media_path = media_dir / media_file
            arcname = f'xl/media/{media_file}'
            
            if media_path.exists() and media_path.is_file():
                zip_ref.write(media_path, arcname)
            else:
                ext = Path(media_file).suffix
                placeholder = get_placeholder_for_extension(ext)
                zip_info = zipfile.ZipInfo(arcname, date_time=(1980, 1, 1, 0, 0, 0))
                zip_ref.writestr(zip_info, placeholder, compress_type=zipfile.ZIP_DEFLATED)

def main():
    parser = argparse.ArgumentParser(description='Pack directory into .xlsm file, excluding images')
    parser.add_argument('source_dir', help='Source directory')
    parser.add_argument('output_xlsm', help='Output .xlsm file')
    parser.add_argument('--overwrite', action='store_true', help='Overwrite existing output file')
    args = parser.parse_args()
    
    pack_xlsm(args.source_dir, args.output_xlsm, overwrite=args.overwrite)

if __name__ == '__main__':
    main()

