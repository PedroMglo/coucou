#!/usr/bin/env python3
"""Create Coucou's private Portuguese Whisper environment and print its Python path."""
import argparse
import json
import hashlib
import os
from pathlib import Path
import shutil
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=root / ".build/whisper-env")
    parser.add_argument("--model-output", type=Path, default=root / ".build/whisper-large-v3-turbo")
    parser.add_argument("--gpu", action="store_true", help="Install private CUDA 12 libraries for an NVIDIA GPU")
    args = parser.parse_args()
    uv = shutil.which("uv")
    if not uv:
        parser.error("uv is required; this script does not install global tools")
    destination = args.output.expanduser().resolve()
    environment = {**os.environ, "UV_CACHE_DIR": str(root / ".build/uv-cache")}
    python = destination / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.is_file():
        subprocess.run([uv, "venv", "--python", "3.11", str(destination)], env=environment, check=True)
    subprocess.run([
        uv, "pip", "sync", "--python", str(python), "--require-hashes",
        str(root / ("linux/requirements-whisper-gpu.txt" if args.gpu else "linux/requirements-whisper.txt")),
    ], env=environment, check=True)
    model = json.loads((root / "linux/whisper-model.json").read_text())
    directory = args.model_output.expanduser().resolve()
    def valid():
        for name, expected in model["files"].items():
            path = directory / name
            if not path.is_file() or path.stat().st_size != expected["size"]:
                return False
            with path.open("rb") as source:
                if hashlib.file_digest(source, "sha256").hexdigest() != expected["sha256"]:
                    return False
        return True
    if not valid():
        subprocess.run(["hf", "download", model["repo"], *model["files"], "--revision", model["revision"], "--local-dir", str(directory)],
                       env={**environment, "HF_HOME": str(root / ".build/hf-cache"), "HF_HUB_DISABLE_IMPLICIT_TOKEN": "1"}, check=True)
        if not valid():
            raise RuntimeError("Model verification failed")
    print(python)
    print(directory)


if __name__ == "__main__":
    main()
