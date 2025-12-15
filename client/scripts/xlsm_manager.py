#!/usr/bin/env python3
"""
XLSM Manager: Tools for extracting, reconstructing, and managing Excel XLSM files
for version control with git.

This script handles:
1. Extracting XLSM files to directories for git storage
2. Reconstructing XLSM files from directories
3. Managing embedded images (replacing with placeholders for git, restoring from cache)

The key challenge is that Excel is extremely sensitive to ZIP file format details.
This script carefully preserves:
- File ordering within the ZIP archive
- Compression methods (DEFLATE for XML, STORED for images)
- Normalized timestamps (1980-01-01)
- Exact binary content of all files
"""

import argparse
import hashlib
import json
import os
import shutil
import struct
import sys
import zipfile
from datetime import datetime
from pathlib import Path


XLSM_TIMESTAMP = (1980, 1, 1, 0, 0, 0)

PLACEHOLDER_JPEG = bytes([
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
    0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
    0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
    0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
    0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
    0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
    0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
    0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
    0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06,
    0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
    0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72,
    0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45,
    0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
    0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3,
    0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
    0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9,
    0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
    0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4,
    0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01,
    0x00, 0x00, 0x3F, 0x00, 0xFB, 0xD5, 0xDB, 0x20, 0xA8, 0xA2, 0x80, 0x3F,
    0xFF, 0xD9
])

MANIFEST_FILENAME = "_xlsm_manifest.json"
IMAGE_CACHE_DIRNAME = "xlsm_image_cache"


SAFE_TO_FORMAT_XML = {
    '[Content_Types].xml',
    'xl/workbook.xml',
    'xl/styles.xml',
    'xl/calcChain.xml',
    'xl/sharedStrings.xml',
    'docProps/core.xml',
    'docProps/app.xml',
}


def is_safe_to_format(filename: str) -> bool:
    if filename in SAFE_TO_FORMAT_XML:
        return True
    if filename.endswith('.rels'):
        return True
    if filename.startswith('xl/worksheets/') and filename.endswith('.xml'):
        return True
    if filename.startswith('xl/tables/') and filename.endswith('.xml'):
        return True
    return False


def minimal_xml_format(xml_bytes: bytes) -> bytes:
    import re
    
    try:
        text = xml_bytes.decode('utf-8')
        
        result = re.sub(r'>(\s*)<', r'>\n<', text)
        
        return result.encode('utf-8')
    except Exception:
        return xml_bytes


def get_compression_for_file(filename: str) -> int:
    lower = filename.lower()
    if lower.endswith(('.jpg', '.jpeg', '.png', '.gif', '.emf', '.wmf')):
        return zipfile.ZIP_STORED
    return zipfile.ZIP_DEFLATED


def extract_xlsm_to_directory(
    xlsm_path: Path,
    output_dir: Path,
    image_cache_dir: Path | None = None,
    strip_images: bool = True
) -> None:
    """
    Extract an XLSM file to a directory structure.
    
    Args:
        xlsm_path: Path to the source XLSM file
        output_dir: Directory to extract contents to
        image_cache_dir: Optional directory to cache original images
        strip_images: If True, replace images with placeholders
    """
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True)
    
    file_order = []
    image_manifest = {}
    
    with zipfile.ZipFile(xlsm_path, 'r') as zf:
        for info in zf.infolist():
            file_order.append(info.filename)
            
            file_path = output_dir / info.filename
            if info.filename.endswith('/'):
                file_path.mkdir(parents=True, exist_ok=True)
                continue
                
            file_path.parent.mkdir(parents=True, exist_ok=True)
            data = zf.read(info.filename)
            
            is_image = info.filename.startswith('xl/media/') and info.filename.lower().endswith(('.jpg', '.jpeg', '.png', '.gif'))
            
            if is_image and strip_images:
                file_hash = hashlib.sha256(data).hexdigest()
                image_manifest[info.filename] = {
                    'hash': file_hash,
                    'size': len(data),
                    'original_name': os.path.basename(info.filename)
                }
                
                if image_cache_dir:
                    cache_file = image_cache_dir / file_hash
                    if not cache_file.exists():
                        image_cache_dir.mkdir(parents=True, exist_ok=True)
                        cache_file.write_bytes(data)
                
                file_path.write_bytes(PLACEHOLDER_JPEG)
            else:
                if is_safe_to_format(info.filename):
                    data = minimal_xml_format(data)
                file_path.write_bytes(data)
    
    manifest = {
        'version': 1,
        'file_order': file_order,
        'images': image_manifest,
        'source_file': xlsm_path.name
    }
    
    manifest_path = output_dir / MANIFEST_FILENAME
    with open(manifest_path, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"Extracted {len(file_order)} files to {output_dir}")
    if image_manifest:
        print(f"Stripped {len(image_manifest)} images (placeholders inserted)")


def reconstruct_xlsm_from_directory(
    input_dir: Path,
    xlsm_path: Path,
    image_cache_dir: Path | None = None,
    restore_images: bool = True,
    quiet: bool = False
) -> None:
    """
    Reconstruct an XLSM file from a directory structure.
    
    Args:
        input_dir: Directory containing extracted XLSM contents
        xlsm_path: Path for the output XLSM file
        image_cache_dir: Optional directory containing cached images
        restore_images: If True, attempt to restore original images from cache
    """
    manifest_path = input_dir / MANIFEST_FILENAME
    if not manifest_path.exists():
        raise FileNotFoundError(f"Manifest not found: {manifest_path}")
    
    with open(manifest_path, 'r', encoding='utf-8') as f:
        manifest = json.load(f)
    
    file_order = manifest.get('file_order', [])
    image_manifest = manifest.get('images', {})
    
    if xlsm_path.exists():
        xlsm_path.unlink()
    
    restored_count = 0
    placeholder_count = 0
    
    with zipfile.ZipFile(xlsm_path, 'w', allowZip64=False) as zf:
        for filename in file_order:
            if filename.endswith('/'):
                continue
                
            file_path = input_dir / filename
            if not file_path.exists():
                print(f"Warning: Missing file {filename}, skipping")
                continue
            
            data = file_path.read_bytes()
            
            if filename in image_manifest and restore_images and image_cache_dir:
                image_info = image_manifest[filename]
                cache_file = image_cache_dir / image_info['hash']
                if cache_file.exists():
                    cached_data = cache_file.read_bytes()
                    if hashlib.sha256(cached_data).hexdigest() == image_info['hash']:
                        data = cached_data
                        restored_count += 1
                    else:
                        placeholder_count += 1
                else:
                    placeholder_count += 1
            elif filename in image_manifest:
                placeholder_count += 1
            
            info = zipfile.ZipInfo(filename, date_time=XLSM_TIMESTAMP)
            info.compress_type = get_compression_for_file(filename)
            
            if info.compress_type == zipfile.ZIP_DEFLATED:
                info.compress_type = zipfile.ZIP_DEFLATED
            
            zf.writestr(info, data)
    
    if not quiet:
        print(f"Created {xlsm_path}")
        if image_manifest:
            print(f"Images: {restored_count} restored from cache, {placeholder_count} using placeholders")


def roundtrip_test(xlsm_path: Path, output_path: Path, image_cache_dir: Path | None = None) -> bool:
    """
    Test round-trip conversion: XLSM -> directory -> XLSM
    
    This validates that the conversion process produces a valid XLSM file.
    """
    temp_dir = Path("/tmp/xlsm_roundtrip_test")
    extracted_dir = temp_dir / "extracted"
    
    if temp_dir.exists():
        shutil.rmtree(temp_dir)
    temp_dir.mkdir(parents=True)
    
    try:
        print(f"Step 1: Extracting {xlsm_path} to directory...")
        extract_xlsm_to_directory(
            xlsm_path,
            extracted_dir,
            image_cache_dir=image_cache_dir,
            strip_images=True
        )
        
        print(f"\nStep 2: Reconstructing XLSM to {output_path}...")
        reconstruct_xlsm_from_directory(
            extracted_dir,
            output_path,
            image_cache_dir=image_cache_dir,
            restore_images=True
        )
        
        print(f"\nStep 3: Validating output...")
        if not output_path.exists():
            print("ERROR: Output file was not created")
            return False
        
        try:
            with zipfile.ZipFile(output_path, 'r') as zf:
                bad_file = zf.testzip()
                if bad_file:
                    print(f"ERROR: Corrupt file in archive: {bad_file}")
                    return False
                
                original_files = set()
                with zipfile.ZipFile(xlsm_path, 'r') as orig_zf:
                    for info in orig_zf.infolist():
                        if not info.filename.endswith('/'):
                            original_files.add(info.filename)
                
                new_files = set()
                for info in zf.infolist():
                    if not info.filename.endswith('/'):
                        new_files.add(info.filename)
                
                missing = original_files - new_files - {MANIFEST_FILENAME}
                extra = new_files - original_files - {MANIFEST_FILENAME}
                
                if missing:
                    print(f"WARNING: Missing files: {missing}")
                if extra:
                    print(f"WARNING: Extra files: {extra}")
        except zipfile.BadZipFile as e:
            print(f"ERROR: Invalid ZIP file: {e}")
            return False
        
        print("\n" + "="*60)
        print("ROUND-TRIP TEST PASSED")
        print("="*60)
        print(f"\nOutput file: {output_path}")
        print(f"Size: {output_path.stat().st_size} bytes")
        print("\nPlease open the file in Excel to verify it's not corrupt.")
        print("If Excel reports 'needs repair', the round-trip failed.")
        
        return True
        
    finally:
        pass


def find_git_root() -> Path | None:
    current = Path.cwd()
    while current != current.parent:
        if (current / '.git').is_dir():
            return current
        current = current.parent
    return None


def get_default_paths(git_root: Path) -> tuple[Path, Path, Path]:
    xlsm_path = git_root / "client/Assets/StreamingAssets/Tabula.xlsm"
    xlsm_dir = git_root / "client/Assets/StreamingAssets/Tabula.xlsm.d"
    image_cache = git_root / ".git/xlsm_image_cache"
    return xlsm_path, xlsm_dir, image_cache


def git_pre_commit(git_root: Path | None = None) -> bool:
    if git_root is None:
        git_root = find_git_root()
    if git_root is None:
        print("ERROR: Not in a git repository")
        return False
    
    xlsm_path, xlsm_dir, image_cache = get_default_paths(git_root)
    
    if not xlsm_path.exists():
        print(f"XLSM file not found: {xlsm_path}")
        print("Skipping pre-commit XLSM processing")
        return True
    
    backup_dir = git_root / ".git/excel-backups"
    backup_dir.mkdir(parents=True, exist_ok=True)
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_filename = f"Tabula_{timestamp}.xlsm"
    backup_path = backup_dir / backup_filename
    
    shutil.copy2(xlsm_path, backup_path)
    print(f"Backed up {xlsm_path.name} to {backup_path.name}")
    
    backup_files = list(backup_dir.glob("Tabula_*.xlsm"))
    if len(backup_files) > 100:
        backup_files.sort(key=lambda p: p.stat().st_mtime)
        files_to_delete = backup_files[:len(backup_files) - 100]
        for file_to_delete in files_to_delete:
            file_to_delete.unlink()
        print(f"Deleted {len(files_to_delete)} oldest backup file(s)")

    print(f"Extracting {xlsm_path.name} to {xlsm_dir.name}...")
    extract_xlsm_to_directory(
        xlsm_path,
        xlsm_dir,
        image_cache_dir=image_cache,
        strip_images=True
    )
    return True


def git_post_checkout(git_root: Path | None = None) -> bool:
    if git_root is None:
        git_root = find_git_root()
    if git_root is None:
        print("ERROR: Not in a git repository")
        return False
    
    xlsm_path, xlsm_dir, image_cache = get_default_paths(git_root)
    
    if not xlsm_dir.exists():
        print(f"XLSM directory not found: {xlsm_dir}")
        print("Skipping post-checkout XLSM reconstruction")
        return True
    
    print(f"Reconstructing {xlsm_path} from {xlsm_dir}...")
    reconstruct_xlsm_from_directory(
        xlsm_dir,
        xlsm_path,
        image_cache_dir=image_cache,
        restore_images=True
    )
    return True


def git_setup(git_root: Path | None = None) -> bool:
    if git_root is None:
        git_root = find_git_root()
    if git_root is None:
        print("ERROR: Not in a git repository")
        return False
    
    hooks_dir = git_root / ".git/hooks"
    script_path = Path(__file__).resolve()
    
    post_checkout_hook = hooks_dir / "post-checkout"
    post_checkout_content = f'''#!/bin/bash
python3 "{script_path}" git-post-checkout
'''
    
    pre_commit_hook = hooks_dir / "pre-commit"
    pre_commit_content = f'''#!/bin/bash
python3 "{script_path}" git-pre-commit
if [ $? -ne 0 ]; then
    echo "XLSM pre-commit hook failed"
    exit 1
fi

XLSM_DIR="{git_root}/client/Assets/StreamingAssets/Tabula.xlsm.d"
if [ -d "$XLSM_DIR" ]; then
    git add "$XLSM_DIR"
fi
'''
    
    print(f"Installing git hooks in {hooks_dir}...")
    
    post_checkout_hook.write_text(post_checkout_content)
    post_checkout_hook.chmod(0o755)
    print(f"  Created {post_checkout_hook}")
    
    pre_commit_hook.write_text(pre_commit_content)
    pre_commit_hook.chmod(0o755)
    print(f"  Created {pre_commit_hook}")
    
    gitignore_path = git_root / ".gitignore"
    gitignore_entry = "client/Assets/StreamingAssets/Tabula.xlsm"
    
    if gitignore_path.exists():
        content = gitignore_path.read_text()
        if gitignore_entry not in content:
            with open(gitignore_path, 'a') as f:
                f.write(f"\n{gitignore_entry}\n")
            print(f"  Added {gitignore_entry} to .gitignore")
        else:
            print(f"  .gitignore already contains {gitignore_entry}")
    else:
        gitignore_path.write_text(f"{gitignore_entry}\n")
        print(f"  Created .gitignore with {gitignore_entry}")
    
    xlsm_path, xlsm_dir, image_cache = get_default_paths(git_root)
    
    if xlsm_path.exists() and not xlsm_dir.exists():
        print(f"\nPerforming initial extraction of {xlsm_path}...")
        extract_xlsm_to_directory(
            xlsm_path,
            xlsm_dir,
            image_cache_dir=image_cache,
            strip_images=True
        )
    
    print("\nSetup complete!")
    print("\nNext steps:")
    print(f"  1. Run: git add {xlsm_dir.relative_to(git_root)}")
    print(f"  2. Run: git add .gitignore")
    print("  3. Commit the changes")
    print("\nAfter setup:")
    print("  - Edit Tabula.xlsm in Excel as normal")
    print("  - When you commit, the directory will be updated automatically")
    print("  - When you checkout, the XLSM will be reconstructed automatically")
    
    return True


def main():
    parser = argparse.ArgumentParser(
        description="XLSM Manager: Extract, reconstruct, and manage Excel XLSM files for git"
    )
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    extract_parser = subparsers.add_parser('extract', help='Extract XLSM to directory')
    extract_parser.add_argument('xlsm_path', type=Path, help='Path to XLSM file')
    extract_parser.add_argument('output_dir', type=Path, help='Output directory')
    extract_parser.add_argument('--image-cache', type=Path, help='Directory to cache images')
    extract_parser.add_argument('--keep-images', action='store_true', help='Keep original images (no placeholders)')
    
    reconstruct_parser = subparsers.add_parser('reconstruct', help='Reconstruct XLSM from directory')
    reconstruct_parser.add_argument('input_dir', type=Path, help='Input directory')
    reconstruct_parser.add_argument('xlsm_path', type=Path, help='Output XLSM path')
    reconstruct_parser.add_argument('--image-cache', type=Path, help='Directory containing cached images')
    reconstruct_parser.add_argument('--no-restore', action='store_true', help='Do not restore images from cache')
    
    roundtrip_parser = subparsers.add_parser('roundtrip', help='Test round-trip conversion')
    roundtrip_parser.add_argument('xlsm_path', type=Path, help='Source XLSM file')
    roundtrip_parser.add_argument('output_path', type=Path, help='Output XLSM file (different from source)')
    roundtrip_parser.add_argument('--image-cache', type=Path, help='Directory to cache/restore images')
    
    subparsers.add_parser('git-pre-commit', help='Git pre-commit hook handler')
    subparsers.add_parser('git-post-checkout', help='Git post-checkout hook handler')
    subparsers.add_parser('git-setup', help='Install git hooks and configure repository')
    
    args = parser.parse_args()
    
    if args.command == 'extract':
        extract_xlsm_to_directory(
            args.xlsm_path,
            args.output_dir,
            image_cache_dir=args.image_cache,
            strip_images=not args.keep_images
        )
    elif args.command == 'reconstruct':
        reconstruct_xlsm_from_directory(
            args.input_dir,
            args.xlsm_path,
            image_cache_dir=args.image_cache,
            restore_images=not args.no_restore
        )
    elif args.command == 'roundtrip':
        if args.xlsm_path.resolve() == args.output_path.resolve():
            print("ERROR: Output path must be different from source path")
            sys.exit(1)
        success = roundtrip_test(args.xlsm_path, args.output_path, args.image_cache)
        sys.exit(0 if success else 1)
    elif args.command == 'git-pre-commit':
        success = git_pre_commit()
        sys.exit(0 if success else 1)
    elif args.command == 'git-post-checkout':
        success = git_post_checkout()
        sys.exit(0 if success else 1)
    elif args.command == 'git-setup':
        success = git_setup()
        sys.exit(0 if success else 1)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == '__main__':
    main()
