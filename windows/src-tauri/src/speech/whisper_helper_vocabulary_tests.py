"""Standard-library fixtures: actual helper defs, captured fake model; no ASR load."""
import ast
import json
from pathlib import Path
import types
import unittest
import unicodedata

SOURCE = Path(__file__).with_name("whisper_helper.py").read_text()
NODES = [n for n in ast.parse(SOURCE).body if
         isinstance(n, ast.FunctionDef) and n.name in {"unique_object", "parse_request", "recognize"}
         or isinstance(n, ast.Assign) and all(isinstance(t, ast.Name) and t.id.startswith("MAX_") for t in n.targets)]
NS = {"json": json, "unicodedata": unicodedata}
exec(compile(ast.Module(body=NODES, type_ignores=[]), "actual-helper-defs", "exec"), NS)


def line(value):
    return (json.dumps(value, ensure_ascii=False) + "\n").encode()


class VocabularyTests(unittest.TestCase):
    def test_default_empty_and_per_request_utf8_dedup(self):
        parse = NS["parse_request"]
        self.assertEqual(parse(line({"samples": 0})), (0, []))
        self.assertEqual(parse(line({"samples": 4, "vocabulary": [" Copilot ", "copilot", "Código", "Código"]})), (4, ["Copilot", "Código"]))
        self.assertEqual(parse(line({"samples": 4})), (4, []))

    def test_invalid_types_bounds_keys_controls_unicode_and_framing(self):
        invalid = [{"samples": x} for x in [True, False, -1, 960001, 1.0, None, "4"]]
        invalid += [{"samples": 1, "vocabulary": x} for x in [None, "Copilot", [1], [""], [" "], ["x\ny"], ["x\x00"], ["x\x7f"], ["x" * 161], ["á" * 81], ["x"] * 33]]
        invalid += [{"samples": 1, "task": "translate"}, {"samples": 1, "vocabulary": [f"{n:02}" + "x" * 158 for n in range(32)]}]
        raw = [line(x) for x in invalid] + [b'{"samples":1,"samples":2}\n', b'{"samples":1}', b'{"samples":1,"vocabulary":["\xff"]}\n', b'{"samples":1,"vocabulary":["\\ud800"]}\n', b"x" * 12289]
        for request in raw:
            with self.subTest(length=len(request)):
                with self.assertRaises((ValueError, UnicodeError)):
                    NS["parse_request"](request)

    def test_hotwords_are_optional_and_language_task_filters_stay_unchanged(self):
        class FakeModel:
            calls = []
            def transcribe(self, audio, **options):
                self.calls.append(options)
                return [types.SimpleNamespace(text="Olá, podes ajudar-me?", no_speech_prob=.1, avg_logprob=-.1)], None
        fake = FakeModel()
        NS["model"] = fake
        self.assertEqual(NS["recognize"]("fake pcm", ["Copilot", "unit tests"]), "Olá, podes ajudar-me?")
        self.assertEqual(NS["recognize"]("fake pcm"), "Olá, podes ajudar-me?")
        self.assertEqual(fake.calls[0]["hotwords"], "Copilot, unit tests")
        self.assertNotIn("hotwords", fake.calls[1])
        for options in fake.calls:
            self.assertIsNone(options["language"])
            self.assertIs(options["multilingual"], True)
            self.assertEqual(options["task"], "transcribe")
            self.assertIs(options["condition_on_previous_text"], False)
            self.assertNotIn("initial_prompt", options)
            self.assertEqual(options["beam_size"], 5)

    def test_bad_request_cannot_reach_fake_recognition_or_action(self):
        calls = []
        for raw in [b'{"samples":true}\n', b'{"samples":4,"vocabulary":"command"}\n']:
            try:
                count, vocabulary = NS["parse_request"](raw)
                calls.append((count, vocabulary))
            except ValueError:
                pass
        self.assertEqual(calls, [])
        self.assertIn('local_files_only=True', SOURCE)
        imports = [a.name for n in ast.parse(SOURCE).body if isinstance(n, ast.Import) for a in n.names]
        self.assertNotIn('requests', imports)


if __name__ == "__main__":
    unittest.main()
