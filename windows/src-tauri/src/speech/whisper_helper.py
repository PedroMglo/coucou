"""Local, bounded PCM requests. No recordings, cloud calls or transcript logs."""
import json
import os
import select
import sys
import time
import unicodedata

import numpy as np
from faster_whisper import WhisperModel


def exact(size):
    data = bytearray()
    while len(data) < size:
        chunk = sys.stdin.buffer.read(size - len(data))
        if not chunk:
            raise EOFError
        data.extend(chunk)
    return data


def load(device):
    return WhisperModel(sys.argv[1], device=device,
                        compute_type="int8_float16" if device == "cuda" else "int8",
                        cpu_threads=6, num_workers=1, local_files_only=True)


MAX_HEADER_BYTES = 12288
MAX_VOCABULARY_TERMS = 32
MAX_VOCABULARY_TERM_BYTES = 160
MAX_VOCABULARY_BYTES = 4096


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError("Invalid request")
        value[key] = item
    return value


def parse_request(line):
    if not line or len(line) > MAX_HEADER_BYTES or not line.endswith(b"\n"):
        raise ValueError("Invalid request")
    request = json.loads(line.decode("utf-8", errors="strict"), object_pairs_hook=unique_object)
    if type(request) is not dict or not set(request).issubset({"samples", "vocabulary"}):
        raise ValueError("Invalid request")
    count = request.get("samples")
    if type(count) is not int or not 0 <= count <= 16000 * 60:
        raise ValueError("Invalid audio length")
    raw = request.get("vocabulary", [])
    if type(raw) is not list or len(raw) > MAX_VOCABULARY_TERMS:
        raise ValueError("Invalid vocabulary")
    vocabulary, seen, total = [], set(), 0
    for item in raw:
        if type(item) is not str:
            raise ValueError("Invalid vocabulary")
        term = item.strip()
        size = len(term.encode("utf-8", errors="strict"))
        if not term or size > MAX_VOCABULARY_TERM_BYTES or any(unicodedata.category(c) == "Cc" for c in item):
            raise ValueError("Invalid vocabulary")
        key = "".join(c.lower() if c.isascii() else c for c in term)
        if key in seen:
            continue
        seen.add(key)
        total += size
        if total > MAX_VOCABULARY_BYTES:
            raise ValueError("Invalid vocabulary")
        vocabulary.append(term)
    return count, vocabulary


def recognize(audio, vocabulary=()):
    # Detect the spoken language per segment; preserve Portuguese/English code switching.
    options = {"hotwords": ", ".join(vocabulary)} if vocabulary else {}
    segments, _ = model.transcribe(
        audio, language=None, multilingual=True, task="transcribe", beam_size=5,
        temperature=0, condition_on_previous_text=False,
        vad_filter=True, vad_parameters={"min_silence_duration_ms": 500}, **options,
    )
    return " ".join(s.text.strip() for s in segments
                    if s.no_speech_prob < 0.6 and s.avg_logprob > -1.0).strip()


device = "cuda" if os.environ.get("COUCOU_WHISPER_CUDA") == "1" else "cpu"
try:
    model = load(device)
except RuntimeError:
    device = "cpu"
    model = load(device)
while select.select([sys.stdin.buffer], [], [], 120)[0]:
    line = sys.stdin.buffer.readline(MAX_HEADER_BYTES + 1)
    if not line:
        break
    count, vocabulary = parse_request(line)
    audio = np.frombuffer(exact(count * 4), dtype="<f4").copy()
    began = time.monotonic()
    text = ""
    if count >= 4000 and np.isfinite(audio).all() and np.max(np.abs(audio)) > 0.004:
        try:
            text = recognize(audio, vocabulary)
        except RuntimeError:
            if device != "cuda":
                raise
            del model
            import gc
            gc.collect()
            device = "cpu"
            model = load(device)
            text = recognize(audio, vocabulary)
    print(json.dumps({"text": text, "seconds": time.monotonic() - began, "device": device}, ensure_ascii=False), flush=True)
