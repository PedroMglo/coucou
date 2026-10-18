import {test} from "node:test";
import assert from "node:assert/strict";
import {shelterLayout,speechPlacement,speechText} from "../src/mochi/shelter.ts";
import {CompanionWorld,companionIdentities} from "../src/mochi/companions.ts";

test("all configured pets have an accessible home without launching an application",()=>{
 const band={height:42,lanes:[[10,100],[841,1078]]};
 for(const expanded of [false,true]){
  const home=shelterLayout(band,6,expanded);
  assert.ok(home.left>=841 && home.left+home.width<=1078);
  assert.ok(home.top>=0 && home.top+home.height<=42);
  for(const [a,b] of home.lanes)assert.ok(b<=home.left || a>=home.left+home.width);
 }
 assert.equal(shelterLayout({height:42,lanes:[]},6,false),null);
});

test("speech follows its moving pet inside free bar space without covering another pet",()=>{
 for(const height of [24,32,42,80]) {
 const band={height,lanes:[[100,500]]};
 for(let x=125;x<=475;x+=.5){
  const others=[x<300?475:125];
  const r=speechPlacement(x,21,.6,band,900,others);
  if(!r)continue;
  assert.ok(r.left>=100 && r.left+r.width<=500);
  assert.ok(r.top>=0 && r.top+r.height<=height);
  assert.ok(r.left>=x+14 || r.left+r.width<=x-14);
  assert.ok(others.every(o=>r.left>=o+14 || r.left+r.width<=o-14));
 }
 }
});

test("thinking, work and listening use expressions, while actual speech expires once",()=>{
 for(const phase of ['thinking','working','preparing','listening']) assert.equal(speechText(phase,'',0,100),'');
 assert.equal(speechText('message','Vou preparar o serviço.',100,101),'Vou preparar o serviço.');
 assert.equal(speechText('working','Vou preparar o serviço.',100,9200),'');
 assert.equal(speechText('speaking','Olá',0,20000),'Olá');
 assert.equal(speechText('approval','',0,20000),'Preciso da tua aprovação');
});

test("quiet pets enter the shelter, activity recalls only its owner, and decisions stay awake",()=>{
 const tasks=['agent_codex','agent_antigravity'].map(id=>({id,name:id,color:'#88aaff',state:'idle',steps:[],stepIndex:0,source:'agent',sessionId:'open'}));
 const pets=companionIdentities(tasks,null,false,'none');
 const s={enabled:true,paused:false,motion:true,sheltered:true,pets,band:{height:42,lanes:[[0,500]]}};
 const w=new CompanionWorld();w.sync(s,500,0);w.tick(.1,.5);
 assert.equal(w.pets.get('agent_codex').shown,true);
 w.tick(.1,61);w.tick(.1,61.5);
 assert.equal([...w.pets.values()].filter(p=>p.shown).length,0);
 w.touch('agent_codex',62);w.tick(.1,62.5);
 assert.equal(w.pets.get('agent_codex').shown,true);
 assert.equal(w.pets.get('agent_antigravity').shown,false);
 w.sync({...s,pets:pets.map(p=>p.id==='agent_antigravity'?{...p,state:'approval',attention:true}:p)},500,63);
 w.tick(.1,3600);w.tick(.1,3601);
 assert.equal(w.pets.get('agent_antigravity').shown,true);
});
