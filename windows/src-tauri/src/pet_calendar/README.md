# Calendar context backend

This Linux backend normalizes only an ICS explicitly selected for the admitted
pet/conversation/turn. It does not discover calendars or connect accounts. The
ordinary chat/voice paths supply up to32events for the next14rollingdays in the
host IANA timezone. The JSON states the exact interval, source revision and
complete/partial/unavailable coverage. The source is an export, not a live feed.
Persistent sources and an arbitrary-window query tool are not implemented here.

`mod.rs` owns native document permission/selection checks, bounded regular-file
reads, source identity and revocation guards. `process.rs` supervises a one-shot
Python worker with a sealed embedded ZIP, bounded pipes, concurrency/deadline
and owned-child cleanup. `worker.py` has no native path/grant/account authority.
Recurrence expansion uses the pinned public libraries. Source text and parsed
events never grant action/command/email permissions. Native CLI approvals remain
in their original providers.

## Rebuild the embedded dependencies

Use a project-private Python environment with exactly the seven distributions
from `requirements.lock`; the builder rejects missing/extra versions and native
extensions. A Python3.10+ interpreter is required by the pinned packages. On the
current workstation the validated interpreter is Python3.14. No global package
installation or host venv is required by the installed application.

From the repository root, using the existing `uv` tool:

```sh
uv venv .build/calendar-runtime
UV_CACHE_DIR=.build/calendar-uv-cache uv pip sync --require-hashes \
  --python .build/calendar-runtime/bin/python \
  windows/src-tauri/src/pet_calendar/requirements.lock
.build/calendar-runtime/bin/python -B windows/scripts/build-calendar-runtime.py
```

`runtime.zip` contains the pure Python modules, timezone data, distribution
metadata and their license files. `runtime-manifest.json` records each included
file and the archive/lock SHA256. The builder uses sorted paths and fixed ZIP
timestamps; rebuilding the validated private environment produced the same
archive hash. Cargo includes the archive and worker directly, without downloads
at application startup. Do not replace them independently of the frozen source
map and release checks.

## Boundaries

The trusted worker denies socket/process audit events and has resource limits;
this is not a general filesystem sandbox. Its captured source arrives on stdin,
never in process arguments. Scope revocation cancels only its original turn.
Cancelling remote model HTTP closes this application's wait/transport; it does
not promise to undo remote work or previously authorized local effects.
