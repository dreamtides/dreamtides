#!/usr/bin/env python3
"""
XLSM/FODS Conversion Script

This script converts XLSM (Excel with macros) files to FODS (Flat OpenDocument Spreadsheet)
format using LibreOffice command-line tools.

Usage Examples:
    # Convert XLSM to FODS
    python3 client/scripts/xlsm_fods_converter.py \
        --input client/Assets/StreamingAssets/Tabula.xlsm \
        --output client/Assets/StreamingAssets/Tabula.fods
"""

import sys
import subprocess
import argparse
import shutil
from pathlib import Path
import tempfile

def find_libreoffice():
    possible_paths = [
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        "/usr/bin/libreoffice",
        "/usr/bin/soffice",
        "libreoffice",
        "soffice",
    ]
    
    for path in possible_paths:
        if path in ["libreoffice", "soffice"]:
            if shutil.which(path):
                return path
        else:
            if Path(path).exists():
                return path
    
    return None

def convert_file(input_path, output_dir):
    libreoffice = find_libreoffice()
    if not libreoffice:
        sys.stderr.write("Error: LibreOffice not found. Please install LibreOffice.\n")
        sys.exit(1)
    
    input_path = Path(input_path).resolve()
    if not input_path.exists():
        sys.stderr.write(f"Error: Input file does not exist: {input_path}\n")
        sys.exit(1)
    
    output_dir = Path(output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    
    cmd = [
        libreoffice,
        "--headless",
        "--nologo",
        "--nodefault",
        "--nofirststartwizard",
        "--convert-to", "fods",
        "--outdir", str(output_dir),
        str(input_path)
    ]
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True
        )
        
        input_stem = input_path.stem
        expected_output = output_dir / f"{input_stem}.fods"
        
        if not expected_output.exists():
            sys.stderr.write(f"Error: Expected output file does not exist: {expected_output}\n")
            sys.stderr.write(f"LibreOffice stdout: {result.stdout}\n")
            sys.stderr.write(f"LibreOffice stderr: {result.stderr}\n")
            sys.exit(1)
        
        return expected_output
    except subprocess.CalledProcessError as e:
        sys.stderr.write(f"Error: LibreOffice conversion failed\n")
        sys.stderr.write(f"Command: {' '.join(cmd)}\n")
        sys.stderr.write(f"Return code: {e.returncode}\n")
        sys.stderr.write(f"Stdout: {e.stdout}\n")
        sys.stderr.write(f"Stderr: {e.stderr}\n")
        sys.exit(1)

def xlsm_to_fods(input_path, output_path):
    output_dir = Path(output_path).parent
    output_name = Path(output_path).name
    
    with tempfile.TemporaryDirectory() as temp_dir:
        converted = convert_file(input_path, temp_dir)
        
        final_output = output_dir / output_name
        shutil.move(str(converted), str(final_output))
        return final_output

def main():
    parser = argparse.ArgumentParser(
        description='Convert XLSM files to FODS format using LibreOffice',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        '--input',
        required=True,
        help='Input XLSM file path'
    )
    parser.add_argument(
        '--output',
        required=True,
        help='Output FODS file path'
    )
    
    args = parser.parse_args()
    
    result = xlsm_to_fods(args.input, args.output)
    print(f"Conversion complete: {result}")

if __name__ == '__main__':
    main()
