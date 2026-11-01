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

test("native services may declare an optional assistant without changing user defaults", () => {
  const s=clone(source), row=clone(s.pets.integration_spotify);
  row.preferences.name="Audio fixture"; row.preferences.aliases=["audio fixture"];
  for(const key of ["chat","voice","taskProvider"]) row.routing[key]="integration_audio_fixture";
  s.pets.integration_audio_fixture=row;
  const factory=factoryModule.createPetFactory(s), native=factory.get("integration_audio_fixture");
  assert.equal(native.runtime.adapter,"spotify"); assert.equal(native.runtime.assistantAdapter,"codex");
  assert.equal(native.preferences.provider,""); assert.equal(native.routing.chat,"integration_audio_fixture");
  assert.equal(factory.get("agent_codex").routing.chat,"codex");
});

test("an optional assistant cannot borrow another account or promote an unsupported runtime", () => {
  for(const [id,adapter] of [["integration_spotify","copilot"],["ai_ollama","codex"],["integration_notion","codex"]]) {
    const s=clone(source); s.pets[id].runtime.assistantAdapter=adapter;
    assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);
  }
  const s=clone(source); s.pets.integration_spotify.runtime.assistantAdapter=null;
  assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);
});

test("runtime metadata preserves independent logical pets and does not enable missing adapters", () => {
  const factory = factoryModule.createPetFactory(source);
  assert.deepEqual(factory.get("agent_codex").runtime, {kind:"llm",adapter:"codex"});
  assert.deepEqual(factory.get("integration_notion").runtime, {kind:"service",adapter:"codex"});
  assert.equal(factory.get("integration_notion").routing.taskProvider,"integration_notion");
  assert.deepEqual(factory.get("integration_spotify").runtime, {kind:"service",adapter:"spotify",assistantAdapter:"codex"});
  for(const id of ["ai_openai","ai_anthropic","ai_google"]){assert.equal(factory.get(id).runtime.kind,"llm");assert.equal(factory.get(id).runtime.adapter,null);assert.equal(factory.get(id).routing.execute,false);}
  assert.equal(factory.get("ai_lmstudio").runtime.adapter,"lmstudio");assert.equal(factory.get("ai_lmstudio").preferences.provider,"");assert.equal(factory.get("ai_lmstudio").routing.voice,null);assert.equal(factory.get("ai_lmstudio").routing.execute,false);assert.equal(factory.forTaskProvider("lmstudio"),"ai_lmstudio");
  assert.equal(factory.get("agent_gemini").runtime.kind,"observer");
  assert.equal(factory.get("integration_github").routing.execute,false);
  assert.deepEqual(factory.configuration(),legacy);
});

test("runtime declarations cannot be inherited or contain grants", () => {
  for(const recipe of ["base","llm","application","codex-account"]){
    for(const key of ["runtime","routing","service"]){const s=clone(source);const target=recipe==="base"?s.base:s.molds[recipe]??s.extensions[recipe];target[key]={};assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);}
  }
  for(const key of ["kind","adapter"]){const s=clone(source);delete s.pets.agent_codex.runtime[key];assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);}
  const s=clone(source);s.pets.agent_codex.runtime.permissions={};assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);
});

test("cross-channel owner collisions and incompatible adapter claims fail closed", () => {
  for(const mutate of [s=>s.pets.ai_openai.routing.chat="gemini-cli",s=>s.pets.agent_copilot.runtime.adapter="codex",s=>s.pets.ai_openai.runtime.adapter="openai",s=>s.pets.ai_ollama.routing.execute=true,s=>s.pets.integration_spotify.runtime.adapter="codex",s=>s.pets.agent_cursor.runtime.kind="llm"]){const s=clone(source);mutate(s);assert.throws(()=>factoryModule.createPetFactory(s),/invalid bundled pet factory/i);}
});

test("a new application owner reuses the implemented adapter with explicit permissions", () => {
  const s=clone(source),p=clone(s.pets.integration_notion);
  for(const key of ["chat","voice","taskProvider"])p.routing[key]="integration_fixture";
  p.preferences.name="Fixture";p.preferences.aliases=["fixture"];p.preferences.permissions={documents:false,emailDrafts:false,music:false,openApps:false};s.pets.integration_fixture=p;
  const factory=factoryModule.createPetFactory(s);
  assert.equal(factory.forTaskProvider("integration_fixture"),"integration_fixture");
  assert.equal(factory.get("integration_fixture").runtime.adapter,"codex");
  assert.ok(factory.services.includes("integration_fixture"));
  assert.deepEqual(factory.get("integration_fixture").preferences.permissions,p.preferences.permissions);
  assert.deepEqual(factory.get("integration_notion").preferences,legacy.pets.integration_notion);
});

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
  assert.equal(spoty.presentation.profiles, true);
  assert.equal(spoty.preferences.provider, "");
  assert.equal(spoty.routing.chat, "integration_spotify");
  assert.equal(spoty.routing.execute, true);
  assert.equal(spoty.runtime.assistantAdapter, "codex");
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
