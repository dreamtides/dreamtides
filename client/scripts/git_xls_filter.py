#!/usr/bin/env python3
import sys
import zipfile
import io
import subprocess
import argparse
from pathlib import Path

PLACEHOLDER_PNG = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\tpHYs\x00\x00\x0b\x13\x00\x00\x0b\x13\x01\x00\x9a\x9c\x18\x00\x00\x00\nIDATx\x9cc\xf8\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00IEND\xaeB`\x82'
PLACEHOLDER_GIF = b'GIF89a\x01\x00\x01\x00\x00\x00\x00!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x04\x01\x00;'
PLACEHOLDER_JPG = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb\x00C\x00\x08\x06\x06\x07\x06\x05\x08\x07\x07\x07\t\t\x08\n\x0c\x14\r\x0c\x0b\x0b\x0c\x19\x12\x13\x0f\x14\x1d\x1a\x1f\x1e\x1d\x1a\x1c\x1c $.\' ",#\x1c\x1c(7),01444\x1f\'9=82<.342\xff\xc0\x00\x0b\x08\x00\x01\x00\x01\x01\x01\x11\x00\xff\xc4\x00\x14\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\xff\xc4\x00\x14\x10\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xda\x00\x08\x01\x01\x00\x00?\x00\xaa\x00\x08\xff\xd9'

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

def validate_relative_path(path_str):
    if not path_str:
        return None
    path = Path(path_str)
    if path.is_absolute():
        return None
    try:
        resolved = path.resolve()
        cwd = Path.cwd().resolve()
        if not str(resolved).startswith(str(cwd)):
            return None
    except (OSError, ValueError):
        return None
    parts = path.parts
    if '..' in parts or path_str.startswith('/'):
        return None
    return path

def get_cache_path(git_dir, relative_path):
    if not git_dir:
        return None
    cache_dir = git_dir / 'excel-image-cache'
    cache_path = cache_dir / relative_path
    return cache_path

def get_placeholder_for_extension(ext):
    ext_lower = ext.lower()
    if ext_lower in ['.png']:
        return PLACEHOLDER_PNG
    elif ext_lower in ['.gif']:
        return PLACEHOLDER_GIF
    elif ext_lower in ['.jpg', '.jpeg']:
        return PLACEHOLDER_JPG
    else:
        return PLACEHOLDER_PNG

def clean_filter(input_bytes, relative_path):
    git_dir = get_git_dir()
    if not git_dir:
        sys.stderr.write("Warning: Not in a git repository, cannot cache\n")
    else:
        validated_path = validate_relative_path(relative_path)
        if not validated_path:
            sys.stderr.write(f"Warning: Invalid path '{relative_path}', skipping cache\n")
        else:
            cache_path = get_cache_path(git_dir, validated_path)
            if cache_path:
                try:
                    parent_dir = cache_path.parent
                    if parent_dir.exists():
                        if not parent_dir.is_dir():
                            sys.stderr.write(f"Error: Cache directory path exists as file: {parent_dir}\n")
                            sys.stderr.write(f"  Remove it to enable caching: rm '{parent_dir}'\n")
                            sys.stderr.write(f"  Without cache, images cannot be restored from Git.\n")
                        else:
                            try:
                                cache_path.write_bytes(input_bytes)
                            except (OSError, PermissionError) as e:
                                sys.stderr.write(f"Warning: Cannot write cache file: {e}\n")
                    else:
                        try:
                            parent_dir.mkdir(parents=True, exist_ok=True)
                            if parent_dir.exists() and parent_dir.is_dir():
                                cache_path.write_bytes(input_bytes)
                        except FileExistsError:
                            if parent_dir.is_dir():
                                try:
                                    cache_path.write_bytes(input_bytes)
                                except (OSError, PermissionError) as e:
                                    sys.stderr.write(f"Warning: Cannot write cache file: {e}\n")
                            else:
                                sys.stderr.write(f"Error: Cache directory path exists as file: {parent_dir}\n")
                                sys.stderr.write(f"  Remove it to enable caching: rm '{parent_dir}'\n")
                                sys.stderr.write(f"  Without cache, images cannot be restored from Git.\n")
                        except (OSError, PermissionError) as e:
                            sys.stderr.write(f"Warning: Cannot create cache directory: {e}\n")
                except Exception as e:
                    sys.stderr.write(f"Warning: Unexpected error with cache: {e}, continuing without cache\n")
    
    try:
        input_zip = zipfile.ZipFile(io.BytesIO(input_bytes), 'r')
        output_buffer = io.BytesIO()
        
        with zipfile.ZipFile(output_buffer, 'w', zipfile.ZIP_DEFLATED) as output_zip:
            entries = sorted(input_zip.namelist())
            
            for entry_name in entries:
                try:
                    entry_data = input_zip.read(entry_name)
                    
                    if entry_name.startswith('xl/media/'):
                        ext = Path(entry_name).suffix
                        placeholder = get_placeholder_for_extension(ext)
                        entry_data = placeholder
                    
                    info = input_zip.getinfo(entry_name)
                    output_zip.writestr(
                        zipfile.ZipInfo(entry_name, date_time=(1980, 1, 1, 0, 0, 0)),
                        entry_data,
                        compress_type=zipfile.ZIP_DEFLATED
                    )
                except Exception as e:
                    sys.stderr.write(f"Warning: Error processing entry '{entry_name}': {e}, skipping\n")
                    continue
        
        return output_buffer.getvalue()
    except zipfile.BadZipFile:
        sys.stderr.write("Warning: Input is not a valid ZIP file, passing through unchanged\n")
        return input_bytes
    except Exception as e:
        sys.stderr.write(f"Warning: Error processing ZIP: {e}, passing through unchanged\n")
        return input_bytes

def smudge_filter(input_bytes, relative_path):
    git_dir = get_git_dir()
    if not git_dir:
        return input_bytes
    
    validated_path = validate_relative_path(relative_path)
    if not validated_path:
        return input_bytes
    
    cache_path = get_cache_path(git_dir, validated_path)
    if cache_path and cache_path.exists():
        try:
            return cache_path.read_bytes()
        except Exception as e:
            sys.stderr.write(f"Warning: Error reading cache: {e}, using stripped version\n")
            return input_bytes
    
    return input_bytes

def main():
    try:
        parser = argparse.ArgumentParser(description='Git filter for Excel .xlsm files')
        parser.add_argument('mode', choices=['clean', 'smudge'], help='Filter mode')
        parser.add_argument('filepath', nargs='?', default='', help='Relative file path (from git)')
        args = parser.parse_args()
        
        input_bytes = sys.stdin.buffer.read()
        
        if args.mode == 'clean':
            output_bytes = clean_filter(input_bytes, args.filepath)
        else:
            output_bytes = smudge_filter(input_bytes, args.filepath)
        
        sys.stdout.buffer.write(output_bytes)
        sys.stdout.buffer.flush()
    except Exception as e:
        sys.stderr.write(f"Error: {e}\n")
        if 'input_bytes' in locals():
            sys.stdout.buffer.write(input_bytes)
        else:
            sys.stdout.buffer.write(sys.stdin.buffer.read())
        sys.stdout.buffer.flush()
        sys.exit(1)

if __name__ == '__main__':
    main()

