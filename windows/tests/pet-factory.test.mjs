import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import * as factoryModule from "../src/core/pet-factory.ts";
import { installFakeDom } from "./fakedom.mjs";
import { petGallery } from "../src/desktop/pet-gallery.ts";

const source = JSON.parse(readFileSync(new URL("../src-tauri/pets.factory.json", import.meta.url), "utf8"));
const legacy = JSON.parse(readFileSync(new URL("../src-tauri/pets.default.json", import.meta.url), "utf8"));
const clone = (value) => JSON.parse(JSON.stringify(value));
const identity = (id) => ({ id, name: id, color: "#2dd4bf", state: "idle", attention: false, busy: false, dancing: false, active: false, activity: "", outfit: "none", sessions: 0 });

test("all legacy pet preferences retain their defaults", () => {
  const { PetFactory } = factoryModule;
  assert.equal(Object.keys(legacy.pets).length, 25);
  for (const [id, expected] of Object.entries(legacy.pets)) {
    assert.deepEqual(PetFactory.get(id)?.preferences, expected, id);
  }
});

test("mold edits flow only to members of that mold and instance values win", () => {
  const editable = clone(source);
  editable.base.presentation.galleryAvatarSize = 71;
  editable.molds.llm.preferences = { mode: "reasoning" };
  editable.pets.agent_cursor.preferences.name = "My Cursor";
  const baseFactory = factoryModule.createPetFactory(editable);

  assert.equal(baseFactory.get("agent_cursor").presentation.galleryAvatarSize, 71);
  assert.equal(baseFactory.get("integration_github").presentation.galleryAvatarSize, 71);
  assert.equal(baseFactory.get("agent_cursor").preferences.mode, "reasoning");
  assert.equal(baseFactory.get("agent_cursor").preferences.name, "My Cursor");
  assert.equal(baseFactory.get("ai_google").presentation.galleryAvatarSize, 71);

  editable.molds.application.presentation = { ...editable.molds.application.presentation, galleryAvatarSize: 83 };
  const moldFactory = factoryModule.createPetFactory(editable);
  assert.equal(moldFactory.get("agent_cursor").presentation.galleryAvatarSize, 71);
  assert.equal(moldFactory.get("integration_github").presentation.galleryAvatarSize, 83);
  assert.equal(moldFactory.get("agent_cursor").preferences.mode, "reasoning");
  assert.equal(moldFactory.get("integration_github").preferences.mode, "auto");
});

test("extensions apply to application services without changing their mold or service identity", () => {
  const { createPetFactory } = factoryModule;
  const factory = createPetFactory(source);
  for (const id of ["integration_github", "integration_resend"]) {
    const pet = factory.get(id);
    assert.equal(pet.mold, "application");
    assert.equal(pet.service, true);
    assert.equal(pet.preferences.provider, "codex");
    assert.equal(pet.preferences.fast.model, "gpt-6-luna");
    assert.equal(pet.presentation.profiles, true);
  }
  const spoty = factory.get("integration_spotify");
  assert.equal(spoty.mold, "application");
  assert.equal(spoty.presentation.profiles, false);
  assert.equal(spoty.preferences.provider, "");
  assert.equal(spoty.routing.chat, null);
  assert.equal(spoty.routing.execute, false);
});

test("factory results are immutable snapshots and configuration reads are isolated", () => {
  const { createPetFactory } = factoryModule;
  const mutableSource = clone(source);
  const factory = createPetFactory(mutableSource);
  const pet = factory.get("agent_codex");
  assert.ok(Object.isFrozen(pet));
  assert.ok(Object.isFrozen(pet.preferences));
  assert.throws(() => { pet.preferences.provider = "changed"; }, TypeError);
  mutableSource.pets.agent_codex.preferences.name = "Mutated after creation";
  assert.equal(factory.get("agent_codex").preferences.name, "Codex");
  const configuration = factory.configuration();
  configuration.reasoningTerms.push("injected");
  assert.equal(factory.configuration().reasoningTerms.includes("injected"), false);
});

test("task and profile provider mappings retain established behavior", () => {
  const { PetFactory } = factoryModule;
  assert.equal(factoryModule.petForTaskProvider("claude-cli"), "integration_claude");
  assert.equal(factoryModule.petForTaskProvider("gemini-cli"), "agent_gemini");
  assert.equal(factoryModule.taskProviderForPet("integration_claude"), "claude-cli");
  assert.equal(factoryModule.taskProviderForPet("agent_gemini"), "gemini-cli");
  assert.equal(factoryModule.petForTaskProvider("integration_spotify"), "integration_spotify");
  assert.equal(factoryModule.taskProviderForPet("integration_spotify"), "integration_spotify");
  assert.equal(factoryModule.factoryProfileProvider("codex"), true);
  assert.equal(factoryModule.factoryProfileProvider("github"), false);
});

test("unknown and prototype-like IDs fail closed", () => {
  const { PetFactory } = factoryModule;
  for (const id of ["unknown", "__proto__", "constructor", "toString"]) {
    assert.equal(PetFactory.get(id), null);
    assert.equal(factoryModule.petBlueprint(id), null);
  }
  for (const provider of [null, undefined, {}, [], 0, "", "unknown"]) {
    assert.equal(factoryModule.petForTaskProvider(provider), null);
    assert.equal(factoryModule.factoryProfileProvider(provider), false);
  }
});

test("malformed mold and extension definitions are rejected", () => {
  const { createPetFactory } = factoryModule;
  const badMold = clone(source);
  badMold.base.presentation.profiles = "yes";
  assert.throws(() => createPetFactory(badMold), /invalid bundled pet factory/i);
  const badExtension = clone(source);
  badExtension.extensions["codex-account"].presentation.contextAction = "launch";
  assert.throws(() => createPetFactory(badExtension), /invalid bundled pet factory/i);

  const unknownMold = clone(source);
  unknownMold.pets.agent_cursor.mold = "unknown";
  assert.throws(() => createPetFactory(unknownMold), /invalid bundled pet factory/i);

  const unknownExtension = clone(source);
  unknownExtension.pets.integration_github.extension = "missing-extension";
  assert.throws(() => createPetFactory(unknownExtension), /invalid bundled pet factory/i);

  const inheritedBasePermissions = clone(source);
  inheritedBasePermissions.base.preferences = { permissions: { documents: true } };
  assert.throws(() => createPetFactory(inheritedBasePermissions), /invalid bundled pet factory/i);

  const protectedExtensionPolicy = clone(source);
  protectedExtensionPolicy.extensions["codex-account"].preferences = { provider: "codex", localExecution: { mode: "always" } };
  assert.throws(() => createPetFactory(protectedExtensionPolicy), /invalid bundled pet factory/i);

  const missingPetPermissions = clone(source);
  delete missingPetPermissions.pets.agent_codex.preferences.permissions;
  assert.throws(() => createPetFactory(missingPetPermissions), /invalid bundled pet factory/i);
});

test("factory rejects duplicate logical routes while allowing shared Codex account providers", () => {
  const { createPetFactory } = factoryModule;
  assert.doesNotThrow(() => createPetFactory(source));

  const duplicateTaskRoute = clone(source);
  duplicateTaskRoute.pets.agent_copilot.routing.taskProvider = "codex";
  assert.throws(() => createPetFactory(duplicateTaskRoute), /invalid bundled pet factory/i);

  const duplicateChatRoute = clone(source);
  duplicateChatRoute.pets.integration_github.routing.chat = "codex";
  assert.throws(() => createPetFactory(duplicateChatRoute), /invalid bundled pet factory/i);
});

test("gallery renders the expected molds and shared sizing tokens for representative pets", () => {
  installFakeDom();
  const gallery = petGallery(() => {}, () => {});
  const pets = ["agent_codex", "agent_copilot", "ai_ollama", "integration_github", "integration_spotify"].map(identity);
  gallery.sync(pets);

  const expected = ["llm", "llm", "llm", "application", "application"];
  const cards = gallery.el.find("button");
  assert.equal(cards.length, pets.length);
  cards.forEach((card, index) => {
    assert.equal(card.dataset.pet, pets[index].id);
    assert.equal(card.dataset.petMold, expected[index]);
    assert.equal(card.style.borderRadius, "15px");
    assert.equal(card.style.gap, "4px");
  });
});

test("gallery context actions keep the latest exact pet identity for sessions and Spoty media", () => {
  installFakeDom();
  const opened = [], sessions = [];
  const gallery = petGallery((pet) => opened.push(pet), (pet) => sessions.push(pet));
  const pets = ["agent_codex", "agent_copilot", "ai_ollama", "integration_github", "integration_spotify"].map(identity);
  gallery.sync(pets);
  const cards = gallery.el.find("button");
  const latest = pets.map((pet) => ({ ...pet, state: "approval" }));
  gallery.sync(latest);

  cards.forEach((card) => card.oncontextmenu({ preventDefault() {} }));
  assert.deepEqual(sessions, latest.slice(0, 4));
  assert.equal(sessions[0], latest[0]);
  assert.equal(sessions[1], latest[1]);
  assert.equal(sessions[2], latest[2]);
  assert.equal(sessions[3], latest[3]);
  assert.deepEqual(opened, [latest[4]]);
});
