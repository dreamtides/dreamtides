#!/usr/bin/env python3

import argparse
import os
import subprocess
import sys
import shlex
import time


IMAGE_NAME = "dreamtides-bench:ubuntu24"
CONTAINER_NAME = "dreamtides-bench"
VOLUME_CODE = "dreamtides_code"
VOLUME_TARGET = "dreamtides_target"
VOLUME_CARGO = "dreamtides_cargo"
WORKDIR = "/workspace"
CODE_SUBDIR = ""


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    RESET = "\033[0m"


def log(msg):
    print(msg)


def ok(msg):
    print(f"{Colors.GREEN}{msg}{Colors.RESET}")


def warn(msg):
    print(f"{Colors.YELLOW}{msg}{Colors.RESET}")


def err(msg):
    print(f"{Colors.RED}{msg}{Colors.RESET}", file=sys.stderr)


def project_root():
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run(cmd, check=True, capture=False):
    print(f"Running: {' '.join(cmd if isinstance(cmd, list) else shlex.split(cmd))}")
    proc = subprocess.run(cmd if isinstance(cmd, list) else shlex.split(cmd),
                          stdout=subprocess.PIPE if capture else None,
                          stderr=subprocess.STDOUT,
                          text=True)
    if capture:
        return proc
    if check and proc.returncode != 0:
        raise RuntimeError(f"Command failed ({proc.returncode}): {cmd}")
    return proc


def require_docker():
    try:
        run(["docker", "version"], check=True)
    except Exception:
        err("Docker is required but not available. Install Docker Desktop and ensure 'docker' works from the terminal.")
        sys.exit(1)


def build_image():
    log("Ensuring Linux benchmark image exists…")
    result = run(["docker", "images", "-q", IMAGE_NAME], check=False, capture=True)
    if result.returncode == 0 and result.stdout.strip():
        ok("Image found")
        return
    dockerfile = os.path.join(project_root(), "docker", "bench.Dockerfile")
    log("Building image; this may take a few minutes the first time…")
    run(["docker", "build", "-f", dockerfile, "-t", IMAGE_NAME, project_root()])
    ok("Image built")


def ensure_volumes():
    for vol in [VOLUME_CODE, VOLUME_TARGET, VOLUME_CARGO]:
        result = run(["docker", "volume", "ls", "-q", "--filter", f"name=^{vol}$"], check=False, capture=True)
        if not result.stdout.strip():
            run(["docker", "volume", "create", vol])


def ensure_container():
    result = run(["docker", "ps", "-a", "--filter", f"name=^{CONTAINER_NAME}$", "--format", "{{.Status}}"], check=False, capture=True)
    if result.stdout:
        run(["docker", "rm", "-f", CONTAINER_NAME], check=False)

    run([
        "docker", "run", "-d",
        "--name", CONTAINER_NAME,
        "-e", "CARGO_TARGET_DIR=/cache/target",
        "-v", f"{VOLUME_CODE}:/workspace",
        "-v", f"{VOLUME_TARGET}:/cache/target",
        "-v", f"{VOLUME_CARGO}:/usr/local/cargo",
        IMAGE_NAME
    ])

    time.sleep(1)
    run(["docker", "exec", "-u", "root", CONTAINER_NAME, "bash", "-lc",
         "mkdir -p /cache/target && chown -R runner:runner /workspace /cache /usr/local/cargo || true"], check=False)


def rsync_code(verbose=False):
    host_dir = project_root()
    excludes = [
        ".git/",
        "target/",
        "**/target/",
        ".cargo/",
        "**/.git/",
        "**/.DS_Store",
        "**/.idea/",
        "**/.vscode/",
        "**/node_modules/",
    ]
    rsync_cmd = [
        "rsync", "-az", "--delete"
    ]
    if verbose:
        rsync_cmd.append("-v")
    for e in excludes:
        rsync_cmd.extend(["--exclude", e])
    rsync_cmd.extend(["/host_project/", "/workspace/"])

    run([
        "docker", "run", "--rm",
        "-v", f"{VOLUME_CODE}:/workspace",
        "-v", f"{host_dir}:/host_project:ro",
        IMAGE_NAME,
        "bash", "-lc", " ".join(shlex.quote(x) for x in rsync_cmd)
    ])


def exec_in_container(cmd, check=True):
    return run(["docker", "exec", "-u", "runner", CONTAINER_NAME, "bash", "-lc", cmd], check=check)


def linux_build():
    path = WORKDIR if not CODE_SUBDIR else f"{WORKDIR}/{CODE_SUBDIR}"
    # Note: If this fails with SIGKILL, try increasing the Docker Desktop VM memory.
    exec_in_container(f"cd {path} && cargo build")


def linux_bench(bench_filter):
    path = WORKDIR if not CODE_SUBDIR else f"{WORKDIR}/{CODE_SUBDIR}"
    exec_in_container(f"cd {path} && cargo bench -- {shlex.quote(bench_filter)}")


def main():
    parser = argparse.ArgumentParser(description="Run Rust benchmarks inside a Linux container with cached builds.")
    parser.add_argument("benchmark", help="Benchmark name filter passed to cargo bench")
    parser.add_argument("--no-build", action="store_true", help="Skip build step and only run benchmarks")
    parser.add_argument("--verbose", action="store_true", help="Verbose sync output")
    args = parser.parse_args()

    require_docker()
    ensure_volumes()
    build_image()
    ensure_container()
    rsync_code(verbose=args.verbose)

    try:
        if not args.no_build:
            log("Building in Linux…")
            linux_build()
            ok("Build complete")
        log("Running benchmarks…")
        linux_bench(args.benchmark)
        ok("Benchmarks finished")
    except RuntimeError as e:
        err(str(e))
        err("If build fails due to missing dependencies, ensure Docker Desktop is running and the image is up to date.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())


