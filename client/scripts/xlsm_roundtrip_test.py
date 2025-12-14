#!/usr/bin/env python3
import sys
import argparse
import tempfile
import shutil
from pathlib import Path
from xlsm_extract import extract_xlsm
from xlsm_pack import pack_xlsm

def roundtrip_test(input_xlsm, output_xlsm, keep_temp=False):
    input_path = Path(input_xlsm)
    output_path = Path(output_xlsm)
    
    if not input_path.exists():
        sys.stderr.write(f"Error: Input file not found: {input_path}\n")
        sys.exit(1)
    
    temp_dir = Path(tempfile.mkdtemp(prefix='xlsm_roundtrip_'))
    
    try:
        sys.stdout.write(f"Extracting {input_path} to temporary directory...\n")
        extract_xlsm(input_path, temp_dir, clean=True)
        
        sys.stdout.write(f"Packing to {output_path}...\n")
        pack_xlsm(temp_dir, output_path, overwrite=True)
        
        sys.stdout.write(f"Round-trip test complete: {output_path}\n")
        
        if keep_temp:
            sys.stdout.write(f"Temporary directory kept at: {temp_dir}\n")
        else:
            shutil.rmtree(temp_dir)
            sys.stdout.write("Temporary directory cleaned up.\n")
            
    except Exception as e:
        if temp_dir.exists():
            if keep_temp:
                sys.stderr.write(f"Error occurred. Temporary directory kept at: {temp_dir}\n")
            else:
                shutil.rmtree(temp_dir)
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description='Round-trip test: extract and repack .xlsm file')
    parser.add_argument('input_xlsm', help='Input .xlsm file')
    parser.add_argument('output_xlsm', help='Output .xlsm file')
    parser.add_argument('--keep-temp', action='store_true', help='Keep temporary extraction directory for inspection')
    args = parser.parse_args()
    
    roundtrip_test(args.input_xlsm, args.output_xlsm, keep_temp=args.keep_temp)

if __name__ == '__main__':
    main()

