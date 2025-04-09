#!/usr/bin/env python3

import subprocess
import shutil
from pathlib import Path

def get_unity_path():
    hub_path = Path("/Applications/Unity/Hub/Editor")
    if not hub_path.exists():
        raise FileNotFoundError("Unity Hub directory not found")
    
    versions = []
    for version_dir in hub_path.iterdir():
        if version_dir.is_dir():
            try:
                version = version_dir.name
                if version.startswith("20") or version[0].isdigit():
                    versions.append(version)
            except:
                continue
    
    if not versions:
        raise FileNotFoundError("No Unity versions found in Hub directory")
    
    highest_version = sorted(versions, reverse=True)[0]
    unity_path = hub_path / highest_version / "Unity.app/Contents/MacOS/Unity"
    
    if not unity_path.exists():
        raise FileNotFoundError(f"Unity executable not found at {unity_path}")
    
    return str(unity_path)

def main():
    print("Starting Unity tests...")
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    test_output_dir = project_root / "test_output"
    
    if test_output_dir.exists():
        shutil.rmtree(test_output_dir)
    test_output_dir.mkdir(exist_ok=True)
    
    unity_path = get_unity_path()
    project_path = str(project_root)
    test_results = str(test_output_dir / "results.xml")
    log_file = str(test_output_dir / "logs.txt")
    
    args = [
        "-runTests",
        "-batchmode",
        f"-projectPath", project_path,
        f"-testResults", test_results,
        "-testPlatform", "PlayMode",
        f"-logFile", log_file
    ]

    try:
        print(f"{unity_path} \\\n ", " \\\n  ".join(args))
        subprocess.run(executable=unity_path, args=args, check=True)
        if Path(test_results).exists():
            print("Unity tests completed successfully")
        else:
            print("Error: Test results file not found")
            exit(1)
    except subprocess.CalledProcessError as e:
        print(f"Error running Unity tests: {e}")
        exit(1)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        exit(1)

if __name__ == "__main__":
    main() 