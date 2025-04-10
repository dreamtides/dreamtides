#!/usr/bin/env python3

import subprocess
import shutil
import platform
import xml.etree.ElementTree as ET
from pathlib import Path
import socket
import sys
import time
import os
import threading

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

def print_detailed_failures(results_path):
    try:
        tree = ET.parse(results_path)
        test_run = tree.getroot()
        
        print("\nDetailed Error Information:")
        
        # Find all failed test cases
        for test_case in test_run.findall('.//test-case[@result="Failed"]'):
            name = test_case.get('name', 'Unknown')
            print(f"\n- {name}")
            
            # Extract error message from the failure element
            failure = test_case.find('./failure/message')
            if failure is not None and failure.text:
                # Skip the "One or more child tests had errors" generic messages
                if "One or more child tests had errors" not in failure.text:
                    # Clean up the message - remove CDATA wrapper if present
                    message = failure.text
                    if message.startswith('<![CDATA[') and message.endswith(']]>'):
                        message = message[9:-3]
                    print(f"  Error: {message}")
        
    except ET.ParseError as e:
        print(f"Error parsing test results for detailed failures: {e}")
    except FileNotFoundError:
        print("Test results file not found")

def is_port_in_use(port):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.connect(('localhost', port))
            return True
        except socket.error:
            return False

def check_rsync_available():
    try:
        subprocess.run(["rsync", "--version"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        return True
    except (subprocess.SubprocessError, FileNotFoundError):
        return False

def sync_project_to_temp(project_root):
    temp_project_path = Path("/tmp/unity_tests/client")
    
    # Ensure the parent directory exists
    temp_parent = temp_project_path.parent
    temp_parent.mkdir(parents=True, exist_ok=True)
    
    print(f"Syncing project to {temp_project_path}...")
    
    try:
        result = subprocess.run(
            ["rsync", "--delete", "--stats", "-avqr", str(project_root), "/tmp/unity_tests/"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        print("Sync completed successfully")
        return str(temp_project_path)
    except subprocess.CalledProcessError as e:
        print(f"Error syncing project: {e}")
        print(f"stderr: {e.stderr}")
        sys.exit(1)

def monitor_log_file(log_file_path):
    """Monitor log file and print a dot for each update."""
    previous_size = 0
    
    while True:
        if os.path.exists(log_file_path):
            current_size = os.path.getsize(log_file_path)
            if current_size > previous_size:
                print(".", end="", flush=True)
                previous_size = current_size
        time.sleep(0.5)

def main():
    print("Starting Unity tests...")
    
    if not is_port_in_use(26598):
        print("Error: No server listening on port 26598")
        print("Make sure the server is running before executing tests")
        sys.exit(1)

    if not check_rsync_available():
        print("Error: rsync binary not found on the system")
        print("Please install rsync before running tests")
        sys.exit(1)

    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    test_output_dir = project_root / "test_output"
    
    if test_output_dir.exists():
        shutil.rmtree(test_output_dir)
    test_output_dir.mkdir(exist_ok=True)
    
    # Sync project to temp directory
    project_path = sync_project_to_temp(project_root)
    
    unity_path = get_unity_path()
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
        
        # Start log file monitoring in a separate thread
        monitor_thread = threading.Thread(target=monitor_log_file, args=(log_file,), daemon=True)
        monitor_thread.start()
        
        subprocess.run(executable=unity_path, args=args, check=True)
        
        print("\n")
        
        if Path(test_results).exists():
            print("Unity tests completed")
            print_test_summary(test_results)
        else:
            print("Error: Test results file not found")
            exit(1)
    except subprocess.CalledProcessError as e:
        print(f"\nError running Unity tests: {e}")

        # Check if we have test results to parse
        if Path(test_results).exists():
            print("Tests failed but results file was generated.")
            print_test_summary(test_results)
            print_detailed_failures(test_results) 

        exit(1)
    except FileNotFoundError as e:
        print(f"\nError: {e}")
        exit(1)
    except OSError as e:
        print(f"\nError: {e}")
        exit(1)

if __name__ == "__main__":
    main() 