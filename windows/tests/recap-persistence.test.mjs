import {afterEach,test} from 'node:test';import assert from 'node:assert/strict';
import {Bridge} from '../src/core/bridge.ts';import {internals} from './tauri.mjs';
import {sampleHistory,previousWeekStart} from '../src/recap/summary.ts';
import {State} from '../src/core/state.ts';
import {RecapController,parseRecapPrefs,parseRecapReceipt,recapReceiptSuccess,recapFailureMessage} from '../src/recap/recap.ts';
const original=internals.invoke;afterEach(()=>{internals.invoke=original;});
const prefs=(extra={})=>({enabled:true,hideProjects:false,lastShownWeek:'',...extra});
const receipt=(extra={})=>({prefs:prefs(),persisted:true,historyCleared:false,errorCode:null,...extra});
const defer=()=>{let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return {promise,resolve,reject};};
test('closed receipt refuses legacy void, wrong booleans and contradictory success, preserving effective RAM prefs',()=>{
 for(const v of [null,{},receipt({persisted:'true'}),receipt({prefs:prefs({enabled:'false'})}),receipt({historyCleared:true,persisted:false,errorCode:'writeFailed'}),receipt({extra:true}),receipt({errorCode:'other'}),receipt({errorCode:'writeFailed'}),receipt({persisted:false,errorCode:null})])assert.throws(()=>parseRecapReceipt(v));
 const failed=parseRecapReceipt(receipt({prefs:prefs({enabled:false,hideProjects:true}),persisted:false,errorCode:'writeFailed'}));assert.equal(failed.prefs.enabled,false);assert.equal(failed.prefs.hideProjects,true);assert.equal(failed.persisted,false);assert.equal(parseRecapPrefs(prefs()).enabled,true);
});
test('clear succeeds only with persistence AND cleared confirmation; other operation receipts do not certify clear',()=>{
 assert.equal(recapReceiptSuccess('clear',receipt()),false);assert.equal(recapReceiptSuccess('clear',receipt({historyCleared:true})),true);assert.equal(recapReceiptSuccess('clear',receipt({persisted:false,errorCode:'writeFailed'})),false);
 assert.equal(recapReceiptSuccess('enabled',receipt({prefs:prefs({enabled:false})}),false),true);assert.equal(recapReceiptSuccess('enabled',receipt(),false),false);
 assert.match(recapFailureMessage('clear',receipt({persisted:false,errorCode:'writeFailed'})),/preservado/);
});
test('privacy write failure is distinct from a failed enable/reveal and cannot claim disk success',()=>{
 assert.match(recapFailureMessage('enabled',receipt({prefs:prefs({enabled:false}),persisted:false,errorCode:'writeFailed'}),false),/desativado nesta sessão/);
 assert.match(recapFailureMessage('hide',receipt({prefs:prefs({hideProjects:true}),persisted:false,errorCode:'writeFailed'}),true),/ocultos nesta sessão/);
 assert.doesNotMatch(recapFailureMessage('enabled',receipt({prefs:prefs({enabled:false}),persisted:false,errorCode:'writeFailed'}),true),/guardada|ativado nesta sessão/);
});
test('all explicit mutation bridge calls reject IPC refusal instead of returning false acknowledgement',async()=>{
 const calls=[];internals.invoke=async(c,a)=>{calls.push([c,a]);return receipt();};await Bridge.recapSetEnabled(false);await Bridge.recapSetHideProjects(true);await Bridge.recapMarkShown('2026-10-19');await Bridge.recapClear();assert.deepEqual(calls,[['recap_set_enabled',{enabled:false}],['recap_set_hide_projects',{hide:true}],['recap_mark_shown',{week:'2026-10-19'}],['recap_clear',{}]]);
 internals.invoke=async()=>{throw Error('transport refused');};for(const call of [()=>Bridge.recapSetEnabled(false),()=>Bridge.recapSetHideProjects(true),()=>Bridge.recapMarkShown('2026-10-19'),()=>Bridge.recapClear(),()=>Bridge.recapPrefs()])await assert.rejects(call(),/transport refused/);
});
test('failed hide retains RAM protection and exposes explicit retry, failed reveal retains safe returned state',async()=>{
 const c=new RecapController();c.prefs=prefs();c.prefsKnown=true;internals.invoke=async()=>receipt({prefs:prefs({hideProjects:true}),persisted:false,errorCode:'writeFailed'});const r=await c.setHideProjects(true);assert.equal(r.persisted,false);assert.equal(c.prefs.hideProjects,true);assert.equal(c.prefsKnown,true);assert.equal(c.retryHide,true);assert.match(c.notice,/nesta sessão/);
 await c.setHideProjects(false);assert.equal(c.prefs.hideProjects,true);assert.equal(c.retryHide,false);assert.doesNotMatch(c.notice,/guardada/);
});
test('transport refusal does not invent effective native preferences or a persisted acknowledgement',async()=>{
 const c=new RecapController();c.prefs=prefs({hideProjects:true});c.prefsKnown=true;internals.invoke=async()=>{throw Error('private path');};assert.equal(await c.setHideProjects(false),null);assert.equal(c.prefs.hideProjects,true);assert.equal(c.prefsKnown,false);assert.equal(c.hidePending,false);assert.doesNotMatch(c.notice,/private path/);
});
test('pending preference write is singleflight and metadata started earlier cannot overwrite its receipt',async()=>{
 const c=new RecapController(),read=defer(),write=defer();let writes=0;internals.invoke=(command)=>command==='recap_prefs'?read.promise:(writes++,write.promise);const before=c.readPreferences(),saving=c.setHideProjects(true);assert.equal(c.hidePending,true);assert.equal(await c.setHideProjects(false),null);assert.equal(writes,1);write.resolve(receipt({prefs:prefs({hideProjects:true})}));await saving;read.resolve(prefs());assert.equal(await before,false);assert.equal(c.prefs.hideProjects,true);assert.equal(c.prefsKnown,true);
});
test('latest read wins over older bootstrap and an older history read cannot reset fresh preferences',async()=>{
 const c=new RecapController(),old=defer(),newer=defer();let n=0;internals.invoke=()=>++n===1?old.promise:newer.promise;const a=c.readPreferences(),b=c.readPreferences();newer.resolve(prefs({enabled:false,hideProjects:true}));await b;old.resolve(prefs());assert.equal(await a,false);assert.equal(c.prefs.enabled,false);
 const history=defer();internals.invoke=command=>command==='recap_history'?history.promise:Promise.resolve(prefs({enabled:false,hideProjects:true}));const loading=c.load();await c.readPreferences();history.resolve({turns:[],decisions:[],prefs:prefs()});await loading;assert.equal(c.prefs.hideProjects,true);assert.equal(c.prefs.enabled,false);
});
for(const outcome of ['success','writeFailed','rejected','null'])test(`metadata started during hide cannot replace its settled ${outcome} state`,async()=>{
 const c=new RecapController(),write=defer(),read=defer();c.prefs=prefs();c.prefsKnown=true;
 internals.invoke=command=>command==='recap_prefs'?read.promise:write.promise;
 const saving=c.setHideProjects(true),observing=c.readPreferences();
 if(outcome==='rejected')write.reject(Error('private RPC error'));else write.resolve(outcome==='null'?null:receipt({prefs:prefs({hideProjects:true}),persisted:outcome==='success',errorCode:outcome==='success'?null:'writeFailed'}));
 await saving;const acknowledged={...c.prefs},known=c.prefsKnown,notice=c.notice,retry=c.retryHide;read.resolve(prefs());assert.equal(await observing,false);assert.deepEqual(c.prefs,acknowledged);assert.equal(c.prefsKnown,known);assert.equal(c.notice,notice);assert.equal(c.retryHide,retry);
 assert.equal(c.prefsKnown,outcome==='success'||outcome==='writeFailed');assert.equal(c.prefs.hideProjects,outcome==='success'||outcome==='writeFailed');
});
for(const operation of ['load','open'])test(`${operation} sampled during hide does not reinstall old history preferences after ACK`,async()=>{
 const c=new RecapController(),write=defer(),read=defer(),now=new Date(2026,9,19,9),shown=[];const old={...sampleHistory(previousWeekStart(now)),prefs:prefs()};c.prefs=prefs();c.prefsKnown=true;
 internals.invoke=command=>command==='recap_history'?read.promise:write.promise;const saving=c.setHideProjects(true),observing=operation==='load'?c.load(now):c.open({alert:view=>shown.push(view)});
 write.resolve(receipt({prefs:prefs({hideProjects:true})}));await saving;read.resolve(old);await observing;assert.equal(c.prefs.hideProjects,true);assert.equal(c.prefsKnown,true);if(operation==='open')assert.deepEqual(shown,['recap']);
});
test('late preference rejection sampled during hide cannot turn a valid ACK into unknown',async()=>{
 const c=new RecapController(),write=defer(),read=defer();internals.invoke=command=>command==='recap_prefs'?read.promise:write.promise;const saving=c.setHideProjects(true),observing=c.readPreferences();write.resolve(receipt({prefs:prefs({hideProjects:true})}));await saving;read.reject(Error('private stale error'));assert.equal(await observing,false);assert.equal(c.prefsKnown,true);assert.equal(c.prefs.hideProjects,true);assert.equal(c.notice,'');
});
test('a read completed while hide remains pending does not publish a preference observation',async()=>{
 const c=new RecapController(),write=defer(),read=defer();let events=0;c.subscribe(()=>events++);internals.invoke=command=>command==='recap_prefs'?read.promise:write.promise;const saving=c.setHideProjects(true),observing=c.readPreferences();const before=events;read.resolve(prefs());assert.equal(await observing,false);assert.equal(events,before);assert.equal(c.hidePending,true);write.resolve(receipt({prefs:prefs({hideProjects:true})}));await saving;assert.equal(c.prefs.hideProjects,true);
});
test('fresh observations after settlement remain admissible, including a genuinely newer different state',async()=>{
 const c=new RecapController(),write=defer();let current=prefs({hideProjects:true});internals.invoke=command=>command==='recap_prefs'?Promise.resolve(structuredClone(current)):write.promise;const saving=c.setHideProjects(true);write.resolve(receipt({prefs:current}));await saving;assert.equal(await c.readPreferences(),true);assert.equal(c.prefs.hideProjects,true);current=prefs();assert.equal(await c.readPreferences(),true);assert.equal(c.prefs.hideProjects,false);
});
test('automatic checking does not issue metadata or present above pending privacy, then works after settlement',async()=>{
 const c=new RecapController(),write=defer(),commands=[];State.mode='hidden';State.view='overview';State.paused=false;State.pendingApproval=null;
 internals.invoke=command=>{commands.push(command);if(command==='recap_set_hide_projects')return write.promise;if(command==='recap_prefs')return Promise.resolve(prefs({hideProjects:true}));return Promise.resolve({turns:[],decisions:[],prefs:prefs({hideProjects:true})});};
 const saving=c.setHideProjects(true);await c.check({alert:()=>assert.fail('No presentation during privacy write')},new Date(2026,9,19,9));assert.deepEqual(commands,['recap_set_hide_projects']);write.resolve(receipt({prefs:prefs({hideProjects:true})}));await saving;await c.check({alert:()=>assert.fail('Empty week')},new Date(2026,9,19,9));assert.deepEqual(commands,['recap_set_hide_projects','recap_prefs','recap_history']);
});
