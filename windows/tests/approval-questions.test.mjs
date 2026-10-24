import test from 'node:test';
import assert from 'node:assert/strict';
import {installFakeDom} from './fakedom.mjs';
import {approvalQuestions} from '../src/views/approval-questions.ts';
installFakeDom();
const build=questions=>approvalQuestions({nativeForm:'mcp',detail:'MCP fixture · confirm the original request',questions});
const q=(id,inputType,extra={})=>({id,question:`Question ${id}`,inputType,required:true,...extra});
test('friendly option titles send their captured IDs rather than the visible label',()=>{
 const view=build([q('field','enum',{options:[{label:'Friendly choice',value:'field-option-0'}]})]);const select=view.el.find('select')[0];assert.equal(select.value,'');select.value='field-option-0';assert.deepEqual(view.answers(true),{field:['field-option-0']});assert.match(view.el.textContent,/Friendly choice/);
});
test('optional omission remains distinct from an explicitly included empty string',()=>{
 const view=build([q('optional','text',{required:false})]);const [include,input]=view.el.find('input');assert.equal(include.checked,false);assert.equal(input.disabled,true);assert.deepEqual(view.answers(true),{});include.checked=true;include.fire('change');assert.equal(input.disabled,false);assert.deepEqual(view.answers(true),{optional:['']});include.checked=false;include.fire('change');assert.deepEqual(view.answers(true),{});
});
test('multi-select sends every checked option ID and preserves an explicit empty array',()=>{
 const view=build([q('many','multi',{options:[{label:'First',value:'many-option-0'},{label:'Second',value:'many-option-1'}]})]);const inputs=view.el.find('input');assert.deepEqual(view.answers(true),{many:[]});inputs[0].checked=true;inputs[1].checked=true;assert.deepEqual(view.answers(true),{many:['many-option-0','many-option-1']});
});
test('false Boolean is an explicit choice and no supplied default is selected or displayed',()=>{
 const view=build([q('flag','boolean',{default:'PRIVATE_DEFAULT',options:[{label:'Sim',value:'yes-id'},{label:'Não',value:'no-id'}]})]);const select=view.el.find('select')[0];assert.equal(select.value,'');assert.ok(!view.el.textContent.includes('PRIVATE_DEFAULT'));select.value='no-id';assert.deepEqual(view.answers(true),{flag:['no-id']});
});
test('secret fields are blank password inputs and denial never reads their contents',()=>{
 const view=build([q('secret','text',{isSecret:true,default:'PRIVATE_DEFAULT'})]);const input=view.el.find('input')[0];assert.equal(input.getAttribute('type'),'password');assert.equal(input.value,'');input.value='user secret';assert.deepEqual(view.answers(false),{secret:[]});assert.ok(!view.el.textContent.includes('user secret'));assert.ok(!view.el.textContent.includes('PRIVATE_DEFAULT'));
});
test('delivery disables fields and omission controls then restores the same values on retry',()=>{
 const view=build([q('one','text'),q('two','text',{required:false})]);const inputs=view.el.find('input');inputs[0].value='original';inputs[1].checked=true;inputs[1].fire('change');inputs[2].value='optional';view.setDisabled(true);assert.ok(inputs.every(i=>i.disabled));view.setDisabled(false);assert.ok(inputs.every(i=>!i.disabled));assert.deepEqual(view.answers(true),{one:['original'],two:['optional']});
});
test('legacy user-input questions retain their original free-text and label options',()=>{
 const view=approvalQuestions({detail:'',questions:[{id:'legacy',question:'Where?',options:[{label:'Original'}]}]});view.el.find('button')[0].fire('click');assert.deepEqual(view.answers(true),{legacy:['Original']});assert.deepEqual(view.answers(false),{legacy:[]});
});
test('native context and hints render as text without interpreting markup',()=>{
 const view=build([q('safe','text',{question:'<script>fixture</script>',hint:'Minimum: 2'})]);assert.equal(view.el.find('SCRIPT').length,0);assert.match(view.el.textContent,/<script>fixture<\/script>/);assert.match(view.el.textContent,/Minimum: 2/);assert.match(view.el.textContent,/confirm the original request/);
});
