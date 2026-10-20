import assert from 'node:assert/strict';
import {after, beforeEach, test} from 'node:test';
import {Island} from '../src/island/island.ts';
import {DesktopLink} from '../src/island/desktop.ts';
import {CompanionsLink} from '../src/island/companions.ts';
import {State, DEFAULT_SETTINGS} from '../src/core/state.ts';
import {Sound} from '../src/core/sound.ts';
import {calls, internals} from './tauri.mjs';

const savedSound = Sound.enabled;
beforeEach(() => {
  State.settings = {...DEFAULT_SETTINGS, companionMode: false, petSystemCenter: true};
  State.mode = 'hidden'; State.view = 'overview'; State.pendingApproval = null;
  State.isPinned = false; State.focusId = 'ai_ollama'; calls.length = 0;
  State.tasks = ['ai_ollama', 'agent_codex'].map(id => ({id, name: id, color: '#94A3B8', state: 'idle', steps: [], stepIndex: 0}));
  Sound.setEnabled(false);
});
after(() => Sound.setEnabled(savedSound));

function island() {
  const card = Object.create(Island.prototype);
  const effects = {desktop: 0, launch: 0, hidden: 0, petit: 0, reveal: 0, entered: 0};
  card.desktop = {supported: true, centerSupported: true, companionsActive: false, launch() {effects.desktop++;}};
  card.fsm = {
    state: 'hidden', pinned: false,
    launch() {effects.launch++;}, forceHidden() {effects.hidden++;},
    forcePetit() {effects.petit++;}, reveal() {effects.reveal++;},
    mouseEntered() {effects.entered++;}, mouseLeft() {}, forceHome() {},
  };
  card.setMode = mode => {State.mode = mode;};
  card.animateGeometry = () => {};
  card.wasInIsland = false;
  card.engine = {animateMorph() {}};
  card.presentApproval=context=>{card.expand(context.view);return Promise.resolve(true);};
  return {card, effects};
}

test('system-only startup hides the classic island without launching desktop pets', () => {
  const {card, effects} = island(); card.launch();
  assert.equal(effects.hidden, 1); assert.equal(effects.desktop, 1);
  assert.equal(effects.launch, 0); assert.equal(card.collapsed, true);
  assert.ok(calls.some(([cmd, args]) => cmd === 'set_collapsed' && args.collapsed === true));
});

test('system-only launch configures the shared centre while a saved desktop pet stays off', async () => {
  const link = Object.create(DesktopLink.prototype);
  const effects = {centre: 0, fly: 0};
  link.mode = 'native';
  link.companions = {active: false, async launch() {effects.centre++;}};
  link.controller = {enabled: true, launchFlyIfNeeded() {effects.fly++;}};
  State.mochiOnDesktop = false;
  await link.launch();
  assert.equal(effects.centre, 1); assert.equal(effects.fly, 0);
  assert.equal(State.mochiOnDesktop, false);
});

for (const system of [true, false]) test(`actual startup chain configures and keeps syncing with companions off and System ${system}`, async () => {
  State.settings.petSystemCenter = system;
  const invoke = internals.invoke;
  internals.invoke = async (cmd, args) => {
    if (cmd === 'companions_configure') {
      calls.push([cmd, args]);
      return {width: 1440, height: 900, top: 44, band: null};
    }
    return invoke(cmd, args);
  };
  try {
    const {card} = island(), link = Object.create(DesktopLink.prototype);
    const companions = new CompanionsLink(() => {}, () => 'none');
    companions.supported = true;
    let fly = 0;
    link.mode = 'native'; link.companions = companions;
    link.controller = {enabled: true, launchFlyIfNeeded() {fly++;}};
    card.desktop = link; card.launch();
    for (let i = 0; i < 64; i++) await Promise.resolve();
    assert.ok(calls.some(([cmd]) => cmd === 'pet_control_configure'));
    const updates = () => calls.filter(([cmd, args]) => cmd === 'plugin:event|emit_to' && args.target.label === 'pet-control').map(([, args]) => args.payload);
    assert.equal(updates().at(-1).enabled, false);
    assert.equal(updates().at(-1).control.systemCenter, system);
    State.settings.petControlBar = false;
    await companions.sync();
    assert.equal(updates().at(-1).control.enabled, false);
    assert.equal(updates().at(-1).control.systemCenter, system);
    assert.ok(calls.filter(([cmd]) => cmd === 'companions_configure').every(([, args]) => args.enabled === false));
    assert.equal(fly, 0);
  } finally {internals.invoke = invoke;}
});

test('integrated startup still launches explicitly enabled desktop pets', () => {
  State.settings.companionMode = true;
  const {card, effects} = island(); card.launch();
  assert.equal(effects.hidden, 1); assert.equal(effects.desktop, 1); assert.equal(effects.launch, 0);
});

test('the original fallback remains usable only when the shared surface is unavailable', () => {
  State.settings.petSystemCenter = false;
  const {card, effects} = island(); card.desktop.centerSupported = false;
  card.launch(); card.reveal(); card.revealSilently();
  assert.equal(effects.launch, 1); assert.equal(effects.hidden, 0); assert.equal(effects.reveal, 2);
});

test('turning both modes off on a supported desktop cannot revive the old island', () => {
  State.settings.petSystemCenter = false;
  const {card, effects} = island();
  card.launch(); card.reveal(); card.revealSilently(); card.onCursor(380, 4);
  assert.equal(effects.hidden, 1); assert.equal(effects.desktop, 1);
  assert.equal(effects.launch, 0); assert.equal(effects.reveal, 0); assert.equal(effects.entered, 0);
});

test('system-only music and cursor events cannot reopen the classic island', () => {
  const {card, effects} = island();
  card.reveal(); card.revealSilently(); card.onCursor(380, 4);
  assert.equal(effects.reveal, 0); assert.equal(effects.entered, 0);
  assert.equal(card.wasInIsland, false); assert.equal(State.mode, 'hidden');
});

test('a delayed compact transition yields to System without launching a pet', () => {
  const {card, effects} = island(); card.wireFsm();
  card.fsm.onTransition('home', 'petit');
  assert.equal(effects.hidden, 1); assert.equal(effects.desktop, 0);
  assert.equal(State.mode, 'hidden');
});

test('closing or folding a system-only card hides it and preserves the exact decision', () => {
  const {card, effects} = island();
  State.mode = 'expanded'; State.pendingApproval = {pillId: 'ai_ollama', requestId: 'exact-review'};
  card.foldApproval(); card.collapse();
  assert.equal(effects.hidden, 2); assert.equal(effects.petit, 0);
  assert.equal(State.pendingApproval.requestId, 'exact-review');
  assert.equal(calls.some(([cmd]) => /answer|approve|deny/.test(cmd)), false);
});

test('system-only explicit pet opening still resolves the exact approval or document', () => {
  const {card} = island();
  State.pendingApproval = {pillId: 'ai_ollama', requestId: 'exact-review'};
  card.openFromDesktop('ai_ollama', 'overview');
  assert.equal(State.mode, 'expanded'); assert.equal(State.view, 'approval');
  assert.equal(State.pendingApproval.requestId, 'exact-review');
  card.openFromDesktop('agent_codex', 'choose');
  assert.equal(State.view, 'choose'); assert.equal(State.focusId, 'agent_codex');
  assert.equal(State.pendingApproval.requestId, 'exact-review');
});
