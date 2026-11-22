import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {h} from '../src/views/dom.ts';

installFakeDom();

test('ARIA boolean states expose both on and off as explicit strings',()=>{
 const on=h('button',{'aria-pressed':true,'aria-expanded':true});
 const off=h('button',{'aria-pressed':false,'aria-expanded':false});
 for(const name of ['aria-pressed','aria-expanded']){
  assert.equal(on.getAttribute(name),'true');
  assert.equal(off.getAttribute(name),'false');
 }
 assert.equal(h('div',{'aria-hidden':true}).getAttribute('aria-hidden'),'true');
 assert.equal(h('div',{'aria-hidden':false}).getAttribute('aria-hidden'),'false');
});

test('ordinary HTML boolean attributes still use presence or absence',()=>{
 assert.equal(h('button',{disabled:true}).getAttribute('disabled'),'');
 assert.equal(h('button',{disabled:false}).getAttribute('disabled'),null);
 assert.equal(h('input',{readonly:true}).getAttribute('readonly'),'');
 assert.equal(h('input',{readonly:false}).getAttribute('readonly'),null);
 assert.equal(h('button',{'aria-pressed':'mixed'}).getAttribute('aria-pressed'),'mixed');
});
