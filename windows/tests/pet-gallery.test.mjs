import test from 'node:test';import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';import {petGallery} from '../src/desktop/pet-gallery.ts';
const pet=(id,state='idle')=>({id,name:id,color:'#2dd4bf',state,attention:false,busy:false,dancing:false,active:false,activity:'',outfit:'none',sessions:0});
test('the general home includes every available pet, including inactive and sleeping pets',()=>{
 installFakeDom();const g=petGallery(()=>{},()=>{});g.sync([pet('one'),pet('two','sleeping')]);
 assert.equal(g.el.find('button').length,2);assert.match(g.el.textContent,/Em descanso/);assert.match(g.el.textContent,/2 disponíveis/);
});
test('live updates retain the card and route clicks to the latest exact pet identity',()=>{
 installFakeDom();let opened,session;const g=petGallery(p=>opened=p,p=>session=p);g.sync([pet('one')]);const card=g.el.find('button')[0];
 const next={...pet('one','approval'),attention:true};g.sync([next]);assert.equal(g.el.find('button')[0],card);card.onclick();assert.equal(opened,next);card.oncontextmenu({preventDefault(){}});assert.equal(session,next);assert.match(card.textContent,/Rever pedido/);
});
test('removing a pet removes its entry without replacing surviving pets',()=>{
 installFakeDom();const g=petGallery(()=>{},()=>{});g.sync([pet('one'),pet('two')]);const survivor=g.el.find('button')[1];g.sync([pet('two')]);assert.deepEqual(g.el.find('button'),[survivor]);
});

test('catalogue ordering changes without losing existing cards',()=>{
 installFakeDom();const g=petGallery(()=>{},()=>{});g.sync([pet('one'),pet('two')]);const cards=g.el.find('button');g.sync([pet('two'),pet('one')]);assert.deepEqual(g.el.find('button'),[cards[1],cards[0]]);
});

for(const state of ['error','ratelimit'])test(`${state} opens an explanation without promising an approval`,()=>{
 installFakeDom();let opened;const g=petGallery(p=>opened=p,()=>{}),identity={...pet('one',state),attention:true};g.sync([identity]);
 const card=g.el.find('button')[0];assert.match(card.textContent,/Ver aviso/);assert.doesNotMatch(card.textContent,/Rever pedido/);card.onclick();assert.equal(opened,identity);
});

test('a resolved request updates the same card back to its normal action',()=>{
 installFakeDom();const g=petGallery(()=>{},()=>{});g.sync([{...pet('one','question'),attention:true}]);const card=g.el.find('button')[0];assert.match(card.textContent,/Rever pedido/);
 g.sync([pet('one')]);assert.equal(g.el.find('button')[0],card);assert.match(card.textContent,/Abrir pet/);assert.doesNotMatch(card.textContent,/Rever pedido/);
});
