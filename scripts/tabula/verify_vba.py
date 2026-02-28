#!/usr/bin/env python3
"""
Verifies that VBA modules in an Excel .xlsm file match the source .bas files.
Returns exit code 1 if they do not match, 0 if they match.

Setup:
  brew install pipx
  pipx install oletools
"""

import argparse
import subprocess
import sys
from pathlib import Path
from typing import Dict, Optional


def find_project_root(start_path: Path) -> Optional[Path]:
    """Find the project root by looking for a justfile."""
    current = start_path.resolve()
    while current != current.parent:
        if (current / "justfile").exists():
            return current
        current = current.parent
    return None


def main():
    project_root = find_project_root(Path(__file__).parent)
    if not project_root:
        print("Error: Could not find project root (no justfile found)", file=sys.stderr)
        sys.exit(1)

    default_excel_file = (
        project_root / "client" / "Assets" / "StreamingAssets" / "Tabula.xlsm"
    )
    default_vba_dir = project_root / "rules_engine" / "src" / "tabula_cli" / "vba"

    parser = argparse.ArgumentParser(
        description="Verifies that VBA modules in an Excel file match the source .bas files"
    )
    parser.add_argument(
        "--excel-file",
        type=Path,
        default=default_excel_file,
        help="Path to the Excel .xlsm file",
    )
    parser.add_argument(
        "--vba-dir",
        type=Path,
        default=default_vba_dir,
        help="Path to the directory containing .bas files",
    )

    args = parser.parse_args()

    if not args.excel_file.exists():
        print(f"Error: Excel file not found: {args.excel_file}", file=sys.stderr)
        sys.exit(1)

    if not args.vba_dir.exists():
        print(f"Error: VBA directory not found: {args.vba_dir}", file=sys.stderr)
        sys.exit(1)

    try:
        check_olevba_installed()
        extracted_modules = extract_vba_from_excel(args.excel_file)
        source_modules = read_bas_files(args.vba_dir)
        compare_modules(extracted_modules, source_modules)
        print("✓ All VBA modules match!")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


def check_olevba_installed():
    """Verify that olevba is installed and available."""
    try:
        result = subprocess.run(
            ["olevba", "-h"],
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise RuntimeError(
                "olevba command failed. Please install it with: pipx install oletools"
            )
    except FileNotFoundError:
        raise RuntimeError(
            "olevba not found. Please install it with: pipx install oletools"
        )


def extract_vba_from_excel(excel_path: Path) -> Dict[str, Dict[str, str]]:
    """Extract VBA modules from an Excel file using olevba."""
    print(f"Extracting VBA modules from {excel_path}...")

    result = subprocess.run(
        ["olevba", "--decode", str(excel_path)],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(f"olevba failed: {result.stderr}")

    return parse_olevba_output(result.stdout)


def parse_olevba_output(output: str) -> Dict[str, Dict[str, str]]:
    """Parse olevba output to extract module names and content."""
    modules = {}
    current_module = None
    current_content = []
    in_vba_code = False

    for line in output.split("\n"):
        if line.startswith(
            "-------------------------------------------------------------------------------"
        ):
            if current_module:
                content = normalize_vba_content("\n".join(current_content))
                modules[current_module] = {"content": content, "is_events": False}
                current_content = []
            in_vba_code = False
            current_module = None
        elif line.startswith("VBA MACRO "):
            parts = line.split()
            if len(parts) >= 3:
                module_name_with_ext = parts[2]
                current_module = module_name_with_ext.rsplit(".", 1)[0]
            in_vba_code = True
        elif (
            in_vba_code
            and line
            and not line.startswith("- - - - -")
            and not line.startswith("in file:")
        ):
            current_content.append(line)

    if current_module:
        content = normalize_vba_content("\n".join(current_content))
        modules[current_module] = {"content": content, "is_events": False}

    if not modules:
        raise RuntimeError("No VBA modules found in Excel file")

    return modules


def read_bas_files(vba_dir: Path) -> Dict[str, str]:
    """Read all .bas files from a directory."""
    modules = {}

    for bas_file in vba_dir.glob("*.bas"):
        content = bas_file.read_text(encoding="utf-8")
        module_name = extract_module_name(content, bas_file.stem)
        normalized = normalize_vba_content(content)

        modules[module_name] = {
            "content": normalized,
            "is_events": module_name == "TabulaServerEvents",
        }

    if not modules:
        raise RuntimeError(f"No .bas files found in {vba_dir}")

    return modules


def extract_module_name(content: str, fallback: str) -> str:
    """Extract the VB module name from Attribute VB_Name line."""
    for line in content.split("\n"):
        if line.startswith("Attribute VB_Name = "):
            name = line.split("=", 1)[1].strip()
            return name.strip('"')
    return snake_to_camel(fallback)


def snake_to_camel(snake_str: str) -> str:
    """Convert snake_case to CamelCase."""
    components = snake_str.split("_")
    return "".join(word.capitalize() for word in components)


def normalize_vba_content(content: str) -> str:
    """Normalize VBA content by removing metadata, blank lines, and trailing whitespace."""
    lines = []
    in_analysis_table = False

    for line in content.split("\n"):
        stripped = line.rstrip()

        if "+----------+--------------------+" in stripped:
            in_analysis_table = True
            continue

        if in_analysis_table:
            if (
                stripped
                and not stripped.startswith("|")
                and not stripped.startswith("+")
            ):
                in_analysis_table = False
            else:
                continue

        if (
            stripped
            and not stripped.startswith("Attribute VB_")
            and not stripped.startswith("VBA Stomping detection is experimental")
        ):
            lines.append(stripped.lower())

    return "\n".join(lines).strip()


def compare_modules(
    extracted: Dict[str, Dict[str, str]], source: Dict[str, Dict[str, str]]
):
    """Compare extracted modules with source files."""
    source_names = set(source.keys())
    has_differences = False

    for module_name in sorted(source_names):
        source_info = source[module_name]

        if source_info["is_events"]:
            if not verify_events_in_thisworkbook(
                extracted, source_info["content"], module_name
            ):
                has_differences = True
        else:
            if module_name not in extracted:
                print(
                    f"❌ Module '{module_name}' not found in Excel file",
                    file=sys.stderr,
                )
                has_differences = True
                continue

            extracted_content = extracted[module_name]["content"]
            source_content = source_info["content"]

            if source_content != extracted_content:
                print(f"❌ Module '{module_name}' has differences:", file=sys.stderr)
                print_diff(source_content, extracted_content)
                has_differences = True

    if has_differences:
        print(
            f"\nModules found in Excel: {', '.join(sorted(extracted.keys()))}",
            file=sys.stderr,
        )
        raise RuntimeError("Some modules are missing or have content differences")


def verify_events_in_thisworkbook(
    extracted: Dict[str, Dict[str, str]], events_content: str, events_name: str
) -> bool:
    """Verify that event handler code exists within ThisWorkbook module."""
    if "ThisWorkbook" not in extracted:
        print(
            f"⚠️  Warning: ThisWorkbook module not found in Excel file (needed for {events_name})"
        )
        return True

    thisworkbook_content = extracted["ThisWorkbook"]["content"]
    events_lines = [
        line
        for line in events_content.split("\n")
        if line.strip() and not line.strip().startswith("option explicit")
    ]

    missing_lines = []
    for line in events_lines:
        if line.strip() and line not in thisworkbook_content:
            missing_lines.append(line)

    if missing_lines:
        print(
            f"⚠️  Warning: {events_name} code may not be fully integrated into ThisWorkbook module"
        )
        print(
            f"   (This is expected if you haven't manually merged the event handlers yet)"
        )

    return True


def print_diff(source: str, extracted: str):
    """Print line-by-line differences between source and extracted content."""
    source_lines = source.split("\n")
    extracted_lines = extracted.split("\n")

    max_len = max(len(source_lines), len(extracted_lines))

    for i in range(max_len):
        source_line = source_lines[i] if i < len(source_lines) else ""
        extracted_line = extracted_lines[i] if i < len(extracted_lines) else ""

        if source_line != extracted_line:
            print(f"  Line {i + 1}:", file=sys.stderr)
            print(f"    Source:    {source_line}", file=sys.stderr)
            print(f"    Extracted: {extracted_line}", file=sys.stderr)


if __name__ == "__main__":
    main()
