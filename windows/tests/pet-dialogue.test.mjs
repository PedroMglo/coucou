import { test } from "node:test";
import assert from "node:assert/strict";
import { PetDialogues, phaseFor, animationFor } from "../src/mochi/dialogue.ts";
const event=(pet,turn,phase,message="")=>({pet,turn,phase,message});
test("working, public messages, approval and resumption keep their owning pet",()=>{
  const d=new PetDialogues();
  d.update(event("codex",1,"working"),0);
  d.update(event("codex",1,"message","Vou preparar o Postgres."),100);
  d.update(event("codex",1,"approval"),200);
  assert.equal(d.visible("codex",600000,false).phase,"approval");
  assert.equal(d.visible("codex",600000,false).message,"Vou preparar o Postgres.");
  assert.equal(d.visible("gravity",600000,false),undefined);
  d.update(event("codex",1,"working"),600001);
  assert.equal(phaseFor(d.entries.get("codex"),"idle"),"working");
});
test("progress never covers an open decision card and native approvals have priority",()=>{
  const d=new PetDialogues();d.update(event("codex",2,"message","Olá"),0);
  assert.equal(d.visible("codex",10,true),undefined);
  assert.equal(phaseFor(d.entries.get("codex"),"approval"),"approval");
  assert.equal(animationFor("question","idle"),"question");
});
test("late events cannot resurrect a cancelled turn or replace another dialogue",()=>{
  const d=new PetDialogues();d.update(event("codex",2,"done","Pronto"),0);
  assert.equal(d.update(event("codex",1,"working"),1),false);
  assert.equal(d.entries.get("codex").phase,"done");
  assert.equal(d.visible("codex",10000,false),undefined);
  d.update(event("gravity",3,"working"),10001);
  assert.equal(d.visible("codex",10002,false),undefined);
  assert.equal(animationFor("message","idle"),"working");
});

test("a completed conversation stays complete while another CLI session works",()=>{
 const d=new PetDialogues();d.update(event("codex",4,"done","Está pronto."),0);
 assert.equal(phaseFor(d.current("codex",100),"working"),"done");
 assert.equal(d.current("codex",10000),undefined);
 assert.equal(phaseFor(d.current("codex",10000),"working"),"working");
});
