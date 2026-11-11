import {test} from "node:test";
import assert from "node:assert/strict";
import {mergeTasks,orderedTasks,taskCanStop,taskRecoverable,taskRunning,taskPet,taskProvider} from "../src/mochi/tasks.ts";
const task=(id,provider,phase,started,updated=started)=>({id,provider,phase,started,updated});
test("running work stays ahead of history without progress polling moving a stop target",()=>{
 const tasks=[task("old-done","codex","done",100),task("gravity","antigravity","working",200),task("codex","codex","thinking",300)];
 assert.deepEqual(orderedTasks(tasks).map(t=>t.id),["codex","gravity","old-done"]);
 tasks[1].updated=100000;
 assert.deepEqual(orderedTasks(tasks).map(t=>t.id),["codex","gravity","old-done"]);
 assert.deepEqual(orderedTasks(tasks,"antigravity").map(t=>t.id),["gravity"]);
});
test("external CLI monitoring cannot cancel or resume another session and own sessions are not counted twice",()=>{
 const own={...task("own","codex","working",1),backend:"codex",sessionId:"a"};
 const same={...own,id:"external-a",origin:"external"};
 const other={...same,id:"external-b",sessionId:"b"};
 const gravity={...same,id:"external-gravity",backend:"antigravity",provider:"antigravity"};
 assert.deepEqual(mergeTasks([own],[same,other,gravity]).map(t=>t.id),["own","external-b","external-gravity"]);
 assert.equal(taskCanStop(own),true);assert.equal(taskCanStop(other),false);
 assert.equal(taskRecoverable({...other,phase:"interrupted"}),false);
});
test("only interrupted, failed or stopped work can be resumed; completion and live requests cannot",()=>{
 for(const phase of ["interrupted","failed","stopped"])assert.equal(taskRecoverable({phase}),true);
 for(const phase of ["done","approval","question","working","resumed"])assert.equal(taskRecoverable({phase}),false);
 assert.equal(taskRunning({phase:"question"}),true);
 for(const phase of ["done","interrupted","resumed","unknown"])assert.equal(taskRunning({phase}),false);
});
test("task controls preserve the pet identity and never default an unknown job to another AI",()=>{
 for(const provider of ["codex","copilot","antigravity","gemini-cli","claude-cli"])assert.equal(taskProvider(taskPet(provider)),provider);
 assert.equal(taskPet("unknown"),null);assert.equal(taskProvider("integration_spotify"),"integration_spotify");
});

const {petCapabilities}=await import("../src/mochi/capabilities.ts");
const {availablePills}=await import("../src/core/pills.ts");
test("every Linux pet has an honest role and only implemented account bridges offer PC execution",()=>{
 const pets=availablePills("linux");assert.equal(pets.length,23);
 for(const pet of pets){const c=petCapabilities(pet.id);assert.ok(c.role.length>12,pet.id);
  if(c.execute)assert.ok(["codex","copilot","antigravity","gemini-cli","claude-cli","integration_github","integration_spotify","integration_resend","integration_notion","integration_n8n","integration_vercel","integration_calcom","integration_stripe"].includes(c.chat),pet.id);
 }
 for(const id of ["agent_cursor","agent_muse","agent_opencode","agent_amp","agent_hermes"]){assert.equal(petCapabilities(id).chat,null);assert.equal(petCapabilities(id).voice,null);}
 assert.equal(petCapabilities("ai_lmstudio").execute,false);assert.match(petCapabilities("ai_openai").role,/API/);
 assert.equal(petCapabilities("unknown").execute,false);
});

const {delegatedRequest}=await import("../src/mochi/delegation.ts");
test("service and monitor delegation identifies the actual source without inventing another execution backend",()=>{
 assert.equal(delegatedRequest("integration_github"),null,"services have their own assistant conversation");assert.equal(delegatedRequest("agent_copilot"),null);assert.match(delegatedRequest("agent_cursor").text,/Cursor/);
 assert.equal(delegatedRequest("unknown"),null);assert.equal(delegatedRequest("agent_codex"),null);assert.equal(delegatedRequest("ai_openai"),null);
});
