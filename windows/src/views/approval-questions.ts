import type {AgentRequest} from '../core/bridge';
import {h} from './dom';

/** Native form values are Rust-owned. Controls send only their captured field
 * and option IDs; optional omission is distinct from an explicit empty value. */
export function approvalQuestions(request:Pick<AgentRequest,'questions'|'nativeForm'|'detail'>){
 const el=h('div',{class:'approval-questions'});
 const rows:{id:string;included:()=>boolean;read:()=>string[];sync:()=>void}[]=[];
 let disabled=false;
 if(request.nativeForm==='mcp')el.append(h('pre',{class:'cli-detail',text:request.detail}));
 for(const question of request.questions??[]){
  const row=h('div',{class:'approval-question'}),typed=question.inputType!==undefined;
  row.dataset.question=question.id;
  row.append(h('label',{text:question.question+(typed?(question.required?' · obrigatório':' · opcional'):'')}));
  if(question.hint)row.append(h('p',{class:'approval-hint',text:question.hint}));
  const include=typed&&!question.required?h('input',{type:'checkbox','aria-label':'Preencher este campo'}) as HTMLInputElement:null;
  if(include){include.checked=false;row.append(h('label',{},include,'Preencher este campo'));}
  const included=()=>!include||include.checked;
  const fields:(HTMLInputElement|HTMLSelectElement|HTMLButtonElement)[]=[];
  let read:()=>string[];
  if(question.inputType==='multi'){
   const checks=(question.options??[]).map(option=>{
    const input=h('input',{type:'checkbox','aria-label':option.label}) as HTMLInputElement;input.checked=false;
    input.value=option.value??'';fields.push(input);row.append(h('label',{},input,option.label));return input;
   });
   read=()=>checks.filter(c=>c.checked).map(c=>c.value);
  }else if(question.inputType==='enum'||question.inputType==='boolean'){
   const select=h('select',{class:'chat-input','aria-label':question.question}) as HTMLSelectElement;
   select.append(h('option',{value:'',text:'Escolhe uma opção'}));
   for(const option of question.options??[])select.append(h('option',{value:option.value??'',text:option.label}));
   select.value='';fields.push(select);row.append(select);read=()=>[select.value];
  }else{
   const numeric=question.inputType==='number'||question.inputType==='integer';
   const input=h('input',{class:'chat-input',type:question.isSecret?'password':numeric?'number':'text',autocomplete:'off',
    'aria-label':question.question,...(numeric?{step:question.inputType==='integer'?'1':'any'}:{})}) as HTMLInputElement;
   fields.push(input);row.append(input);read=()=>[input.value];
   if(!typed)for(const option of question.options??[]){
    const button=h('button',{text:option.label,onclick:()=>{if(!button.disabled)input.value=option.value??option.label;}}) as HTMLButtonElement;
    fields.push(button);row.append(button);
   }
  }
  const sync=()=>{if(include)include.disabled=disabled;for(const field of fields)field.disabled=disabled||!included();};
  include?.addEventListener('change',sync);sync();rows.push({id:question.id,included,read,sync});el.append(row);
 }
 return {el,
  answers:(allow:boolean):Record<string,string[]>=>Object.fromEntries(rows.filter(r=>!allow||r.included()).map(r=>[r.id,allow?r.read():[]])),
  setDisabled:(value:boolean)=>{disabled=value;for(const row of rows)row.sync();},
 };
}
