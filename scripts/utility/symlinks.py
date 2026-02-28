#!/usr/bin/env python3
"""
Script to replace directories with symbolic links.
Copies directory contents to a destination and creates symlinks pointing to the new locations.
"""

import os
import sys
import shutil
import argparse
from pathlib import Path


def replace_with_symlink(source_dir, dest_base):
    """Replace a directory with a symlink pointing to its new location."""
    source_path = Path(source_dir)

    if not source_path.exists() and not source_path.is_symlink():
        print(f"Error: Source directory '{source_dir}' does not exist")
        return False

    # Create expected destination path
    dest_path = Path(dest_base) / source_path.name

    # Check if source is already a symlink
    if source_path.is_symlink():
        try:
            current_target = source_path.resolve()
            expected_target = dest_path.resolve()

            if current_target == expected_target:
                print(
                    f"Skipping '{source_dir}': already a symlink pointing to correct destination"
                )
                return True
            else:
                print(
                    f"'{source_dir}' is a symlink but points to '{current_target}' instead of '{expected_target}'"
                )
                print(f"Removing existing symlink...")
                source_path.unlink()
        except Exception as e:
            print(f"Error checking symlink target: {e}")
            return False
    elif not source_path.is_dir():
        print(f"Error: '{source_dir}' exists but is not a directory")
        return False

    # If we get here, either it wasn't a symlink or it was pointing to wrong place
    source_path_resolved = (
        source_path.resolve() if source_path.exists() else source_path
    )

    # Create destination directory structure if it doesn't exist
    dest_path.parent.mkdir(parents=True, exist_ok=True)

    # Only copy if source exists (it might have been an incorrect symlink we removed)
    if source_path.exists():
        # Copy the directory contents to destination
        print(f"Copying '{source_dir}' to '{dest_path}'...")
        try:
            if dest_path.exists():
                print(
                    f"Warning: Destination '{dest_path}' already exists, removing it first"
                )
                shutil.rmtree(dest_path)
            shutil.copytree(source_path_resolved, dest_path)
        except Exception as e:
            print(f"Error copying directory: {e}")
            return False

        # Remove the original directory
        print(f"Removing original directory '{source_dir}'...")
        try:
            shutil.rmtree(source_path_resolved)
        except Exception as e:
            print(f"Error removing original directory: {e}")
            return False
    elif not dest_path.exists():
        print(
            f"Error: Neither source '{source_dir}' nor destination '{dest_path}' exists"
        )
        return False

    # Create symlink pointing to the new location
    print(f"Creating symlink from '{source_dir}' to '{dest_path}'...")
    try:
        # Use relative path for the symlink if possible
        try:
            rel_dest = os.path.relpath(dest_path, source_path.parent)
            os.symlink(rel_dest, source_path)
        except ValueError:
            # If on different drives/mount points, use absolute path
            os.symlink(str(dest_path), source_path)
        print(f"Successfully created symlink: {source_dir} -> {dest_path}")
        return True
    except Exception as e:
        print(f"Error creating symlink: {e}")
        # Try to restore the original directory if we had copied it
        if source_path_resolved.exists():
            print("Attempting to restore original directory...")
            try:
                shutil.copytree(dest_path, source_path)
                shutil.rmtree(dest_path)
                print("Original directory restored")
            except Exception as restore_error:
                print(f"Failed to restore original directory: {restore_error}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Replace directories with symbolic links pointing to a new location"
    )
    parser.add_argument(
        "directories",
        nargs="+",
        help="List of directories to replace with symlinks (relative paths)",
    )
    parser.add_argument(
        "-d",
        "--destination",
        required=True,
        help="Destination directory where contents will be copied",
    )
    parser.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Force operation even if some directories fail",
    )

    args = parser.parse_args()

    success_count = 0
    fail_count = 0

    for directory in args.directories:
        print(f"\nProcessing '{directory}'...")
        if replace_with_symlink(directory, args.destination):
            success_count += 1
        else:
            fail_count += 1
            if not args.force:
                print(
                    "\nAborting due to error. Use --force to continue with remaining directories."
                )
                sys.exit(1)

    print(f"\n{'='*50}")
    print(f"Summary: {success_count} succeeded, {fail_count} failed")

    if fail_count > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
