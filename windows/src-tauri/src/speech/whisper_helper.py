"""Local, bounded PCM requests. No recordings, cloud calls or transcript logs."""
import json
import os
import select
import sys
import time

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


def recognize(audio):
    # Fix Portuguese, never translation or language auto-detection on names.
    segments, _ = model.transcribe(
        audio, language="pt", task="transcribe", beam_size=5,
        temperature=0, condition_on_previous_text=False,
        initial_prompt="Transcrição em português de Portugal.",
        hotwords="Codex, Gravity, Gemini, Claude, Ollama, PostgreSQL, Docker",
        vad_filter=True, vad_parameters={"min_silence_duration_ms": 500},
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
    line = sys.stdin.buffer.readline(4097)
    if not line:
        break
    request = json.loads(line)
    count = request["samples"]
    if not isinstance(count, int) or not 0 <= count <= 16000 * 60:
        raise ValueError("Invalid audio length")
    audio = np.frombuffer(exact(count * 4), dtype="<f4").copy()
    began = time.monotonic()
    text = ""
    if count >= 4000 and np.isfinite(audio).all() and np.max(np.abs(audio)) > 0.004:
        try:
            text = recognize(audio)
        except RuntimeError:
            if device != "cuda":
                raise
            del model
            import gc
            gc.collect()
            device = "cpu"
            model = load(device)
            text = recognize(audio)
    print(json.dumps({"text": text, "seconds": time.monotonic() - began, "device": device}, ensure_ascii=False), flush=True)
