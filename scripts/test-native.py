#!/usr/bin/env python3
"""Run the native GPU E2E suites in isolated directories, with a hard deadline.

Usage: python3 scripts/test-native.py [path/to/ditto]
On headless Linux: xvfb-run -a python3 scripts/test-native.py
Keeps screenshots, project/export fixtures and logs even on failure.
"""
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
binary = pathlib.Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/ditto"
parent = ROOT / "validation/runtime/native"
parent.mkdir(parents=True, exist_ok=True)
output = pathlib.Path(tempfile.mkdtemp(prefix="run-", dir=parent))
print(f"Native evidence: {output}", flush=True)
for name, flag in [("editor", "--e2e-dir"), ("shaders", "--shader-quality-smoke-dir")]:
    log = output / f"{name}.log"
    try:
        with log.open("w") as stream:
            subprocess.run([str(binary), flag, str(output / name)], cwd=ROOT,
                           stdout=stream, stderr=subprocess.STDOUT, check=True, timeout=120)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        print(log.read_text(), file=sys.stderr)
        sys.exit(f"Native {name} failed: {error}. Evidence: {output}")
    if name == "editor" and not (output / name / "e2e-result.txt").is_file():
        sys.exit(f"Native editor exited without its assertion report: {output}")
    if name == "editor" and not (output / name / "grid/result.txt").is_file():
        sys.exit(f"Native editor exited without its display-scale grid report: {output}")
    print(f"PASS {name}", flush=True)
print(f"PASS native E2E — artifacts: {output}")
