import {Bridge} from '../core/bridge';
import {h} from '../views/dom';
import {t} from '../i18n/i18n';
import {statusDot} from './parts';

/** Saving an empty field must never silently remove an existing credential. */
export function secretControls(key:string,stored:boolean,placeholder:string,onChanged:(stored:boolean)=>void){
 const input=h('input',{type:'password',autocomplete:'off',spellcheck:'false',style:'flex:1 1 auto;min-width:0'}) as HTMLInputElement;
 const save=h('button',{text:t('Save')}) as HTMLButtonElement;
 const remove=h('button',{text:t('Remove'),class:'danger'}) as HTMLButtonElement;
 const dot=statusDot(stored),feedback=h('div',{class:'hint',role:'status','aria-live':'polite'});
 let pending=false;
 function refresh(){input.placeholder=stored?'••••••••  '+t('(stored)'):placeholder;input.disabled=pending;save.disabled=pending||!input.value.trim();remove.disabled=pending;remove.hidden=!stored;dot.style.background=stored?'#22c55e':'#f4505e';}
 async function apply(clear:boolean){
  const value=input.value.trim();if(pending||(!clear&&!value))return;
  pending=true;feedback.className='hint';feedback.textContent=clear?'A remover…':'A guardar…';refresh();
  try{
   if(clear)await Bridge.secretClear(key);else await Bridge.secretSet(key,value);
   stored=!clear;input.value='';onChanged(stored);feedback.textContent=clear?'Removido.':'Guardado.';
  }catch(error){feedback.className='notice err';feedback.textContent=String(error).replace(/^Error:\s*/,'');}
  finally{pending=false;refresh();}
 }
 input.addEventListener('input',refresh);input.addEventListener('keydown',e=>{if(e.key==='Enter'&&!e.isComposing){e.preventDefault();void apply(false);}});
 save.onclick=()=>void apply(false);remove.onclick=()=>void apply(true);refresh();
 return {input,save,remove,dot,feedback};
}
