import {h,clear} from '../views/dom';
import type {ShortcutReviewActions,ShortcutReviewPreview} from '../core/shortcuts-review';
export function buildShortcutReview(actions:ShortcutReviewActions){
 const el=h('div',{'data-shortcuts-review':'native'});let preview:ShortcutReviewPreview|null=null,busy=false,applying=false,disposed=false,sequence=0,message='',observation='';
 let reviewed=false,impact=false,scope=false,uncertainty=false;
 function key(p:ShortcutReviewPreview){return {token:p.token,fingerprint:p.fingerprint};}
 async function forget(p:ShortcutReviewPreview){try{await actions.cancel(key(p));}catch{/* Status, not retry, resolves a lost cancellation ACK. */}}
 function invalidate(){
  const old=preview;reviewed=impact=scope=uncertainty=false;observation='';
  if(applying){message='Cancelamento pedido. A aguardar o resultado da tentativa original; o cancelamento não desfaz efeitos anteriores.';if(old)void forget(old);render();return;}
  sequence++;preview=null;busy=false;message='';if(old)void forget(old);render();
 }
 async function prepare(currentScope=false){if(busy||disposed)return;const ticket=++sequence;busy=true;message='';observation='';preview=null;reviewed=impact=scope=uncertainty=false;render();
  try{const p=await (currentScope?actions.prepareCurrentScope!():actions.prepare());if(disposed||sequence!==ticket){void forget(p);return;}preview=p;message=p.message;}
  catch(error){if(!disposed&&sequence===ticket)message=error instanceof Error?error.message:String(error);}
  finally{if(!disposed&&sequence===ticket){busy=false;render();}}
 }
 async function inspect(){if(!preview||busy||disposed)return;const p=preview,ticket=++sequence;busy=true;render();
  try{const checked=await actions.inspect(key(p));if(!disposed&&sequence===ticket){preview=checked;message=checked.message;}}
  catch(error){if(!disposed&&sequence===ticket){preview=null;message=error instanceof Error?error.message:String(error);}}
  finally{if(!disposed&&sequence===ticket){busy=false;render();}}
 }
 async function apply(){
  if(!preview||![2,3].includes(preview.schemaVersion)||!actions.apply||busy||disposed||!reviewed||!impact||!scope||preview.schemaVersion===3&&!uncertainty)return;
  const p=preview,ticket=++sequence;busy=applying=true;message='A aguardar o executor e, se aplicável, a confirmação nativa. Não repetir a tentativa.';render();
  try{const receipt=await actions.apply({...key(p),reviewedPlan:true,understandNativeEffects:true,keepOtherComponentsSuspended:true,...(p.schemaVersion===3?{previewSchemaVersion:3 as const,acknowledgeRetainedUncertainty:true}:{})});if(!disposed&&sequence===ticket){observation=JSON.stringify(receipt,null,2);message=receipt.message;}}
  catch(error){if(!disposed&&sequence===ticket)message=`Não foi recebido um resultado confirmado. Consulta o estado; não repitas automaticamente. ${error instanceof Error?error.message:String(error)}`;}
  finally{if(!disposed&&sequence===ticket){preview=null;busy=applying=false;reviewed=impact=scope=uncertainty=false;sequence++;render();}}
 }
 async function status(){if(!actions.status||disposed)return;const ticket=sequence;
  try{const value=await actions.status();if(!disposed&&sequence===ticket){observation=JSON.stringify(value,null,2);render();}}
  catch(error){if(!disposed&&sequence===ticket){observation=error instanceof Error?error.message:String(error);render();}}
 }
 function check(id:string,text:string,value:boolean,set:(v:boolean)=>void){
  const input=h('input',{type:'checkbox',checked:value,disabled:busy,'data-shortcuts-review-ack':id,onchange:()=>{set(input.checked);render();}});return h('label',{},input,h('span',{text}));
 }
 function render(){if(disposed)return;clear(el);el.append(h('button',{'data-shortcuts-review-action':'prepare',text:'Rever destino e teclas',disabled:busy,onclick:()=>void prepare()}));
  if(actions.prepareCurrentScope)el.append(h('button',{'data-shortcuts-review-action':'prepare-current-scope',text:'Rever novo pedido no contexto atual',disabled:busy,onclick:()=>void prepare(true)}));
  if(preview){el.append(h('button',{'data-shortcuts-review-action':'inspect',text:'Verificar se a revisão continua válida',disabled:busy,onclick:()=>void inspect()}));el.append(h('pre',{'data-shortcuts-review-details':'true',text:JSON.stringify({destino:preview.destination,origem:preview.origin,configuracao:preview.configuration,recurso:preview.resource,expiraEm:new Date(preview.expiresAtMs).toISOString(),preferencias:preview.desiredBindings,acoes:preview.actions,acoesDesconhecidas:preview.unknownActionIds,...(preview.recovery?{historicoConservado:preview.recovery}:{})},null,2)}));
   if([2,3].includes(preview.schemaVersion)&&preview.approvalAvailable&&actions.apply){el.append(check('plan','Revi o destino, as teclas e os registos anteriores que serão libertados.',reviewed,v=>{reviewed=v;}),check('impact','Autorizo esta tentativa nativa e compreendo a possibilidade de efeitos parciais e confirmação pelo ambiente gráfico.',impact,v=>{impact=v;}),check('scope','Esta aprovação abrange apenas estes atalhos; outros registos e reinícios exigem nova revisão.',scope,v=>{scope=v;}),...(preview.schemaVersion===3?[check('uncertainty','Autorizo um novo pedido no contexto atual. O encerramento dos efeitos antigos continua incerto; pode haver conflito e, no portal, configurações partilhadas pela aplicação.',uncertainty,v=>{uncertainty=v;})]:[]),h('button',{'data-shortcuts-review-action':'apply',text:'Aprovar esta tentativa nativa',disabled:busy||!reviewed||!impact||!scope||preview.schemaVersion===3&&!uncertainty,onclick:()=>void apply()}));}
  }
  if(preview||busy)el.append(h('button',{'data-shortcuts-review-action':'cancel',text:'Cancelar revisão',onclick:invalidate}));
  if(actions.status)el.append(h('button',{'data-shortcuts-review-action':'status',text:'Consultar estado sem repetir',onclick:()=>void status()}));
  if(message)el.append(h('p',{'data-shortcuts-review-message':'true',text:message}));
  if(observation)el.append(h('pre',{'data-shortcuts-review-observation':'true',text:observation}));
 }
 render();return {el,invalidate,dispose(){if(disposed)return;invalidate();disposed=true;sequence++;},render};
}
