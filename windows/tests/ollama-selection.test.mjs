import {test} from 'node:test';
import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';
import {State,DEFAULT_SETTINGS} from '../src/core/state.ts';
import {OllamaSelection} from '../src/core/ollama-selection.ts';
const ack=(model,selectionId)=>({provider:'ollama',model,selectionId,generation:1});
test('a slower A cannot replace B after its confirmation, even if A succeeds',async()=>{
 const OllamaModelSelection=new OllamaSelection();
 const original=Bridge.petModelSelect,pending=new Map();State.settings={...DEFAULT_SETTINGS,chatModels:{ollama:'original'}};
 Bridge.petModelSelect=(model,id)=>new Promise(resolve=>pending.set(model,()=>resolve(ack(model,id))));
 try{
  const a=OllamaModelSelection.select('a'),b=OllamaModelSelection.select('b');assert.equal(OllamaModelSelection.pending,true);assert.equal(State.settings.chatModels.ollama,'original');
  pending.get('b')();assert.equal(await b,true);assert.equal(State.settings.chatModels.ollama,'b');
  pending.get('a')();assert.equal(await a,false);assert.equal(State.settings.chatModels.ollama,'b');assert.equal(OllamaModelSelection.pending,false);
 }finally{Bridge.petModelSelect=original;}
});
test('a failed or mismatched confirmation preserves the model and releases the pending state',async()=>{
 const OllamaModelSelection=new OllamaSelection();
 const original=Bridge.petModelSelect;State.settings={...DEFAULT_SETTINGS,chatModels:{ollama:'original'}};
 try{
  Bridge.petModelSelect=async()=>{throw Error('Não foi possível guardar');};await assert.rejects(OllamaModelSelection.select('new'),/guardar/);
  Bridge.petModelSelect=async(model,id)=>ack('different',id);await assert.rejects(OllamaModelSelection.select('new'),/confirmada/);
  assert.equal(State.settings.chatModels.ollama,'original');assert.equal(OllamaModelSelection.pending,false);
  Bridge.petModelSelect=async(model,id)=>ack(model,id);assert.equal(await OllamaModelSelection.select('@default'),true);assert.equal(State.settings.chatModels.ollama,'@default');
 }finally{Bridge.petModelSelect=original;}
});
test('a late ACK cannot replace a newer selection observed from another window',async()=>{
 const selector=new OllamaSelection(),original=Bridge.petModelSelect;let finish;
 State.settings={...DEFAULT_SETTINGS,chatModels:{ollama:'previous'}};
 Bridge.petModelSelect=(model,selectionId)=>new Promise(resolve=>finish=()=>resolve(ack(model,selectionId)));
 try{
  const pending=selector.select('model-a');
  State.settings=selector.applySettings({...State.settings,chatModels:{ollama:'model-b'},ollamaSelectionGeneration:2});
  finish();assert.equal(await pending,false);assert.equal(State.settings.chatModels.ollama,'model-b');
  State.settings=selector.applySettings({...State.settings,chatModels:{ollama:'model-a'},ollamaSelectionGeneration:1});assert.equal(State.settings.chatModels.ollama,'model-b');
  State.settings=selector.applySettings({...State.settings,chatModels:{ollama:'old-unversioned'}});assert.equal(State.settings.chatModels.ollama,'model-b');
  State.settings=selector.applySettings({...State.settings,chatModels:{ollama:'model-a'},ollamaSelectionGeneration:3});assert.equal(State.settings.chatModels.ollama,'model-a');
 }finally{Bridge.petModelSelect=original;}
});
