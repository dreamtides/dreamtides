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
import signal
import argparse

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
        print(f"Run Duration: {duration:.2f} seconds")

        if failed > 0:
            print("\nFailed Tests:")
            for test_case in test_run.findall('.//test-case[@result="Failed"]'):
                name = test_case.get('name', 'Unknown')
                print(f"- {name}")
            exit(1)
    except ET.ParseError as e:
        print(f"Error parsing test results: {e}")
        exit(1)
    except FileNotFoundError:
        print("Test results file not found")
        exit(1)

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

def check_rsync_available():
    try:
        subprocess.run(["rsync", "--version"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
        return True
    except (subprocess.SubprocessError, FileNotFoundError):
        return False

def sync_project_to_temp(project_root):
    home_dir = Path.home()
    final_project_path = home_dir / "unity_tests/test_client"

    temp_parent = Path(home_dir / "unity_tests")
    temp_parent.mkdir(parents=True, exist_ok=True)

    print(f"Syncing project to {final_project_path}...")

    source_path = str(project_root) + "/"
    rsync_cmd = [
        "rsync",
        "--delete",
        "--stats",
        "--copy-links",
        "-avqr",
        source_path,
        str(final_project_path)
    ]

    print(rsync_cmd)

    try:
        subprocess.run(
            rsync_cmd,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        print("Sync completed successfully to", final_project_path)

        return str(final_project_path)
    except subprocess.CalledProcessError as e:
        print(f"Error syncing project: {e}")
        print(f"stderr: {e.stderr}")
        sys.exit(1)
    except (shutil.Error, OSError) as e:
        print(f"Error moving project directory: {e}")
        sys.exit(1)

def read_previous_run_time(test_output_dir):
    run_time_file = test_output_dir / "run_time.txt"
    if run_time_file.exists():
        try:
            with open(run_time_file, 'r') as f:
                return float(f.read().strip())
        except (ValueError, IOError):
            pass
    print("Previous run time not found, returning 300 seconds")
    return 300

def write_run_time(test_output_dir, run_time):
    run_time_file = test_output_dir / "run_time.txt"
    with open(run_time_file, 'w') as f:
        f.write(str(run_time))

def format_time(seconds):
    minutes = int(seconds // 60)
    seconds = int(seconds % 60)
    return f"{minutes:02d}:{seconds:02d}"

def display_countdown(estimated_time, start_time):
    while True:
        elapsed = time.time() - start_time
        remaining = max(0, estimated_time - elapsed)
        print(f"\rEstimated time remaining: {format_time(remaining)}", end="", flush=True)
        if remaining <= 0:
            break
        time.sleep(1)

def monitor_log_file(log_file_path, estimated_time, start_time):
    """Monitor log file and show countdown with [*] indicator for log changes."""
    previous_size = 0
    last_update = time.time()

    while True:
        current_time = time.time()
        elapsed = current_time - start_time
        remaining = max(0, estimated_time - elapsed)

        log_changed = False
        if os.path.exists(log_file_path):
            current_size = os.path.getsize(log_file_path)
            if current_size > previous_size:
                log_changed = True
                previous_size = current_size

        if current_time - last_update >= 1.0:
            indicator = "[*]" if log_changed else " > "
            print(f"\r{indicator} Estimated time remaining: {format_time(remaining)}", end="", flush=True)
            last_update = current_time

        time.sleep(0.1)

def main():
    parser = argparse.ArgumentParser(description="Run Unity tests or sync project files.")
    parser.add_argument("--sync-only", action="store_true", help="Only sync project files without running tests")
    args = parser.parse_args()

    if args.sync_only:
        print("Running in sync-only mode...")
    else:
        print("Starting Unity tests...")

    if not check_rsync_available():
        print("Error: rsync binary not found on the system")
        print("Please install rsync before running tests")
        sys.exit(1)

    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    test_output_dir = project_root / "test_output"

    previous_run_time = 300  # Default to 5 minutes

    if not args.sync_only:
        previous_run_time = read_previous_run_time(test_output_dir)
        if previous_run_time:
            print(f"Previous test run took {format_time(previous_run_time)}")

        if test_output_dir.exists():
            shutil.rmtree(test_output_dir)
        test_output_dir.mkdir(exist_ok=True)

    project_path = sync_project_to_temp(project_root)

    if args.sync_only:
        print("Project sync completed. Exiting without running tests.")
        sys.exit(0)

    unity_path = get_unity_path()
    test_results = str(test_output_dir / "results.xml")
    log_file = str(test_output_dir / "logs.txt")

    unity_args = [
        "-runTests",
        "-batchmode",
        f"-projectPath", project_path,
        f"-testResults", test_results,
        "-testPlatform", "PlayMode",
        f"-logFile", log_file
    ]

    try:
        print(f"{unity_path} \\\n ", " \\\n  ".join(unity_args))

        start_time = time.time()

        # Start log file monitoring in a separate thread
        monitor_thread = threading.Thread(
            target=monitor_log_file,
            args=(log_file, previous_run_time, start_time),
            daemon=True
        )
        monitor_thread.start()

        # Create a process group for the Unity process
        process = subprocess.Popen(
            executable=unity_path,
            args=unity_args,
            preexec_fn=os.setsid if os.name != 'nt' else None
        )

        # Function to handle termination signals
        def signal_handler(signum, frame):
            print("\nTerminating Unity process...")
            if os.name == 'nt':
                process.terminate()
            else:
                os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            sys.exit(0)

        # Register signal handlers
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

        # Wait for the process to complete
        process.wait()

        end_time = time.time()
        total_time = end_time - start_time

        # Save the run time for future reference
        write_run_time(test_output_dir, total_time)

        print("\n")

        if Path(test_results).exists():
            print("Unity tests completed")
            print(f"Total execution time: {format_time(total_time)}")
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
