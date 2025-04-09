#!/usr/bin/env python3

import subprocess
import shutil
import platform
import xml.etree.ElementTree as ET
from pathlib import Path

def find_highest_unity_version(hub_path):
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
    
    return sorted(versions, reverse=True)[0]

def get_unity_path_osx():
    hub_path = Path("/Applications/Unity/Hub/Editor")
    highest_version = find_highest_unity_version(hub_path)
    unity_path = hub_path / highest_version / "Unity.app/Contents/MacOS/Unity"
    
    if not unity_path.exists():
        raise FileNotFoundError(f"Unity executable not found at {unity_path}")
    
    return str(unity_path)

def get_unity_path_windows():
    hub_path = Path("C:/Program Files/Unity/Hub/Editor")
    highest_version = find_highest_unity_version(hub_path)
    unity_path = hub_path / highest_version / "Editor/Unity.exe"
    
    if not unity_path.exists():
        raise FileNotFoundError(f"Unity executable not found at {unity_path}")
    
    return str(unity_path)

def get_unity_path():
    system = platform.system()
    if system == "Darwin":
        return get_unity_path_osx()
    elif system == "Windows":
        return get_unity_path_windows()
    else:
        raise OSError(f"Unsupported operating system: {system}")

def print_test_summary(results_path):
    try:
        tree = ET.parse(results_path)
        test_run = tree.getroot()
        total = int(test_run.get('total', 0))
        passed = int(test_run.get('passed', 0))
        failed = int(test_run.get('failed', 0))
        duration = float(test_run.get('duration', 0))
        
        print("\nTest Results Summary:")
        print(f"Total Tests: {total}")
        print(f"Passed: {passed}")
        print(f"Failed: {failed}")
        print(f"Duration: {duration:.2f} seconds")
        
        if failed > 0:
            print("\nFailed Tests:")
            for test_case in test_run.findall('.//test-case[@result="Failed"]'):
                name = test_case.get('name', 'Unknown')
                print(f"- {name}")
    except ET.ParseError as e:
        print(f"Error parsing test results: {e}")
    except FileNotFoundError:
        print("Test results file not found")

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
            print("Unity tests completed")
            print_test_summary(test_results)
        else:
            print("Error: Test results file not found")
            exit(1)
    except subprocess.CalledProcessError as e:
        print(f"Error running Unity tests: {e}")
        exit(1)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        exit(1)
    except OSError as e:
        print(f"Error: {e}")
        exit(1)

if __name__ == "__main__":
    main() 