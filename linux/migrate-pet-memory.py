#!/usr/bin/env python3
"""Prepare a private v4 journal, preserving installed and additional dialogues.

Both applications must be stopped before applying the result. This tool never
replaces an input, starts a task, or changes which native session owns a job.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import stat

MAX_BYTES = 128 * 1024 * 1024
TERMINAL = {"done", "failed", "stopped", "interrupted", "resumed"}
SERVICE_PETS = {"integration_resend", "integration_n8n", "integration_vercel", "integration_github",
                "integration_notion", "integration_calcom", "integration_stripe", "integration_spotify"}
PROVIDERS = {
    "codex", "copilot", "antigravity", "gemini-cli", "claude-cli", "ollama",
    "lmstudio", "custom", "anthropic", "openai", "google", "openrouter",
} | SERVICE_PETS


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def read(path):
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb") as source:
        require(stat.S_ISREG(os.fstat(source.fileno()).st_mode), "Not a regular journal")
        data = source.read(MAX_BYTES + 1)
    require(len(data) <= MAX_BYTES, "Journal exceeds the native limit")
    def unique(pairs):
        result = {}
        for key, item in pairs:
            require(key not in result, "Duplicate JSON key; review the original journal")
            result[key] = item
        return result
    value = json.loads(data, object_pairs_hook=unique)
    require(isinstance(value, dict) and type(value.get("version")) is int
            and value["version"] in (1, 2, 3, 4), "Unknown journal format")
    require(set(value) <= {"version", "pets", "archives"}, "Unknown journal fields")
    require(isinstance(value.get("pets"), dict) and isinstance(value.get("archives", {}), dict), "Invalid library")
    return value


def natural(value):
    return type(value) is int and 0 <= value <= 2**64 - 1


def execution_receipts(task):
    rows = task.get("localExecution", [])
    require(isinstance(rows, list) and len(rows) <= 8, "Invalid local execution receipts")
    require(not rows or task["backend"] == "ollama", "Execution receipts belong to another backend")
    states = {"proposed", "starting", "exited", "denied", "expired", "failedToStart",
              "timedOut", "outputLimit", "cancelled", "skipped", "unknown"}
    causes = {"userCancel", "approvalDeadline", "taskDeadline", "commandDeadline", "outputBudget",
              "previousDenied", "policyRevoked", "spawnError", "crashDuringStart", "cleanupUnconfirmed", "processExit"}
    seen = set()
    for row in rows:
        require(isinstance(row, dict) and set(row) <= {
            "stepId", "ordinal", "state", "cause", "trigger", "startedAt", "finishedAt",
            "exitCode", "signal", "stdoutBytes", "stderrBytes", "truncated"}, "Unknown local receipt fields")
        ordinal = row.get("ordinal")
        require(type(ordinal) is int and 1 <= ordinal <= 8 and ordinal not in seen
                and row.get("stepId") == f"step-{ordinal}" and isinstance(row.get("state"), str) and row["state"] in states,
                "Invalid local receipt identity or state")
        seen.add(ordinal)
        require(all(row.get(k) is None or (isinstance(row[k], str) and row[k] in causes)
                    for k in ("cause", "trigger")), "Invalid local receipt cause")
        require(all(row.get(k) is None or natural(row[k]) for k in ("startedAt", "finishedAt")), "Invalid receipt timestamp")
        require(all(row.get(k) is None or (type(row[k]) is int and -(2**31) <= row[k] < 2**31)
                    for k in ("exitCode", "signal")), "Invalid receipt process result")
        require(all(natural(row.get(k, 0)) and row.get(k, 0) <= 131072 for k in ("stdoutBytes", "stderrBytes"))
                and type(row.get("truncated", False)) is bool, "Invalid receipt output counters")


PETS = {"codex": "agent_codex", "copilot": "agent_copilot", "antigravity": "agent_antigravity",
        "gemini-cli": "agent_gemini", "claude-cli": "integration_claude", "ollama": "ai_ollama",
        **{provider: provider for provider in SERVICE_PETS}}


def request_history(provider, value):
    rows = value.pop("request_history", [])
    require(isinstance(rows, list), "Invalid public request history")
    ids = set()
    previous_anchor = 0
    occupied_until = 0
    for index, row in enumerate(rows):
        require(isinstance(row, dict) and set(row) <= {"taskId", "conversationId", "provider", "petId", "sequence",
                "query", "phase", "started", "updated", "source", "messageIndex", "messagesAdded"},
                "Unknown public request fields")
        identifier = row.get("taskId")
        require(isinstance(identifier, str) and re.fullmatch(r"[A-Za-z0-9-]{1,128}", identifier)
                and identifier not in ids, "Invalid or duplicate public request ID")
        ids.add(identifier)
        require(row.get("provider") == provider and row.get("conversationId") == value["id"]
                and provider in PETS and row.get("petId") == PETS[provider], "Public request belongs to another destination")
        require(type(row.get("sequence")) is int and row["sequence"] == index + 1
                and isinstance(row.get("query"), str), "Invalid public request order or query")
        require(row.get("phase") in {"done", "failed", "stopped", "interrupted"},
                "Active public request: defer migration until work finishes")
        require(natural(row.get("started")) and natural(row.get("updated")), "Invalid public request timestamp")
        at, count = row.get("messageIndex"), row.get("messagesAdded")
        if row.get("source") == "captured":
            require(natural(at) and type(count) is int and count in (0, 2)
                    and max(previous_anchor, occupied_until) <= at and at + count <= len(value["messages"]),
                    "Invalid public request message anchor or overlap")
            if count == 2:
                require(value["messages"][at]["role"] == "user" and value["messages"][at + 1]["role"] == "assistant",
                        "Public request must reference an actual user/assistant pair")
                occupied_until = at + count
            previous_anchor = at
        else:
            require(row.get("source") == "taskJournal" and at is None and count is None,
                    "Invalid public request provenance")
            row.pop("messageIndex", None)
            row.pop("messagesAdded", None)
    if rows:
        value["request_history"] = rows


def conversation(provider, source):
    require(provider in PROVIDERS, "Unknown provider; preserve the inputs for review")
    require(isinstance(source, dict), "Invalid conversation")
    require(set(source) <= {"id", "created", "updated", "messages", "session", "tasks", "task_notice_id", "request_history"}, "Unknown conversation fields")
    value = copy.deepcopy(source)
    reference = value.pop("task_notice_id", None)
    require(reference is None or (isinstance(reference, str)
            and re.fullmatch(r"[A-Za-z0-9-]{1,128}", reference)), "Invalid task notice reference")
    if reference is not None:
        value["task_notice_id"] = reference
    messages = value.get("messages")
    require(isinstance(messages, list), "Invalid message archive")
    for message in messages:
        require(isinstance(message, dict) and set(message) == {"role", "content"}
                and message["role"] in ("user", "assistant") and isinstance(message["content"], str),
                "Non-public or invalid message; review without discarding it")
    session = value.get("session")
    require(session is None or (isinstance(session, dict)
            and set(session) == {"id", "cwd", "full_access"}
            and isinstance(session["id"], str) and 0 < len(session["id"].encode()) <= 512
            and isinstance(session["cwd"], str) and len(session["cwd"].encode()) <= 4096
            and Path(session["cwd"]).is_absolute() and type(session["full_access"]) is bool),
            "Invalid native session reference")
    value.setdefault("session", None)
    for field in ("created", "updated"):
        value.setdefault(field, 0)
        require(natural(value[field]), "Invalid conversation timestamp")
    # Stable IDs make repeated preparation idempotent even for the v1 journal.
    if not value.get("id"):
        identity = json.dumps([provider, messages, session, value["created"], value["updated"]],
                              ensure_ascii=False, sort_keys=True).encode()
        value["id"] = "import-" + hashlib.sha256(identity).hexdigest()[:40]
    require(isinstance(value["id"], str) and re.fullmatch(r"[A-Za-z0-9-]{1,128}", value["id"]), "Invalid conversation ID")
    value.setdefault("tasks", [])
    require(isinstance(value["tasks"], list), "Invalid task journal")
    for task in value["tasks"]:
        require(isinstance(task, dict) and task.get("provider") == provider, "Task belongs to another provider")
        require(set(task) <= {"id", "provider", "backend", "model", "query", "cwd", "phase", "progress",
                              "result", "started", "updated", "finished", "resumes", "resumedBy", "sessionId", "conversationId",
                              "localExecution", "localReasoningEffort"},
                "Unknown task fields")
        require(task.get("phase") in TERMINAL, "Active task: defer migration until work finishes")
        backend = task.get("backend")
        valid_backend = (backend == "codex" and provider in ({"codex"} | SERVICE_PETS)
                         or backend == "copilot" and provider == "copilot"
                         or backend == "antigravity" and provider in {"antigravity", "gemini-cli", "claude-cli"}
                         or backend == "ollama" and provider == "ollama")
        require(valid_backend, "Invalid task backend")
        for field, limit in (("id", 128), ("provider", 32), ("backend", 32), ("model", 256),
                             ("cwd", 4096), ("query", 4000), ("progress", 1000), ("result", 8000)):
            require(isinstance(task.get(field), str) and len(task[field]) <= limit, "Task would be truncated by the native reader")
        require(bool(task["id"]), "Missing task ID")
        for field in ("resumes", "resumedBy", "sessionId"):
            require(task.get(field) is None or (isinstance(task[field], str) and len(task[field]) <= 512),
                    "Invalid optional task reference")
        require(task.get("conversationId") is None or (isinstance(task["conversationId"], str)
                and re.fullmatch(r"[A-Za-z0-9-]{1,128}", task["conversationId"])), "Invalid task conversation reference")
        if backend == "ollama":
            require(task.get("sessionId") is None and task.get("conversationId") is not None,
                    "Local task must reference its own conversation, not a native CLI session")
        effort = task.get("localReasoningEffort")
        require(effort is None or (backend == "ollama" and isinstance(effort, str)
                and 0 < len(effort.encode()) <= 64 and not any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in effort)),
                "Invalid captured local reasoning effort")
        execution_receipts(task)
        require(all(natural(task.get(field)) for field in ("started", "updated"))
                and (task.get("finished") is None or natural(task["finished"])), "Invalid task timestamp")
    request_history(provider, value)
    return value


def merge(installed, additional):
    result = {"version": 4, "pets": {}, "archives": {}}
    seen = {}
    tasks = {}
    task_owners = {}
    request_owners = {}
    for library in (installed, additional):
        require(library.get("version") in (1, 2, 3, 4), "Unknown journal format")
        for archived in (False, True):
            groups = library.get("archives", {}) if archived else library["pets"]
            for provider, source in groups.items():
                rows = source if archived else [source]
                require(isinstance(rows, list), "Invalid archive list")
                for row in rows:
                    value = conversation(provider, row)
                    journal = tasks.setdefault(provider, {})
                    for task in value.pop("tasks"):
                        require(task["id"] not in task_owners or task_owners[task["id"]] == provider,
                                "Task ID collides across providers")
                        task_owners[task["id"]] = provider
                        require(task["id"] not in journal or journal[task["id"]] == task, "Conflicting task ID")
                        journal[task["id"]] = task
                    key = (provider, value["id"])
                    for row in value.get("request_history", []):
                        record_key = (*key, row["sequence"])
                        require(row["taskId"] not in request_owners or request_owners[row["taskId"]] == record_key,
                                "Public request ID collides across destinations")
                        request_owners[row["taskId"]] = record_key
                    require(key not in seen or seen[key] == value, "Conflicting conversation ID")
                    if key in seen:
                        continue
                    seen[key] = value
                    current = result["pets"].get(provider)
                    if current is None:
                        result["pets"][provider] = value
                    elif not current["messages"] and not current["session"] and value["messages"] and not archived:
                        result["pets"][provider] = value
                        result["archives"].setdefault(provider, []).append(current)
                    else:
                        result["archives"].setdefault(provider, []).append(value)
    all_tasks = [task for journal in tasks.values() for task in journal.values()]
    for provider, current in result["pets"].items():
        require(len(tasks[provider]) <= 10, "More than ten tasks: review retention before migration")
        for task in tasks[provider].values():
            require(task.get("conversationId") is None or (provider, task["conversationId"]) in seen,
                    "Task origin conversation is not present in this provider's library")
        current["tasks"] = sorted(tasks[provider].values(), key=lambda task: (task["started"], task["id"]))
        for row in [current, *result["archives"].get(provider, [])]:
            for event in row.get("request_history", []):
                require(all(task["id"] != event["taskId"] or
                            (task["provider"] == provider and task.get("conversationId") == row["id"])
                            for task in all_tasks), "Public request and retained task disagree on destination")
            reference = row.get("task_notice_id")
            if reference is not None:
                linked = tasks[provider].get(reference)
                require(linked is not None and linked.get("conversationId") == row["id"],
                        "Task notice does not belong to this conversation")
    require(all(len(rows) <= 100 for rows in result["archives"].values()), "Archive limit would be exceeded")
    require(len(encode(result)) <= MAX_BYTES, "Merged library exceeds the native limit")
    return result


def encode(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def prepare(installed, additional, output):
    require(output.resolve() not in {installed.resolve(), additional.resolve()}, "Output cannot replace an input")
    value = merge(read(installed), read(additional))
    # Exclusive creation leaves prior preparations/backups intact, including symlinks.
    parent = os.open(output.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        require(os.fstat(parent).st_uid == os.getuid(), "Output directory belongs to another account")
        descriptor = os.open(output.name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                             0o600, dir_fd=parent)
        identity = os.fstat(descriptor)
        try:
            with os.fdopen(descriptor, "wb") as target:
                target.write(encode(value))
                target.flush()
                os.fsync(target.fileno())
            os.fsync(parent)
        except BaseException:
            # Never resolve the parent again: it may have been renamed or
            # replaced by a symlink while the file was being written.
            try:
                current = os.stat(output.name, dir_fd=parent, follow_symlinks=False)
                if (current.st_dev, current.st_ino) == (identity.st_dev, identity.st_ino):
                    os.unlink(output.name, dir_fd=parent)
            except OSError:
                pass
            raise
    finally:
        os.close(parent)
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--installed", required=True, type=Path)
    parser.add_argument("--additional", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        value = prepare(args.installed, args.additional, args.output)
    except (ValueError, OSError) as error:
        parser.exit(1, f"Migration not prepared: {error}\n")
    print(json.dumps({"version": 4, "providers": len(value["pets"]),
                      "conversations": len(value["pets"]) + sum(map(len, value["archives"].values())),
                      "messages": sum(len(c["messages"]) for c in value["pets"].values())
                                  + sum(len(c["messages"]) for rows in value["archives"].values() for c in rows),
                      "tasks": sum(len(c["tasks"]) for c in value["pets"].values())}))


if __name__ == "__main__":
    main()
