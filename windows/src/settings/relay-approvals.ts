import {h} from '../views/dom';
import {parseRelayApprovalStatus,type RelayApprovalMode,type RelayApprovalStatus} from '../core/relay-approvals';
export function buildRelayApprovalRouting(transport:{read():Promise<unknown>;change(request:{mode:RelayApprovalMode;revision:string}):Promise<unknown>}){
 let state:RelayApprovalStatus|null=null,busy=false;
 const status=h('p',{class:'hint',role:'status'}),error=h('p',{class:'notice err',role:'alert'});
 const terminal=h('button',{type:'button',text:'Decidir no terminal'}) as HTMLButtonElement;
 const coucou=h('button',{type:'button',text:'Decidir no Coucou'}) as HTMLButtonElement;
 const refresh=h('button',{type:'button',text:'Consultar modo'}) as HTMLButtonElement;
 const el=h('div',{class:'relay-approval-routing'},h('strong',{text:'Aprovações de terminais externos'}),h('p',{class:'hint',text:'Só nesta execução. No modo terminal, o Coucou observa as sessões e devolve imediatamente os pedidos ao agente. As tarefas iniciadas nos pets conservam as suas próprias aprovações.'}),status,h('div',{class:'row'},terminal,coucou,refresh),error);
 function render(){
  terminal.disabled=busy||!state||state.mode==='terminal';coucou.disabled=busy||!state||state.mode==='coucou';refresh.disabled=busy;
  terminal.setAttribute('aria-pressed',String(state?.mode==='terminal'));coucou.setAttribute('aria-pressed',String(state?.mode==='coucou'));el.setAttribute('aria-busy',String(busy));
  status.textContent=state?`Modo confirmado: ${state.mode==='terminal'?'no terminal':'no Coucou'}. Ao reiniciar: ${state.restartMode==='terminal'?'no terminal':'no Coucou'}${state.developmentBuild?' (build de desenvolvimento)':''}.`:'Modo por confirmar. Consulta antes de alterar.';
 }
 async function run(mode?:RelayApprovalMode){
  if(busy||mode&&!state)return;const previous=state;busy=true;error.textContent='';render();
  try{state=parseRelayApprovalStatus(mode?await transport.change({mode,revision:previous!.revision}):await transport.read());}
  catch(e){state=null;error.textContent=String(e).replace(/^Error:\s*/,'');}
  finally{busy=false;render();}
 }
 terminal.onclick=()=>void run('terminal');coucou.onclick=()=>void run('coucou');refresh.onclick=()=>void run();render();void run();
 return {el,refresh:()=>run()};
}
