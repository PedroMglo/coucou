import {beforeEach,afterEach,mock,test} from 'node:test';
import assert from 'node:assert/strict';
import {CardLifecycle} from '../src/island/card-lifecycle.ts';
let card,states;
beforeEach(()=>{mock.timers.enable({apis:['setTimeout']});card=new CardLifecycle();states=[];card.onTransition=(_from,to)=>states.push(to);});
afterEach(()=>mock.timers.reset());
test('startup, playback and a slow cursor cannot open a card or create a compact state',()=>{
 card.openOnHover=true;card.launch();card.reveal();card.mouseEntered();card.click();card.greetComplete();mock.timers.tick(120000);
 assert.equal(card.state,'hidden');assert.deepEqual(states,[]);
});
test('leaving an explicit card closes directly, with no compact or wake interval',()=>{
 card.homeToPetitDelay=3;card.forceHome();card.mouseLeft();mock.timers.tick(3000);
 assert.equal(card.state,'hidden');assert.deepEqual(states,['home','hidden']);
});
test('editing and approval pins retain a card, while explicit close preserves those holds',()=>{
 card.forceHome();card.mouseLeft();card.editing=true;mock.timers.tick(60000);assert.equal(card.state,'home');
 card.editing=false;card.pinned=true;card.mouseLeft();mock.timers.tick(60000);assert.equal(card.state,'home');
 card.forcePetit();assert.equal(card.state,'hidden');assert.equal(card.pinned,true);
});
test('pointer return and preference updates control only the current card countdown',()=>{
 card.forceHome();card.mouseLeft();card.mouseEntered();mock.timers.tick(60000);assert.equal(card.state,'home');
 card.mouseLeft();card.homeToPetitDelay=1;mock.timers.tick(1000);assert.equal(card.state,'hidden');
 card.mouseEntered();assert.equal(card.state,'hidden');
});


test('manual cards close within 50ms when both pet and card are outside',()=>{
 card.setManual(true,true);card.forceHome();card.mouseLeft();mock.timers.tick(0);assert.equal(card.state,'home');
 card.setPetInside(false);mock.timers.tick(49);assert.equal(card.state,'home');mock.timers.tick(1);assert.equal(card.state,'hidden');
});
test('pet to card and card to pet tolerate a separately sampled native enter 33ms later without reopening',()=>{
 card.setManual(true,true);card.forceHome();card.setPetInside(false);mock.timers.tick(33);assert.equal(card.state,'home');
 card.mouseEntered();mock.timers.tick(100);assert.equal(card.state,'home');
 card.mouseLeft();mock.timers.tick(33);assert.equal(card.state,'home');card.setPetInside(true);mock.timers.tick(100);assert.equal(card.state,'home');
 assert.deepEqual(states,['home']);
 card.setPetInside(false);mock.timers.tick(50);assert.equal(card.state,'hidden');card.mouseEntered();assert.equal(card.state,'hidden');
});
test('manual editing does not hold a card outside the pointer; automatic editing still does',()=>{
 card.setManual(true,true);card.forceHome();card.editing=true;card.setPetInside(false);mock.timers.tick(50);assert.equal(card.state,'hidden');
 card.setManual(false);card.forceHome();card.mouseLeft();mock.timers.tick(60000);assert.equal(card.state,'home');
});
test('live decision and document guards protect manual cards without changing pins',()=>{
 let held=true;card=new CardLifecycle(()=>held);card.setManual(true);card.forceHome();card.mouseLeft();mock.timers.tick(60000);assert.equal(card.state,'home');assert.equal(card.pinned,false);
 held=false;card.refreshHold();mock.timers.tick(50);assert.equal(card.state,'hidden');
});
test('an automatic presentation cancels a pending manual close and keeps its original delay',()=>{
 card.setManual(true);card.forceHome();card.mouseLeft();card.setManual(false);card.forceHome();mock.timers.tick(0);assert.equal(card.state,'home');
 card.mouseLeft();mock.timers.tick(14999);assert.equal(card.state,'home');mock.timers.tick(1);assert.equal(card.state,'hidden');
});

test('repeated native outside samples do not extend the 50ms manual deadline',()=>{
 card.setManual(true,true);card.forceHome();card.setPetInside(false);
 const due=card.homeCollapseDueAt;
 mock.timers.tick(20);card.mouseLeft();assert.equal(card.homeCollapseDueAt,due);
 mock.timers.tick(20);card.setPetInside(false);card.refreshHold();assert.equal(card.homeCollapseDueAt,due);
 mock.timers.tick(9);assert.equal(card.state,'home');mock.timers.tick(1);assert.equal(card.state,'hidden');
});

test('a decision or document arriving during handoff protects the live card until its hold is released',()=>{
 let held=false;card=new CardLifecycle(()=>held);card.setManual(true);card.forceHome();card.mouseLeft();
 mock.timers.tick(33);held=true;mock.timers.tick(100);assert.equal(card.state,'home');
 held=false;card.refreshHold();mock.timers.tick(49);assert.equal(card.state,'home');mock.timers.tick(1);assert.equal(card.state,'hidden');
});

test('replacing a manual card cancels its old deadline instead of closing the new pet',()=>{
 card.setManual(true);card.forceHome();card.mouseLeft();mock.timers.tick(33);
 card.setManual(true,true);card.forceHome();mock.timers.tick(100);assert.equal(card.state,'home');
 card.setPetInside(false);mock.timers.tick(50);assert.equal(card.state,'hidden');
});

test('pinning during the short manual handoff protects a card until explicitly released',()=>{
 card.setManual(true);card.forceHome();card.mouseLeft();mock.timers.tick(33);card.pinned=true;
 mock.timers.tick(100);assert.equal(card.state,'home');card.pinned=false;card.refreshHold();mock.timers.tick(50);assert.equal(card.state,'hidden');
});
