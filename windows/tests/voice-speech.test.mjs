import { beforeEach, test } from "node:test";
import assert from "node:assert/strict";
import { calls, emit, sent, subscriptions } from "./tauri.mjs";
import { registerVoiceSpeech, VoiceSpeech } from "../src/pill/voice-speech.ts";

const reply = { id: 42, text: "Olá, Pedro.", lang: "pt-PT", audio: "data:audio/mpeg;base64,reply", gender: "female", allowSystemFallback: false };
beforeEach(() => { calls.length = 0; subscriptions.length = 0; });

test("the mapped pet surface can play and acknowledge speech without the hidden pill", async () => {
  let heard;
  await registerVoiceSpeech("companions", () => {}, { cancel() {}, async play(source) { heard = source; return true; } });
  assert.deepEqual(subscriptions, [
    { event: "voice-say", target: { kind: "Webview", label: "companions" } },
    { event: "voice-cancel", target: { kind: "Webview", label: "companions" } },
  ]);
  emit("voice-say", reply);
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(heard, reply.audio);
  assert.deepEqual(sent("voice_done"), [{ id: 42, text: null, played: true }]);
  assert.deepEqual(sent("voice_say_native"), []);
});

test("Raquel playback failure stays in text instead of using another voice", async () => {
  const speaker = new VoiceSpeech(() => {}, { cancel() {}, async play() { return false; } });
  await speaker.say(reply);
  assert.equal(sent("voice_done")[0].played, false);
  assert.deepEqual(sent("voice_say_native"), []);
});

test("cancelling a pending reply acknowledges cancellation without marking it played", async () => {
  let finish;
  const speaker = new VoiceSpeech(() => {}, { cancel() { finish?.(true); }, play() { return new Promise(resolve => { finish = resolve; }); } });
  const pending = speaker.say(reply);
  speaker.cancel();
  await pending;
  assert.equal(sent("voice_done")[0].played, false);
});

test("cancelling an old reply by id cannot interrupt its replacement", async () => {
  let complete;
  let cancellations = 0;
  const speaker = new VoiceSpeech(() => {}, {
    cancel() { cancellations++; },
    play() { return new Promise(resolve => { complete = resolve; }); },
  });
  const pending = speaker.say({ ...reply, id: 88 });
  const before = cancellations;
  speaker.cancel(42);
  assert.equal(cancellations, before);
  complete(true);
  await pending;
  assert.deepEqual(sent("voice_done"), [{ id: 88, text: null, played: true }]);
});

test("an exact reply id revokes only that mapped playback", async () => {
  let finish;
  const speaker = await registerVoiceSpeech("companions", () => {}, {
    cancel() { finish?.(true); },
    play() { return new Promise(resolve => { finish = resolve; }); },
  });
  emit("voice-say", reply);
  emit("voice-cancel", { id: reply.id });
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(sent("voice_done").at(-1).played, false);
  speaker.cancel();
});


test("a malformed scoped cancellation cannot become a global Stop", async () => {
  let complete;
  let cancellations = 0;
  await registerVoiceSpeech("companions", () => {}, {
    cancel() { cancellations++; },
    play() { return new Promise(resolve => { complete = resolve; }); },
  });
  emit("voice-say", reply);
  const before = cancellations;
  for (const payload of [{}, { id: "42" }, { id: -1 }, { id: 42.5 }]) emit("voice-cancel", payload);
  assert.equal(cancellations, before);
  complete(true);
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(sent("voice_done").at(-1).played, true);
});


test("system speech asks for only the backend reply id, never renderer text or language", async () => {
  const before = window.speechSynthesis;
  try {
    window.speechSynthesis = undefined;
    const speaker = new VoiceSpeech(() => {}, { cancel() {}, async play() { return false; } });
    await speaker.say({ ...reply, id: 99, text: "renderer text is not an invocation", lang: "pt-PT", audio: null, allowSystemFallback: true });
    assert.deepEqual(sent("voice_say_native"), [{ id: 99 }]);
    assert.deepEqual(sent("voice_done"), [{ id: 99, text: null, played: false }]);
  } finally {
    window.speechSynthesis = before;
  }
});
