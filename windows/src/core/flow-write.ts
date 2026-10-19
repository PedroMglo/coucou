import {invoke} from '@tauri-apps/api/core';
import {PendingWrite,type WriteState} from './pending-write';
const writers=new Map<string,PendingWrite<Record<string,unknown>>>();
const delays=new Map<string,ReturnType<typeof setTimeout>>();
function feedback(command:string,label:string,state:WriteState,retry:()=>void){
 if(typeof document==='undefined')return;
 const pending=delays.get(command);if(pending!==undefined){clearTimeout(pending);delays.delete(command);}
 const remove=()=>{document.getElementById(`flow-write-${command}`)?.remove();const box=document.getElementById('flow-write-feedback');if(box&&!box.childElementCount)box.remove();};
 if(!state.error){remove();if(state.pending)delays.set(command,setTimeout(()=>draw(false),400));return;}
 draw(true);
 function draw(error:boolean){
  delays.delete(command);let box=document.getElementById('flow-write-feedback');
  if(!box){box=document.createElement('div');box.id='flow-write-feedback';box.style.cssText='position:fixed;inset:10px 14px auto;z-index:2147483646;display:flex;flex-direction:column;gap:8px;pointer-events:none;max-height:38vh;overflow:auto';document.body.append(box);}
  let row=document.getElementById(`flow-write-${command}`);
  if(!row){row=document.createElement('div');row.id=`flow-write-${command}`;box.append(row);}
  row.replaceChildren();row.setAttribute('role',error?'alert':'status');row.style.cssText=`display:flex;align-items:center;gap:12px;padding:11px 13px;pointer-events:auto;border:1px solid ${error?'#e6a077':'#ffffff25'};border-radius:10px;background:${error?'#2c211a':'#172121'};color:#f1e7df;font:12px/1.5 system-ui;box-shadow:0 4px 20px #0005`;
  const text=document.createElement('span');text.style.flex='1';text.textContent=error?`${label}: alterações ainda por guardar. ${state.error}`:`A guardar ${label.toLowerCase()}…`;row.append(text);
  if(error){const button=document.createElement('button');button.type='button';button.textContent='Tentar novamente';button.style.cssText='color:inherit;background:#ffffff0d;border:1px solid #ffffff30;border-radius:7px;padding:6px 9px;cursor:pointer;font:inherit';button.onclick=retry;row.append(button);}
 }
}
/** Complete snapshots are queued per command; no text enters chat logs. */
export function flowWrite(command:string,label:string,args:Record<string,unknown>):Promise<boolean>{
 let writer=writers.get(command);
 if(!writer){const created:PendingWrite<Record<string,unknown>>=new PendingWrite(value=>invoke(command,value),state=>feedback(command,label,state,()=>{void created.retry();}));writers.set(command,created);writer=created;}
 return writer.write(args);
}
