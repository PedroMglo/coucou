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


if __name__ == "__main__":
    unittest.main()
