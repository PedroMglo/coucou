"""Bundle only the seven locked, pure Python distributions, without installation."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import re
import sysconfig
import zipfile

root = Path(__file__).resolve().parents[1]
out = root / "src-tauri/src/pet_calendar"
out.mkdir(parents=True, exist_ok=True)
site = Path(sysconfig.get_path("purelib")).resolve()
expected = {"click": "8.5.0", "icalendar": "7.3.0", "python-dateutil": "2.9.0.post0",
            "recurring-ical-events": "3.8.2", "six": "1.17.0", "tzdata": "2026.5", "x-wr-timezone": "2.0.1"}
files = {}
for dist in importlib.metadata.distributions(path=[str(site)]):
    name = re.sub(r"[-_.]+", "-", dist.metadata["Name"]).lower()
    if name not in expected:
        raise ValueError("unexpected runtime distribution")
    if dist.version != expected.pop(name):
        raise ValueError("runtime version differs from lock")
    for relative in dist.files:
        path = Path(dist.locate_file(relative)).resolve()
        if not path.is_relative_to(site):
            # Console entry points are unnecessary: the worker never launches them.
            continue
        if path.suffix in {".pyc", ".pyo"} or "__pycache__" in path.parts:
            continue
        key = path.relative_to(site).as_posix()
        if path.suffix in {".so", ".dll", ".dylib"}:
            raise ValueError("runtime must remain pure Python")
        files[key] = path.read_bytes()
if expected:
    raise ValueError("missing locked distribution")
with zipfile.ZipFile(out / "runtime.zip", "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for key, data in sorted(files.items()):
        info = zipfile.ZipInfo(key, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        archive.writestr(info, data, compresslevel=9)
manifest = {"schemaVersion": 1, "requirementsSha256": hashlib.sha256((out / "requirements.lock").read_bytes()).hexdigest(),
            "runtimeSha256": hashlib.sha256((out / "runtime.zip").read_bytes()).hexdigest(),
            "files": {key: hashlib.sha256(value).hexdigest() for key, value in sorted(files.items())}}
(out / "runtime-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(json.dumps({"runtimeSha256": manifest["runtimeSha256"], "files": len(files), "bytes": (out / "runtime.zip").stat().st_size}))
