import type {AgentRequest,NativeApprovalChoice} from '../core/bridge';
import {h} from './dom';
import {t} from '../i18n/i18n';

/** Both conversation surfaces select the same Rust-owned native choice ID. */
export function approvalActions(request:Pick<AgentRequest,'kind'|'approvalScope'|'nativeChoices'>,answer:(allow:boolean,nativeChoice?:string)=>void){
 const el=h('div',{class:'actions'}),buttons:HTMLButtonElement[]=[];
 const button=(label:string,allow:boolean,id?:string)=>{
  const control=h('button',{class:allow?'cli-allow btn primary':'cli-deny btn secondary',text:label,onclick:()=>{if(!control.disabled)answer(allow,id);}}) as HTMLButtonElement;
  buttons.push(control);return control;
 };
 const valid=(v:NativeApprovalChoice)=>v&&typeof v.id==='string'&&v.id.length>0&&v.id.length<=256&&typeof v.label==='string'&&v.label.length>0&&v.label.length<=2048&&typeof v.allow==='boolean'&&['once','turn','session','policy','turnCancellation','nativeRemembered'].includes(v.scope)&&(v.detail===null||v.detail===undefined||typeof v.detail==='string'&&v.detail.length<=256*1024);
 if(request.nativeChoices!==undefined){
  const entries=request.nativeChoices;
  if(!Array.isArray(entries)||!entries.length||entries.length>32||!entries.every(valid)||new Set(entries.map(v=>v.id)).size!==entries.length){
   el.append(h('p',{class:'chat-feedback',role:'status',text:'As escolhas nativas deste pedido não estão disponíveis. Volta a abrir o pedido ou decide no agente original.'}));
  }else for(const choice of entries){
   const row=h('div',{class:'native-approval-choice'}),control=button(choice.label,choice.allow,choice.id);
   control.dataset.nativeChoice=choice.id;control.dataset.scope=choice.scope;row.append(control);
   if(choice.scope==='nativeRemembered')row.append(h('p',{text:'O agente memoriza esta decisão com o alcance da opção nativa.'}));
   if(choice.detail)row.append(h('pre',{class:'cli-detail',text:choice.detail}));
   el.append(row);
  }
 }else{
  el.append(button(request.kind==='question'?t("Send"):request.approvalScope==='recipe'?'Autorizar esta receita':request.approvalScope==='turn'?'Permitir neste turno':t("Allow once"),true),button(t("Deny"),false));
 }
 return {el,setDisabled:(disabled:boolean)=>{for(const control of buttons)control.disabled=disabled;}};
}
