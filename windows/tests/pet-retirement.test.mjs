import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFileSync} from 'node:fs';
import {createPetFactory,PetFactory,petRetired,petForTaskProvider,factoryProfileProvider} from '../src/core/pet-factory.ts';
import {providerForPill} from '../src/core/providers.ts';
import {isServiceProvider} from '../src/mochi/pet-preferences.ts';
import {petCapabilities} from '../src/mochi/capabilities.ts';
import {delegatedRequest} from '../src/mochi/delegation.ts';
import {availablePills,pillDefinition,sanitizeDeclared,toggleDeclared} from '../src/core/pills.ts';
const source=JSON.parse(readFileSync(new URL('../src-tauri/pets.factory.json',import.meta.url),'utf8'));
const legacy=JSON.parse(readFileSync(new URL('../src-tauri/pets.default.json',import.meta.url),'utf8'));
test('retired GitHub keeps preferences/history identity while new routes are disabled',()=>{
 const pet=PetFactory.get('integration_github');assert.equal(pet.retired,true);assert.deepEqual(pet.preferences,legacy.pets.integration_github);
 assert.equal(pet.routing.chat,'integration_github');assert.equal(pet.routing.voice,'integration_github');assert.equal(pet.routing.execute,false);
 assert.equal(providerForPill('integration_github'),'integration_github');assert.equal(petCapabilities('integration_github').chat,null);assert.equal(delegatedRequest('integration_github'),null);assert.equal(petForTaskProvider('integration_github'),'integration_github');
 assert.equal(isServiceProvider('integration_github'),true);assert.equal(factoryProfileProvider('integration_github'),false);
 assert.deepEqual(PetFactory.configuration(),legacy);
 for(const id of ['agent_codex','agent_copilot','agent_antigravity','ai_ollama','integration_notion'])assert.equal(petRetired(id),false);
 assert.equal(providerForPill('agent_copilot'),'copilot');
});
test('old declarations cannot reintroduce retired GitHub and the catalog remains addressable',()=>{
 const saved={mainPill:'agent_codex',activeIntegrations:['integration_github','agent_copilot','ai_ollama']};const before=structuredClone(saved);
 for(const os of ['linux','windows']){
  assert.equal(availablePills(os).some(p=>p.id==='integration_github'),false);
  assert.deepEqual(sanitizeDeclared(saved,os).activeIntegrations,['agent_copilot','ai_ollama']);
  assert.equal(toggleDeclared(saved,'integration_github',os),null);
 }
 assert.deepEqual(saved,before);assert.equal(pillDefinition('integration_github').id,'integration_github');
});
test('retirement is instance-only typed metadata and cannot alter stored preferences',()=>{
 for(const value of ['true',null,1,[],{}]){const bad=structuredClone(source);bad.pets.integration_github.retired=value;assert.throws(()=>createPetFactory(bad),/invalid bundled/i);}
 for(const target of [['base'],['molds','llm'],['molds','application'],['extensions','codex-account']]){
  const bad=structuredClone(source);let recipe=bad;for(const part of target)recipe=recipe[part];recipe.retired=true;assert.throws(()=>createPetFactory(bad),/invalid bundled/i);
 }
 const restored=structuredClone(source);restored.pets.integration_github.retired=false;const factory=createPetFactory(restored);
 assert.equal(factory.get('integration_github').routing.chat,'integration_github');assert.deepEqual(factory.configuration(),legacy);
});
