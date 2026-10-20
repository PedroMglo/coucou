import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("migration", Path(__file__).with_name("migrate-pet-memory.py"))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


def dialogue(text, identifier=None, tasks=None):
    value = {"messages": [{"role": "user", "content": text}], "session": None, "tasks": tasks or []}
    if identifier:
        value["id"] = identifier
    return value


def task(identifier="task", phase="done"):
    return dict(id=identifier, provider="codex", backend="codex", model="local-fixture", cwd="/tmp",
                query="synthetic", phase=phase, progress="", result="", started=1, updated=2, finished=2)


class MigrationTests(unittest.TestCase):
    def test_local_receipts_are_preserved_without_replay_or_native_session_adoption(self):
        local = task()
        local.update(provider="ollama", backend="ollama", conversationId="local-chat", sessionId=None, localReasoningEffort="none",
                     localExecution=[dict(stepId="step-1", ordinal=1, state="unknown", cause="crashDuringStart",
                                          startedAt=1, stdoutBytes=42, stderrBytes=0, truncated=False)])
        real = {"version": 3, "pets": {"ollama": dialogue("original", "local-chat", [local])}}
        before = copy.deepcopy(real)
        result = m.merge(real, {"version": 3, "pets": {}})
        self.assertEqual(result["pets"]["ollama"]["tasks"], [local])
        self.assertEqual(real, before)
        self.assertEqual(m.merge(result, {"version": 3, "pets": {}}), result)

    def test_local_receipts_refuse_cross_owner_private_output_duplicates_and_bad_limits(self):
        local = task()
        local.update(provider="ollama", backend="ollama", conversationId="local-chat", sessionId=None,
                     localExecution=[dict(stepId="step-1", ordinal=1, state="exited")])
        real = {"version": 3, "pets": {"ollama": dialogue("local", "local-chat", [local])}}
        changes = [dict(sessionId="native-session"), dict(conversationId=None),
                   dict(localReasoningEffort="x" * 65), dict(localReasoningEffort="high\n"),
                   dict(localReasoningEffort=[]), dict(localReasoningEffort=""),
                   dict(localExecution=None), dict(localExecution=local["localExecution"] * 2),
                   dict(localExecution=[dict(stepId="step-1", ordinal=1, state="exited", stdout="private")]),
                   dict(localExecution=[dict(stepId="step-1", ordinal=1, state="exited", stdoutBytes=131073)]),
                   dict(localExecution=[dict(stepId="step-1", ordinal=True, state="exited")])]
        for change in changes:
            with self.subTest(fields=list(change)):
                invalid = copy.deepcopy(real)
                invalid["pets"]["ollama"]["tasks"][0].update(change)
                with self.assertRaises(ValueError):
                    m.merge(invalid, {"version": 3, "pets": {}})
        native = {"version": 3, "pets": {"codex": dialogue("native", "native-chat", [task()])}}
        native["pets"]["codex"]["tasks"][0]["localExecution"] = local["localExecution"]
        with self.assertRaisesRegex(ValueError, "another backend"):
            m.merge(native, {"version": 3, "pets": {}})

    def test_keeps_installed_current_and_qa_archive_with_all_tasks_and_ollama(self):
        real = {"version": 1, "pets": {"codex": dialogue("real", tasks=[task()])}}
        qa = {"version": 3, "pets": {"codex": dialogue("qa", "qa", [task("two")]), "ollama": dialogue("local", "local")}}
        before = copy.deepcopy((real, qa))
        result = m.merge(real, qa)
        self.assertEqual((real, qa), before)
        self.assertEqual(result["pets"]["codex"]["messages"][0]["content"], "real")
        self.assertEqual(result["archives"]["codex"][0]["messages"][0]["content"], "qa")
        self.assertEqual(len(result["pets"]["codex"]["tasks"]), 2)
        self.assertEqual(result["pets"]["ollama"]["id"], "local")
        self.assertEqual(m.merge(result, qa), result)

    def test_conflicting_ids_and_active_jobs_refuse_without_loss(self):
        real = {"version": 3, "pets": {"codex": dialogue("one", "same")}}
        qa = {"version": 3, "pets": {"codex": dialogue("two", "same")}}
        with self.assertRaisesRegex(ValueError, "Conflicting conversation"):
            m.merge(real, qa)
        qa["pets"]["codex"] = dialogue("two", "other", [task(phase="working")])
        with self.assertRaisesRegex(ValueError, "Active task"):
            m.merge(real, qa)

    def test_native_retention_cannot_silently_delete_tasks(self):
        real = {"version": 1, "pets": {"codex": dialogue("real", tasks=[task(str(i)) for i in range(11)])}}
        with self.assertRaisesRegex(ValueError, "More than ten tasks"):
            m.merge(real, {"version": 3, "pets": {}})

    def test_task_ids_are_global_and_optional_references_match_native_schema(self):
        real = {"version": 3, "pets": {"codex": dialogue("one", tasks=[task()])}}
        copilot = task()
        copilot.update(provider="copilot", backend="copilot")
        qa = {"version": 3, "pets": {"copilot": dialogue("two", tasks=[copilot])}}
        with self.assertRaisesRegex(ValueError, "collides across providers"):
            m.merge(real, qa)
        for field in ("resumes", "resumedBy", "sessionId"):
            for bad in (42, False, [], {}, "x" * 513):
                with self.subTest(field=field, bad_type=type(bad).__name__):
                    invalid = copy.deepcopy(real)
                    invalid["pets"]["codex"]["tasks"][0][field] = bad
                    with self.assertRaisesRegex(ValueError, "optional task reference"):
                        m.merge(invalid, {"version": 3, "pets": {}})

    def test_logical_task_origin_stays_with_its_provider_and_archived_conversation(self):
        original = task()
        original["conversationId"] = "original-codex"
        real = {"version": 3, "pets": {"codex": dialogue("new", "current-codex", [original])},
                "archives": {"codex": [dialogue("original", "original-codex")]}}
        result = m.merge(real, {"version": 3, "pets": {}})
        self.assertEqual(result["pets"]["codex"]["tasks"][0]["conversationId"], "original-codex")
        for bad in (42, "../original", "x" * 129, "missing"):
            invalid = copy.deepcopy(real)
            invalid["pets"]["codex"]["tasks"][0]["conversationId"] = bad
            with self.subTest(bad_type=type(bad).__name__):
                with self.assertRaises(ValueError):
                    m.merge(invalid, {"version": 3, "pets": {}})
        invalid = copy.deepcopy(real)
        invalid["archives"] = {"ollama": invalid["archives"]["codex"]}
        with self.assertRaisesRegex(ValueError, "not present in this provider"):
            m.merge(invalid, {"version": 3, "pets": {}})

    def test_source_immutable_output_private_exclusive_and_symlinks_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            real, qa, output = (Path(directory) / name for name in ("real", "qa", "out"))
            data = json.dumps({"version": 1, "pets": {"codex": dialogue("original")}}).encode()
            real.write_bytes(data)
            qa.write_text('{"version":3,"pets":{}}')
            m.prepare(real, qa, output)
            self.assertEqual(real.read_bytes(), data)
            self.assertEqual(output.stat().st_mode & 0o777, 0o600)
            with self.assertRaises(FileExistsError):
                m.prepare(real, qa, output)
            with self.assertRaisesRegex(ValueError, "replace an input"):
                m.prepare(real, qa, real)
            link = Path(directory) / "link"
            link.symlink_to(real)
            with self.assertRaises(OSError):
                m.read(link)

    def test_nonregular_sources_and_duplicate_json_keys_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            fifo = Path(directory) / "pipe"
            os.mkfifo(fifo)
            with self.assertRaisesRegex(ValueError, "Not a regular"):
                m.read(fifo)
            duplicate = Path(directory) / "duplicate"
            duplicate.write_text('{"version":3,"pets":{},"pets":{}}')
            with self.assertRaisesRegex(ValueError, "Duplicate JSON"):
                m.read(duplicate)

    def test_failed_write_after_parent_swap_cannot_unlink_an_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, destination, moved = (root / name for name in ("source", "destination", "moved"))
            source.mkdir()
            destination.mkdir()
            real, qa = source / "journal.json", root / "qa.json"
            data = b'{"version":1,"pets":{}}'
            real.write_bytes(data)
            qa.write_bytes(data)

            def swap_then_fail(_descriptor):
                destination.rename(moved)
                destination.symlink_to(source, target_is_directory=True)
                raise OSError("synthetic write failure")

            with patch.object(m.os, "fsync", side_effect=swap_then_fail):
                with self.assertRaises(OSError):
                    m.prepare(real, qa, destination / "journal.json")
            self.assertEqual(real.read_bytes(), data)
            self.assertFalse((moved / "journal.json").exists())



class TaskNoticeMigrationTests(unittest.TestCase):
    def library(self):
        original = task("stopped-task", "stopped")
        original["conversationId"] = "stopped-chat"
        current = dialogue("new current", "current-chat", [original])
        archived = dialogue("old original request", "stopped-chat")
        archived["task_notice_id"] = "stopped-task"
        return {"version": 3, "pets": {"codex": current}, "archives": {"codex": [archived]}}

    def test_archived_notice_reference_survives_merge_without_switching_or_replaying(self):
        source = self.library()
        before = copy.deepcopy(source)
        result = m.merge(source, {"version": 3, "pets": {}})
        self.assertEqual(source, before)
        self.assertEqual(result["pets"]["codex"]["id"], "current-chat")
        self.assertEqual(result["archives"]["codex"][0]["task_notice_id"], "stopped-task")
        self.assertEqual(result["pets"]["codex"]["tasks"][0]["phase"], "stopped")
        self.assertEqual(m.merge(result, {"version": 3, "pets": {}}), result)

    def test_invalid_dangling_and_cross_conversation_references_are_refused(self):
        for reference in (False, 42, [], {}, "", "../task", "x" * 129, "missing"):
            source = self.library()
            source["archives"]["codex"][0]["task_notice_id"] = reference
            with self.subTest(reference_type=type(reference).__name__):
                with self.assertRaises(ValueError):
                    m.merge(source, {"version": 3, "pets": {}})
        source = self.library()
        source["pets"]["codex"]["task_notice_id"] = "stopped-task"
        with self.assertRaisesRegex(ValueError, "does not belong"):
            m.merge(source, {"version": 3, "pets": {}})

    def test_absent_and_null_reference_preserve_legacy_default(self):
        source = {"version": 3, "pets": {"codex": dialogue("legacy", "legacy-chat")}}
        absent = m.merge(source, {"version": 3, "pets": {}})
        source["pets"]["codex"]["task_notice_id"] = None
        self.assertEqual(m.merge(source, {"version": 3, "pets": {}}), absent)

class RequestHistoryMigrationTests(unittest.TestCase):
    def library(self, count=1):
        row = dialogue("actual public message", "durable-chat")
        row["request_history"] = [dict(taskId=f"pruned-{i}", conversationId="durable-chat", provider="codex",
            petId="agent_codex", sequence=i + 1, query=f"full original {i} " + "á" * 6500,
            phase="stopped", started=1, updated=2, source="captured", messageIndex=0, messagesAdded=0)
            for i in range(count)]
        return {"version": 4, "pets": {"codex": row}}

    def test_full_history_survives_without_tasks_or_pointer_and_is_idempotent(self):
        source = self.library(24)
        before = copy.deepcopy(source)
        result = m.merge(source, {"version": 3, "pets": {}})
        self.assertEqual(source, before)
        self.assertEqual(result["version"], 4)
        self.assertEqual(result["pets"]["codex"]["request_history"], source["pets"]["codex"]["request_history"])
        self.assertEqual(result["pets"]["codex"]["tasks"], [])
        self.assertEqual(m.merge(result, {"version": 4, "pets": {}}), result)

    def test_invalid_private_fields_destinations_states_and_anchors_refuse(self):
        changes = [dict(provider="copilot"), dict(petId="integration_github"), dict(conversationId="foreign-chat"),
            dict(taskId="../task"), dict(sequence=True), dict(sequence=2), dict(phase="pending"), dict(phase="resumed"),
            dict(source="native-output"), dict(query=[]), dict(started=True), dict(updated=-1),
            dict(messageIndex=True), dict(messageIndex=999), dict(messagesAdded=True), dict(messagesAdded=1),
            dict(stdout="private execution output"), dict(role="assistant"), dict(source="taskJournal")]
        for change in changes:
            with self.subTest(fields=change):
                source = self.library()
                source["pets"]["codex"]["request_history"][0].update(change)
                with self.assertRaises(ValueError):
                    m.merge(source, {"version": 4, "pets": {}})

    def test_task_journal_provenance_keeps_unknown_message_position(self):
        source = self.library()
        record = source["pets"]["codex"]["request_history"][0]
        record.update(source="taskJournal", messageIndex=None, messagesAdded=None)
        result = m.merge(source, {"version": 4, "pets": {}})
        record = result["pets"]["codex"]["request_history"][0]
        self.assertNotIn("messageIndex", record)
        self.assertNotIn("messagesAdded", record)
        self.assertEqual(record["source"], "taskJournal")

    def test_duplicate_identity_and_retained_foreign_task_refuse(self):
        source = self.library(2)
        source["pets"]["codex"]["request_history"][1]["taskId"] = "pruned-0"
        with self.assertRaisesRegex(ValueError, "duplicate public request"):
            m.merge(source, {"version": 4, "pets": {}})
        source = self.library()
        other = dialogue("other", "other-chat")
        other["request_history"] = copy.deepcopy(source["pets"]["codex"]["request_history"])
        other["request_history"][0]["conversationId"] = "other-chat"
        source["archives"] = {"codex": [other]}
        with self.assertRaisesRegex(ValueError, "collides across destinations"):
            m.merge(source, {"version": 4, "pets": {}})
        source = self.library()
        linked = task("pruned-0", "stopped")
        linked["conversationId"] = "another-chat"
        source["pets"]["codex"]["tasks"] = [linked]
        source["archives"] = {"codex": [dialogue("another", "another-chat")]}
        with self.assertRaisesRegex(ValueError, "disagree on destination"):
            m.merge(source, {"version": 4, "pets": {}})

    def test_private_output_is_exclusive_and_inputs_are_unchanged(self):
        with tempfile.TemporaryDirectory() as directory:
            installed, additional, output = (Path(directory) / name for name in ("installed", "additional", "output"))
            installed.write_bytes(m.encode(self.library()))
            additional.write_bytes(m.encode({"version": 3, "pets": {}}))
            originals = installed.read_bytes(), additional.read_bytes()
            m.prepare(installed, additional, output)
            self.assertEqual((installed.read_bytes(), additional.read_bytes()), originals)
            self.assertEqual(output.stat().st_mode & 0o777, 0o600)
            self.assertEqual(m.read(output)["version"], 4)

    def test_all_canonical_service_pets_keep_logical_owner_instead_of_codex_engine(self):
        for provider in m.SERVICE_PETS:
            with self.subTest(provider=provider):
                source = self.library()
                row = source["pets"].pop("codex")
                event = row["request_history"][0]
                event.update(provider=provider, petId=provider)
                source["pets"][provider] = row
                linked = task("pruned-0", "stopped")
                linked.update(provider=provider, conversationId="durable-chat")
                row["tasks"] = [linked]
                result = m.merge(source, {"version": 4, "pets": {}})
                self.assertEqual(result["pets"][provider]["request_history"][0]["petId"], provider)

    def test_reordered_message_anchors_are_not_silently_sorted(self):
        source = self.library(2)
        source["pets"]["codex"]["request_history"][0]["messageIndex"] = 1
        with self.assertRaisesRegex(ValueError, "message anchor"):
            m.merge(source, {"version": 4, "pets": {}})

    def test_private_prefix_cannot_be_filtered_to_another_still_in_range_pair(self):
        for prefix in (1, 2):
            for count in (0, 2):
                with self.subTest(prefix=prefix, count=count), tempfile.TemporaryDirectory() as directory:
                    source = self.library()
                    row = source["pets"]["codex"]
                    public = [dict(role=role, content=label) for label in ("A", "B", "C") for role in ("user", "assistant")]
                    row["messages"] = [dict(role="tool", content="private")] * prefix + public
                    row["request_history"][0].update(messageIndex=prefix, messagesAdded=count, phase="done" if count else "stopped")
                    self.assertLessEqual(prefix + count, len(public))
                    if prefix == 2 and count == 2:
                        self.assertEqual([m["role"] for m in public[prefix:prefix + count]], ["user", "assistant"])
                    installed, extra, output = (Path(directory) / name for name in ("installed", "extra", "output"))
                    installed.write_bytes(m.encode(source))
                    extra.write_bytes(m.encode({"version": 4, "pets": {}}))
                    original = installed.read_bytes(), extra.read_bytes()
                    with self.assertRaisesRegex(ValueError, "Non-public"):
                        m.prepare(installed, extra, output)
                    self.assertFalse(output.exists())
                    self.assertEqual((installed.read_bytes(), extra.read_bytes()), original)

    def test_two_messages_require_user_assistant_order_and_disjoint_spans(self):
        source = self.library(2)
        row = source["pets"]["codex"]
        row["messages"] = [dict(role=role, content="synthetic") for role in ("user", "assistant", "user", "assistant")]
        first, second = row["request_history"]
        first.update(phase="done", messageIndex=0, messagesAdded=2)
        for roles in (("assistant", "user"), ("user", "user"), ("assistant", "assistant")):
            invalid = copy.deepcopy(source)
            invalid["pets"]["codex"]["messages"][0]["role"] = roles[0]
            invalid["pets"]["codex"]["messages"][1]["role"] = roles[1]
            with self.assertRaisesRegex(ValueError, "user/assistant pair"):
                m.merge(invalid, {"version": 4, "pets": {}})
        for at, count in ((0, 2), (1, 0)):
            second.update(messageIndex=at, messagesAdded=count)
            with self.assertRaisesRegex(ValueError, "overlap"):
                m.merge(source, {"version": 4, "pets": {}})
        second.update(phase="done", messageIndex=2, messagesAdded=2)
        self.assertEqual(len(m.merge(source, {"version": 4, "pets": {}})["pets"]["codex"]["request_history"]), 2)
        first.update(phase="stopped", messagesAdded=0)
        second.update(messageIndex=0)
        self.assertEqual(len(m.merge(source, {"version": 4, "pets": {}})["pets"]["codex"]["request_history"]), 2)

if __name__ == "__main__":
    unittest.main()
