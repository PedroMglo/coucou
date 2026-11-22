import {getCurrentWindow} from '@tauri-apps/api/window';
import {invoke} from '@tauri-apps/api/core';
import {Bridge,onEvent} from '../core/bridge';
import {petsPage} from '../hub/pages/pets';
import type {HubContext} from '../hub/shell';
interface Context{pet?:string;section?:string}
const root=document.getElementById('manager')!;let current:Context|null=null;let ctx:HubContext|undefined;
async function open(value:Context){
 if(current&&current.pet===value.pet&&current.section===value.section)return;
 current=value;ctx?.onLeave?.();root.replaceChildren();
 ctx={petSelection:value.pet,petSection:value.section??'sessions',initialVisible:await getCurrentWindow().isVisible()} as HubContext;
 await petsPage(root,ctx);
}
document.getElementById('close')!.onclick=()=>void Bridge.petPanelHide();
document.addEventListener('keydown',event=>{if(event.key==='Escape'){event.preventDefault();void Bridge.petPanelHide();}});
await onEvent<Context>('pet-panel-opened',value=>void open(value));
await open(await invoke<Context>('pet_panel_context'));
